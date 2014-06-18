#![crate_id = "asm_format#0.11-pre"]
#![crate_type="dylib"]
#![feature(managed_boxes, globs, plugin_registrar, macro_rules, quote)]
#![experimental]

extern crate syntax;
extern crate rustc;
extern crate collections;
extern crate fmt_macros;

use syntax::ast;
use syntax::ast::TokenTree;
use syntax::codemap::Span;
use syntax::ext::base;
use syntax::ext::base::*;
use syntax::parse;
use syntax::parse::token;
use syntax::parse::token::InternedString;
use rustc::plugin::Registry;

use std::vec::Vec;
use std::string::String;
use std::collections::HashMap;
use std::gc::{Gc, GC};

#[plugin_registrar]
pub fn registrar(reg: &mut Registry) {
    reg.register_macro("asm_format", expand_asm_format);
}

enum AsmPiece<'a> {
    String(&'a str),
    Output(uint),
    Input(uint),
}

struct Context<'a, 'b> {
    asm_str: String,
    expr: ast::InlineAsm,
    dummy: Option<Box<MacResult>>,
    ecx: &'a mut ExtCtxt<'b>,

    // Parsed argument expressions and the types that we've found so far for
    // them.
    // TODO try to use HashMap<(uint, ~str), uint>
    args: Vec<(Gc<ast::Expr>, Option<Gc<ast::Expr>>, Option<uint>)>,
    // as used in the string. (args_idx, ty)
    arg_outputs: Vec<(uint, String)>,
    arg_inputs: Vec<(uint, String)>,

    // Names of parsed named expressions mapped to their index into args.
    names: HashMap<String, uint>,

    // Updated as arguments are consumed or methods are entered
    next_argument: uint,
}

#[inline]
fn fetch_idx(arg: &mut Vec<(uint, String)>, key: (uint, String)) -> uint {
    match arg.as_slice().position_elem(&key) {
        Some(idx) => idx,
        None => {
            let last_idx = arg.len();
            arg.push(key);
            last_idx
        }
    }
}

impl<'a, 'b> Context<'a, 'b> {
    fn trans_piece<'a>(&mut self, piece: fmt_macros::Piece<'a>, sp: Span) -> AsmPiece<'a> {
        use fmt_macros::{Argument, ArgumentNext, ArgumentIs, ArgumentNamed,
            CountImplied, FormatSpec, AlignUnknown, AlignLeft};

        match piece {
            fmt_macros::String(s) => String(s),
            fmt_macros::Argument(Argument {
                position: pos,
                format: FormatSpec {
                    ty: ty,
                    align: align,
                    fill: None, flags: 0, precision: CountImplied, width: CountImplied,
                },
            }) => {
                let key = match pos {
                    ArgumentNext => {
                        let key = self.next_argument;
                        self.next_argument += 1;
                        key
                    }
                    ArgumentIs(n) => n,
                    ArgumentNamed(name) => match self.names.find(&name.to_string()) {
                        Some(&n) => n,
                        None => {
                            self.ecx.span_err(sp, format!("there is no argument named `{}`",
                                                            name).as_slice());
                            self.dummy = Some(DummyResult::expr(sp));
                            return String("");
                        }
                    }
                };

                match align {
                    AlignLeft => {
                        Output(fetch_idx(&mut self.arg_outputs, (key, ty.to_string())))
                    }
                    AlignUnknown => {
                        Input(fetch_idx(&mut self.arg_inputs, (key, ty.to_string())))
                    }
                    _ => fail!("invalid align")
                }
            }
            _ => fail!("methods not implemented")
        }
    }

    fn format_pieces<'a>(&mut self, s: &'a InternedString, sp: Span) -> Vec<AsmPiece<'a>> {
        let mut pieces = Vec::new();
        let asm_str = s.get();
        let mut parser = fmt_macros::Parser::new(asm_str);
        loop {
            match parser.next() {
                Some(piece) => {
                    if parser.errors.len() > 0 { break }
                    // self.verify_piece(&piece);
                    pieces.push(self.trans_piece(piece, sp));
                }
                None => break
            }
        }

        match parser.errors.shift() {
            Some(error) => {
                self.ecx.span_err(sp, format!("invalid format string: {}", error).as_slice());
                self.dummy = Some(DummyResult::expr(sp));
            }
            None => {}
        }
        pieces
    }

    fn into_expr(mut self, sp: Span) -> Box<MacResult> {
        //-----
        println!("{}", self.asm_str);
        for (a, _) in self.expr.outputs.clone().move_iter() {
            println!("out: {}", a)
        }
        for (a, _) in self.expr.inputs.clone().move_iter() {
            println!("in: {}", a)
        }
        //-----
        self.expr.asm = token::intern_and_get_ident(self.asm_str.as_slice());
        match self.dummy {
            Some(e) => e,
            None => base::MacExpr::new(box(GC) ast::Expr {
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
        next_argument: 0,
        asm_str: String::new(),
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
        dummy: None
    };

    let mut clobs = Vec::new();

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
                } else {
                    clobs.push(option.get().to_string());
                }

                p.expect(&token::COMMA);
            }
            _ => break
        }
    }

    let clobbers = format!("~{{{}}}", clobs.connect("},~{"));
    cx.expr.clobbers = token::intern_and_get_ident(clobbers.as_slice());

    let asm_expr = p.parse_expr();
    let (asm_str, _) = match expr_to_str(cx.ecx, asm_expr,
                                       "inline assembly must be a string literal.") {
        Some(tuple) => tuple,
        // let the compilation continue
        None => return DummyResult::expr(sp),
    };

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
                    // "sel" = in_expr
                    p.expect(&token::EQ);
                    let e = p.parse_expr();
                    cx.expr.inputs.push((s.clone(), e));
                    if p.eat(&token::RARROW) {
                        // -> out_expr
                        let out = p.parse_expr();
                        let ident = token::intern_and_get_ident(format!("={}", s).as_slice());
                        cx.expr.outputs.push((ident, out));
                    }
                }
                None => {
                    // ident = in_expr
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
                                        format!("expected ident for named argument, \
                                                but found `{}`",
                                                p.this_token_to_str()).as_slice());
                            return DummyResult::expr(sp);
                        }
                    };
                    let interned_name = token::get_ident(ident);
                    let name = interned_name.get();
                    p.expect(&token::EQ);
                    let e = p.parse_expr();
                    match cx.names.find_equiv(&name) {
                        None => {}
                        Some(&idx) => match cx.args.get(idx) {
                            &(prev, _, _) => {
                                cx.ecx.span_err(e.span,
                                                format!("duplicate argument named `{}`",
                                                        name).as_slice());
                                cx.ecx.parse_sess.span_diagnostic.span_note(prev.span,
                                                                            "previously here");
                                continue
                            }
                        }
                    }

                    cx.names.insert(name.to_str(), cx.args.len());

                    if p.eat(&token::RARROW) {
                        // -> out_expr
                        let out = p.parse_expr();
                        cx.args.push((e, Some(out), None));
                    }
                    else {
                        cx.args.push((e, None, None));
                    }
                }
            }
        } else {
            // in_expr
            let e = p.parse_expr();
            if p.eat(&token::RARROW) {
                // -> out_expr
                let out = p.parse_expr();
                cx.args.push((e, Some(out), None));
                // TODO ensure out_expr = in_expr?
            }
            else {
                cx.args.push((e, None, None));
            }
        }
    }

    // Translation of pieces.
    let pieces = cx.format_pieces(&asm_str, asm_expr.span);

    let offset_outputs = cx.expr.outputs.len();
    let offset_inputs = offset_outputs + cx.arg_outputs.len() + cx.expr.inputs.len();

    let len = cx.args.len();
    for &(a, ref b) in cx.arg_outputs.mut_iter() {
        match cx.args.get_mut(a) {
            ref mut t if a < len => {
                *t.mut2() = Some(cx.expr.outputs.len());
                match *t {
                    &(_, Some(out), _) | &(out, _, _) => {
                        let ident = token::intern_and_get_ident(format!("={}", b).as_slice());
                        cx.expr.outputs.push((ident, out));
                    }
                }
            }
            _ => {
                cx.ecx.span_err(sp, "no such output");
                return DummyResult::expr(sp);
            }
        }
    }

    for &(a, ref b) in cx.arg_inputs.iter() {
        match cx.args.get(a) {
            &(out, _, None) if a < cx.args.len() => {
                cx.expr.inputs.push((token::intern_and_get_ident(b.as_slice()), out));
            }
            &(out, _, Some(idx)) if a < cx.args.len() => {
                cx.expr.inputs.push((token::intern_and_get_ident(idx.to_str().as_slice()), out));
            }
            _ => {
                cx.ecx.span_err(sp, "no such input");
                return DummyResult::expr(sp);
            }
        }
    }

    // Transcription and concatenation of pieces.
    let mut strs = pieces.iter().map(|p| match p {
        &String(s) => String(s),
        &Output(n) => Output(n + offset_outputs),
        &Input(n) => Input(n + offset_inputs),
    });

    for p in strs {
        match p {
            String(s) => cx.asm_str.push_str(s),
            Output(i) | Input(i) => {
                cx.asm_str.push_char('$');
                cx.asm_str.push_str(i.to_str().as_slice());
            }
        }
    }

    cx.into_expr(sp)
}
