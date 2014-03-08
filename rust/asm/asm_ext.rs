#[feature(managed_boxes, globs, macro_registrar, macro_rules, quote)];
extern crate syntax;
extern crate collections;

use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base;
use syntax::ext::base::*;
use syntax::parse;
use syntax::parse::token::InternedString;
use syntax::parse::token;

use syntax::ast::{Name, TokenTree};
use syntax::codemap::Span;
use syntax::ext::base::*;
use syntax::parse::token;

use collections::{HashMap, HashSet};
use std::fmt;
use std::intrinsics::transmute;

#[macro_export]
macro_rules! exported_macro (() => (2))

/*#[macro_registrar]
pub fn macro_registrar(register: |Name, SyntaxExtension|) {
    register(token::intern(&"asm_format"),
        NormalTT(~BasicMacroExpander {
            expander: expand_asm_format, // MacroExpanderFn
            span: None,
        },// as ~MacroExpander:'static,
        None));
}*/

#[macro_registrar]
pub fn macro_registrar(register: |Name, SyntaxExtension|) {
    register(token::intern("asm_format"),
        NormalTT(~BasicMacroExpander {
            expander: expand_asm_format,
            span: None,
        },
        None));
}

enum AsmPiece<'a> {
    String(&'a str),
    Output(uint),
    // OutputNamed(&'a str, uint),
    Input(uint, uint),
    InputNamed(&'a str, &'a str)
}

struct Context<'a> {
    asm_str: ~str,
    expr: ast::InlineAsm,
    expr_f: Option<MacResult>,
    ecx: &'a mut ExtCtxt<'a>,
    fmtsp: ~[Span],

    // Parsed argument expressions and the types that we've found so far for
    // them.
    args: ~[(@ast::Expr, Option<@ast::Expr>)],
    arg_types: ~[~[~str]],
    // Parsed named expressions and the types that we've found for them so far.
    // Note that we keep a side-array of the ordering of the named arguments
    // found to be sure that we can translate them in the same order that they
    // were declared in.
    names: HashMap<~str, (@ast::Expr, Option<@ast::Expr>)>,
    name_types: HashMap<~str, ~[~str]>,
    name_ordering: ~[~str],

    // Collection of strings
    pieces: ~[~AsmPiece<'a>],
    name_positions: HashMap<~str, uint>,

    // Updated as arguments are consumed or methods are entered
    next_output: uint,
    next_input: uint,
    num_outputs: uint,
}

impl<'a> Context<'a> {
    fn trans_piece<'a>(&mut self, piece: fmt::parse::Piece<'a>) -> AsmPiece<'a> {
        use fmt::parse::{Argument, ArgumentNext, ArgumentIs, ArgumentNamed, CountImplied, FormatSpec, AlignUnknown};

        match piece {
            fmt::parse::String(s) => String(s),
            fmt::parse::Argument(Argument {
                position: pos,
                format: FormatSpec {
                    ty: ty,
                    align: AlignLeft,
                    fill: None, flags: 0, precision: CountImplied, width: CountImplied,
                },
                method: None,
            }) => {
                println!("{}", ty)
                match pos {
                    ArgumentNext => {
                        let types = &mut self.arg_types[self.next_input];
                        let pos = self.next_input;
                        self.next_input += 1;
                        let ty = ty.to_owned();
                        match types.position_elem(&ty) {
                            Some(idx) => {
                                Input(pos, idx)
                            }
                            None => {
                                let this = Input(pos, types.len());
                                types.push(ty);
                                this
                            }
                        }
                    },
                    ArgumentIs(n) => match self.arg_types[n].position_elem(&ty.to_owned()) {
                        Some(idx) => {
                            Input(n, idx)
                        }
                        None => {
                            self.arg_types[n].push(ty.to_owned());
                            Input(n, self.arg_types[n].len() - 1)
                        }
                    },
                    ArgumentNamed(name) => /*self.name_types[name]*/ {
                        // match self.name_types.mangle(&name) {
                        //     None => {
                        //         self.name_types.insert(name.to_owned(), ~[ty.to_owned()]);
                        //     }
                        //     Some(t) => {
                        //         // t.push(ty.to_owned());
                        //     }
                        // }
                        self.name_types.insert_or_update_with(
                            name.to_owned(),
                            ~[ty.to_owned()],
                            |k, v| { v.push(ty.to_owned()); });
                        InputNamed(name, ty)
                    }
                }
            }
            // fmt::parse::Argument(ref arg) => {}
            // fmt::parse::CurrentArgument => fail!("methods not impl")
            _ => fail!("methods not impl")
        }
    }


    fn format_pieces<'a>(&mut self, s: &'a InternedString, sp: Span) -> ~[AsmPiece<'a>] {
        let mut p = ~[];
        let asm_str = s.get();
        let mut parser = fmt::parse::Parser::new(asm_str);
        loop {
            let n = parser.next();
            match n {
                Some(piece) => {
                    if parser.errors.len() > 0 { break }
                    // verify_piece(cx, &piece);
                    // let piecen = cx.trans_piece(piece);
                    p.push(self.trans_piece(piece));
                    // self.pieces.push(~p);
                }
                None => break
            }
        }

        match parser.errors.shift() {
            Some(error) => {
                self.ecx.span_err(sp, "invalid format string: " + error);
                self.expr_f = Some(MacResult::dummy_expr(sp));
            }
            None => {}
        }
        p
    }

    fn to_expr_inline_asm(&self) -> ast::InlineAsm {
        let mut expr = ast::InlineAsm {
            asm: token::intern_and_get_ident(self.asm_str),
            asm_str_style: ast::CookedStr,
            clobbers: token::intern_and_get_ident(~""),
            inputs: ~[],
            outputs: ~[],
            volatile: false,
            alignstack: false,
            dialect: ast::AsmIntel
        };
        expr
    }

    fn to_expr(self, sp: Span) -> MacResult {
        // println!("{}", self.expr.outputs.len())
        match self.expr_f {
            Some(e) => e,
            None => MRExpr(@ast::Expr {
                id: ast::DUMMY_NODE_ID,
                node: ast::ExprInlineAsm(self.to_expr_inline_asm()),
                span: sp
            })
        }
    }
}

pub fn expand_asm_format(ecx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> MacResult {
    let mut p = parse::new_parser_from_tts(ecx.parse_sess(),
                                           ecx.cfg(),
                                           tts.to_owned());

    // if !tts.is_empty() {
    //     cx.span_fatal(sp, "asm_format takes no arguments");
    // }

    // let mut asm_pieces = ~[];
    // let mut asm = ~[]; // StrVector
    let mut asm_str_style = None;
    let mut cx = Context {
        ecx: ecx,
        args: ~[],
        arg_types: ~[],
        names: HashMap::new(),
        name_positions: HashMap::new(),
        name_types: HashMap::new(),
        name_ordering: ~[],
        next_input: 0,
        next_output: 0,
        num_outputs: 0,
        pieces: ~[],
        fmtsp: ~[],
        asm_str: ~"",
        expr: ast::InlineAsm {
            asm: token::intern_and_get_ident(~""),
            asm_str_style: ast::CookedStr,
            clobbers: token::intern_and_get_ident(~""),
            inputs: ~[],
            outputs: ~[],
            volatile: false,
            alignstack: false,
            dialect: ast::AsmIntel
        },
        expr_f: None
    };

    let mut clobs = ~[];
    let mut apieces = ~[];
    loop {
        match p.token {
                // while p.token != token::EOF &&
                //       p.token != token::COLON &&
                //       p.token != token::MOD_SEP {

                //     if clobs.len() != 0 {
                //         p.eat(&token::COMMA);
                //     }

                //     let (s, _str_style) = p.parse_str();
                //     let clob = format!("~\\{{}\\}", s);
                //     clobs.push(clob);
                // }

            token::IDENT(_, false) => {
                let option = token::get_ident(p.parse_ident());

                if option.equiv(&("volatile")) {
                    cx.expr.volatile = true;
                } else if option.equiv(&("alignstack")) {
                    cx.expr.alignstack = true;
                } else {//if option.is_some() {
                    clobs.push(option.get().to_owned());
                }
                // match token::get_ident(p.parse_ident()).get() {
                //     &"volatile" => println!("vol"),
                //     _ => ()
                // }
                p.expect(&token::COMMA);
            }
            _ => break
        }
    }

    cx.expr.clobbers = token::intern_and_get_ident(clobs.connect(","));

    let asm_expr = p.parse_expr();
    let (s, style) = match expr_to_str(cx.ecx, asm_expr,
                                       "inline assembly must be a string literal.") {
        Some((s, st)) => (s, st),
        // let compilation continue
        None => return MacResult::dummy_expr(sp),
    };
    // asm.push(s);
    cx.format_pieces(&s, asm_expr.span);

    asm_str_style = Some(style);
    loop {
        let (s, style) = match p.parse_optional_str() {
            Some((s_, st)) => (s_, st),
            None => break
        };
        apieces.push((s, p.span));
        // apieces.push(cx.format_pieces(&s, p.span));

        match asm_str_style {
            Some(st) => {
                if st != style {
                    // perhaps don't check
                    fail!("style")
                }
            },
            None => ()
        }

        // asm.push(s);
    }

    // for &(ref p, ref pspan) in apieces.iter() {
        // cx.format_pieces(p, *pspan);
    // }

    let mut named = false;
    while p.token != token::EOF {
        if !p.eat(&token::COMMA) {
            cx.ecx.span_err(sp, "expected token: `,`");
            return MacResult::dummy_expr(sp);
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
                    if p.token == token::RARROW {
                        p.bump();
                        let out = p.parse_expr();
                        // cx.names.insert(name.to_str(), (e, Some(out)));
                        cx.expr.outputs.push((s, out));
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
                            return MacResult::dummy_expr(sp);
                        }
                        _ => {
                            cx.ecx.span_err(p.span,
                                         format!("expected ident for named argument, but found `{}`",
                                                 p.this_token_to_str()));
                            return MacResult::dummy_expr(sp);
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
                    cx.name_ordering.push(name.to_str());
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

    // handle asm pieces
    for &(ref p, ref pspan) in apieces.iter() {
            // for &(ref p, ref pspan) in apieces.iter() {
        // cx.format_pieces(p, *pspan);
    // }
        for j in cx.format_pieces(p, *pspan).iter() {
        match *j {
            String(s) => cx.asm_str.push_str(s),
            Output(i) => cx.asm_str.push_str("$" + i.to_str()),
            Input(i, n) => cx.asm_str.push_str("$" + (i + cx.num_outputs).to_str()),
            InputNamed(n, ty) => cx.asm_str.push_str("$0")
        }
    }
    }
    println!("{} {}", cx.arg_types, cx.name_types);
    // MRExpr(quote_expr!(cx, 1i))
    cx.to_expr(sp)
}

fn verify_piece(cx: &mut Context, p: &fmt::parse::Piece) {
}

// fn trans_piece(piece: &fmt::parse::Piece) -> @ast::Expr {
    // MacResult::dummy_expr(sp)
// }
/*
MRExpr(@ast::Expr {
    id: ast::DUMMY_NODE_ID,
    node: ast::ExprInlineAsm(ast::InlineAsm {
        asm: token::intern_and_get_ident(asm.get()),
        asm_str_style: asm_str_style.unwrap(),
        clobbers: token::intern_and_get_ident(cons),
        inputs: inputs,
        outputs: outputs,
        volatile: volatile,
        alignstack: alignstack,
        dialect: dialect
    }),
    span: sp
})
*/