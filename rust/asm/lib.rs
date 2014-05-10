#![crate_id = "asm_format#0.11-pre"]
#![crate_type="dylib"]
#![feature(managed_boxes, globs, macro_registrar, macro_rules, quote)]
#![experimental]

extern crate syntax;
extern crate collections;
extern crate fmt_macros;

use syntax::ast;
// use syntax::codemap::Span;
use syntax::ext::base;
// use syntax::ext::base::*;
use syntax::parse;
use syntax::parse::token::InternedString;
// use syntax::parse::token;

use syntax::ast::{Name, TokenTree};
use syntax::codemap::Span;
use syntax::ext::base::*;
use syntax::parse::token;

use collections::HashMap;
// use std::intrinsics::transmute;

use std::vec::Vec;
use std::strbuf::StrBuf;

#[macro_registrar]
pub fn macro_registrar  (register: |Name, SyntaxExtension|) {
    register(token::intern("asm_format"),
        NormalTT(box BasicMacroExpander {
            expander: expand_asm_format,
            span: None,
        },
        None));
}

enum AsmPiece<'a> {
    String(&'a str),
    Output(uint),
    OutputNamed(uint),
    Input(uint),
    InputNamed(uint)
}

struct Context<'a, 'b> {
    asm_str: StrBuf,
    expr: ast::InlineAsm,
    expr_f: Option<Box<MacResult>>,
    ecx: &'a mut ExtCtxt<'b>,

    // Parsed argument expressions and the types that we've found so far for
    // them.
    args: Vec<(@ast::Expr, Option<@ast::Expr>)>,
    arg_outputs: Vec<(uint, ~str)>,
    arg_inputs: Vec<(uint, ~str)>,
    // Parsed named expressions and the types that we've found for them so far.
    // Note that we keep a side-array of the ordering of the named arguments
    // found to be sure that we can translate them in the same order that they
    // were declared in.
    names: HashMap<~str, (@ast::Expr, Option<@ast::Expr>)>,
    named_outputs: Vec<(~str, ~str)>,
    named_inputs: Vec<(~str, ~str)>,

    // Collection of strings
    // pieces: ~[~AsmPiece<'a>],
    name_positions: HashMap<~str, uint>,

    // Updated as arguments are consumed or methods are entered
    next_argument: uint, // TODO rename to next_argument
    num_outputs: uint,
    num_named_outputs: uint,
    num_inputs: uint,
}

impl<'a, 'b> Context<'a, 'b> {
    fn trans_piece<'a>(&mut self, piece: fmt_macros::Piece<'a>) -> AsmPiece<'a> {
        use fmt_macros::{Argument, ArgumentNext, ArgumentIs, ArgumentNamed, CountImplied, FormatSpec, AlignUnknown, AlignLeft};

        match piece {
            fmt_macros::String(s) => String(s),
            fmt_macros::Argument(Argument {
                position: pos,
                format: FormatSpec {
                    ty: ty,
                    align: align,
                    fill: None, flags: 0, precision: CountImplied, width: CountImplied,
                },
                method: None,
            }) => {
                let (args, named) = match (align, pos) {
                    (AlignLeft, ArgumentNext) | (AlignLeft, ArgumentIs(_)) => {
                        (Some(&mut self.arg_outputs), None)
                    }
                    (AlignUnknown, ArgumentNext) | (AlignUnknown, ArgumentIs(_)) => {
                        (Some(&mut self.arg_inputs), None)
                    }
                    (AlignLeft, ArgumentNamed(_)) => {
                        (None, Some(&mut self.named_outputs))
                    }
                    (AlignUnknown, ArgumentNamed(_)) => {
                        (None, Some(&mut self.named_inputs))
                    }
                    _ => fail!("invalid align")
                };

                let (key, name_key) = match pos {
                    ArgumentNext => {
                        let key = self.next_argument;
                        self.next_argument += 1;
                        (Some((key, ty.to_owned())), None)
                    }
                    ArgumentIs(n) => (Some((n, ty.to_owned())), None),
                    ArgumentNamed(name) => (None, Some((name.to_owned(), ty.to_owned())))
                };

                let idx = match ((args, key), (named, name_key)) {
                    ((Some(arg), Some(key)), _) => {
                        match arg.as_slice().position_elem(&key) {
                            Some(idx) => idx,
                            None => {
                                let last_idx = arg.len();
                                arg.push(key);
                                last_idx
                            }
                        }
                    }
                    (_, (Some(arg), Some(key))) => {
                        match arg.as_slice().position_elem(&key) {
                            Some(idx) => idx,
                            None => {
                                let last_idx = arg.len();
                                arg.push(key);
                                last_idx
                            }
                        }
                    }
                    _ => fail!("")
                };

                match (align, pos) {
                    (AlignLeft, ArgumentNext) | (AlignLeft, ArgumentIs(_)) => {
                        Output(idx)
                    }
                    (AlignUnknown, ArgumentNext) | (AlignUnknown, ArgumentIs(_)) => {
                        Input(idx)
                    }
                    (AlignLeft, ArgumentNamed(_)) => {
                        OutputNamed(idx)
                    }
                    (AlignUnknown, ArgumentNamed(_)) => {
                        InputNamed(idx)
                    }
                    _ => fail!("invalid align")
                }
            }
            _ => fail!("methods not impl")
        }
    }

    fn format_pieces<'a>(&mut self, s: &'a InternedString, sp: Span) -> Vec<AsmPiece<'a>> {
        let mut pieces = Vec::new();
        let asm_str = s.get();
        let mut parser = fmt_macros::Parser::new(asm_str);
        loop {
            let n = parser.next();
            match n {
                Some(piece) => {
                    if parser.errors.len() > 0 { break }
                    // verify_piece(cx, &piece);
                    pieces.push(self.trans_piece(piece));
                }
                None => break
            }
        }

        match parser.errors.shift() {
            Some(error) => {
                self.ecx.span_err(sp, "invalid format string: " + error);
                self.expr_f = Some(DummyResult::expr(sp));
            }
            None => {}
        }
        pieces
    }

    // fn to_expr_inline_asm(&self) -> ast::InlineAsm {
    //     let expr = ast::InlineAsm {
    //         asm: token::intern_and_get_ident(self.asm_str.as_slice()),
    //         asm_str_style: ast::CookedStr,
    //         clobbers: self.expr.clobbers.clone(),
    //         inputs: self.expr.inputs.clone(),
    //         outputs: self.expr.outputs.clone(),
    //         volatile: self.expr.volatile.clone(),
    //         alignstack: self.expr.alignstack.clone(),
    //         dialect: self.expr.dialect.clone()
    //     };
    //     expr
    // }

    fn into_expr(mut self, sp: Span) -> Box<MacResult> {
        //-----
        println!("{}", self.asm_str);
        for &(ref a, _) in self.expr.inputs.clone().iter() {
            println!("in: {}", a)
        }
        for &(ref a, _) in self.expr.outputs.clone().iter() {
            println!("out: {}", a)
        }
        //-----
        self.expr.asm = token::intern_and_get_ident(self.asm_str.as_slice());
        match self.expr_f {
            Some(e) => e,
            None => base::MacExpr::new(@ast::Expr {
                id: ast::DUMMY_NODE_ID,
                node: ast::ExprInlineAsm(self.expr),
                span: sp
            })
        }
    }
}

pub fn expand_asm_format(ecx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<base::MacResult> {
    let mut p = parse::new_parser_from_tts(ecx.parse_sess(),
                                           ecx.cfg(),
                                           tts.iter()
                                              .map(|x| (*x).clone())
                                              .collect());
    let mut cx = Context {
        ecx: ecx,
        args: Vec::new(),
        arg_outputs: Vec::new(),
        arg_inputs: Vec::new(),
        names: HashMap::new(),
        name_positions: HashMap::new(),
        named_outputs: Vec::new(),
        named_inputs: Vec::new(),
        next_argument: 0,
        num_outputs: 0,
        num_named_outputs: 0,
        num_inputs: 0,
        // pieces: ~[],
        asm_str: StrBuf::new(),
        expr: ast::InlineAsm {
            asm: InternedString::new(""),
            asm_str_style: ast::CookedStr,
            clobbers: InternedString::new(""),
            inputs: Vec::new(),
            outputs: Vec::new(),
            volatile: false,
            alignstack: false,
            dialect: ast::AsmAtt
        },
        expr_f: None
    };

    let mut clobs = Vec::new();
    // let mut apieces = Vec::new();
    loop {
        match p.token {
            token::IDENT(_, false) => {
                let option = token::get_ident(p.parse_ident());

                if option.equiv(&("volatile")) {
                    cx.expr.volatile = true;
                } else if option.equiv(&("alignstack")) {
                    cx.expr.alignstack = true;
                } else if option.equiv(&("intel")) {
                    cx.expr.dialect = ast::AsmIntel;
                } else { //if option.is_some() {
                    let clob = format!("~\\{{}\\}", option.get().to_owned());
                    clobs.push(clob);
                }

                p.expect(&token::COMMA);
            }
            _ => break
        }
    }

    cx.expr.clobbers = token::intern_and_get_ident(clobs.connect(","));

    let asm_expr = p.parse_expr();
    let (asm_str, style) = match expr_to_str(cx.ecx, asm_expr,
                                       "inline assembly must be a string literal.") {
        Some(tuple) => tuple,
        // let compilation continue
        None => return DummyResult::expr(sp),
    };

    // apieces.push((s, asm_expr.span));

    let mut named = false;
    while p.token != token::EOF {
        if !p.eat(&token::COMMA) {
            cx.ecx.span_err(sp, "expected token: `,`");
            return DummyResult::expr(sp);
        }
        if p.token == token::EOF { break } // accept trailing commas

        let tok_str = p.parse_optional_str();
        if named || ((tok_str.is_some() && p.token == token::EQ) ||
                     (token::is_ident(&p.token) && p.look_ahead(1, |t| *t == token::EQ))) {
            named = true;
            match tok_str {
                Some((s, _)) => {
                    p.expect(&token::EQ);
                    let e = p.parse_expr();
                    cx.expr.inputs.push((s.clone(), e));
                    cx.num_inputs += 1;
                    if p.token == token::RARROW {
                        p.bump();
                        let out = p.parse_expr();
                        cx.expr.outputs.push((token::intern_and_get_ident("=" + s.get()), out));
                        cx.num_outputs += 1;
                    }
                }
                None => {
                    let ident = match p.token {
                        token::IDENT(i, _) => {
                            p.bump();
                            i
                        }
                        _ if named => {
                            cx.ecx.span_err(p.span,
                                         "expected ident, positional arguments \
                                         cannot follow named arguments");
                            return DummyResult::expr(sp);
                        }
                        _ => {
                            cx.ecx.span_err(p.span,
                                         format!("expected ident for named argument, but found `{}`",
                                                 p.this_token_to_str()));
                            return DummyResult::expr(sp);
                        }
                    };
                    let interned_name = token::get_ident(ident);
                    let name = interned_name.get();
                    p.expect(&token::EQ);
                    let e = p.parse_expr();
                    match cx.names.find_equiv(&name) {
                        None => {}
                        Some(&(prev, _)) => {
                            cx.ecx.span_err(e.span, format!("duplicate argument named `{}`", name));
                            cx.ecx.parse_sess.span_diagnostic.span_note(prev.span, "previously here");
                            continue
                        }
                    }

                    if p.token == token::RARROW {
                        p.bump();
                        let out = p.parse_expr();
                        cx.names.insert(name.to_str(), (e, Some(out)));
                    }
                    else {
                        cx.names.insert(name.to_str(), (e, None));
                    }
                }
            }
        } else {
            let e = p.parse_expr();
            if p.token == token::RARROW {
                p.bump();
                let out = p.parse_expr();
                cx.args.push((e, Some(out)));
            }
            else {
                cx.args.push((e, None));
            }
        }
    }

    // let mut pieces = Vec::new();
    // for &(ref pcs, ref pspan) in apieces.iter() {
    //     for &p in cx.format_pieces(pcs, *pspan).iter() {
    //         pieces.push(p);
    //     }
    // }
    // Translation of pieces.
    let pieces = cx.format_pieces(&asm_str, asm_expr.span);

    cx.num_outputs += cx.arg_outputs.len();
    cx.num_named_outputs += cx.named_outputs.len();
    cx.num_inputs += cx.arg_inputs.len();

    let offset_named_outputs = cx.num_outputs;
    let offset_inputs = offset_named_outputs + cx.num_named_outputs;
    let offset_named_inputs = offset_inputs + cx.num_inputs;

    // Transcription of pieces.
    for &p in pieces.iter() {
        match p {
            String(s) => cx.asm_str.push_str(s),
            Output(i) => cx.asm_str.push_str("$" + i.to_str()),
            OutputNamed(i) => cx.asm_str.push_str("$" + (i + offset_named_outputs).to_str()),
            Input(n) => cx.asm_str.push_str("$" + (n + offset_inputs).to_str()),
            InputNamed(n) => cx.asm_str.push_str("$" + (n + offset_named_inputs).to_str())
        }
    }

    for &(ref a, ref b) in cx.named_outputs.iter() {
        match cx.names.get(a) {
            &(_, Some(out)) | &(out, _) => {
                cx.expr.outputs.push((token::intern_and_get_ident("=" + *b), out));
            }
        }
    }

    for &(ref a, ref b) in cx.named_inputs.iter() {
        match cx.names.get(a) {
            &(inp, _) => {
                cx.expr.inputs.push((token::intern_and_get_ident(*b), inp));
            }
        }
    }

    cx.into_expr(sp)
}

// TODO
fn verify_piece(cx: &mut Context, p: &fmt_macros::Piece) {
}
