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

#[macro_export]
#[macro_escape]
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

pub fn expand_asm_format(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> MacResult {
    let mut p = parse::new_parser_from_tts(cx.parse_sess(),
                                           cx.cfg(),
                                           tts.to_owned());

    // if !tts.is_empty() {
    //     cx.span_fatal(sp, "asm_format takes no arguments");
    // }

    let mut asm = ~[]; // StrVector
    let mut asm_str_style = None;

    loop {
        match p.token {
            token::IDENT(_, false) => {
                let option = token::get_ident(p.parse_ident());

                if option.equiv(&("volatile")) {
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

    let (s, style) = match expr_to_str(cx, p.parse_expr(),
                                       "inline assembly must be a string literal.") {
        Some((s, st)) => (s, st),
        // let compilation continue
        None => return MacResult::dummy_expr(sp),
    };
    asm.push(s);
    asm_str_style = Some(style);
    loop {
        let (s, style) = match p.parse_optional_str() {
            Some((s_, st)) => (s_, st),
            None => break
        };

        match asm_str_style {
            Some(st) => {
                if st != style {
                    fail!("style")
                }
            },
            None => ()
        }

        asm.push(s);
    }

    let mut args = ~[];
    let mut names = HashMap::<~str, (@ast::Expr, Option<@ast::Expr>)>::new();
    let mut order = ~[];
    let mut named = false;
    while p.token != token::EOF {
        if !p.eat(&token::COMMA) {
            cx.span_err(sp, "expected token: `,`");
            return MacResult::dummy_expr(sp);
        }
        if p.token == token::EOF { break } // accept trailing commas
        if named || (token::is_ident(&p.token) &&
                     p.look_ahead(1, |t| *t == token::EQ)) {
            named = true;
            let ident = match p.token {
                token::IDENT(i, _) => {
                    p.bump();
                    i
                }
                _ if named => {
                    cx.span_err(p.span,
                                 "expected ident, positional arguments \
                                 cannot follow named arguments");
                    return MacResult::dummy_expr(sp);
                }
                _ => {
                    cx.span_err(p.span,
                                 format!("expected ident for named argument, but found `{}`",
                                         p.this_token_to_str()));
                    return MacResult::dummy_expr(sp);
                }
            };
            let interned_name = token::get_ident(ident);
            let name = interned_name.get();
            p.expect(&token::EQ);
            let e = p.parse_expr();
            match names.find_equiv(&name) {
                None => {}
                Some(&(prev, _)) => {
                    cx.span_err(e.span, format!("duplicate argument named `{}`", name));
                    cx.parse_sess.span_diagnostic.span_note(prev.span, "previously here");
                    continue
                }
            }
            order.push(name.to_str());
            if p.token == token::RARROW {
                p.bump();
                let out = p.parse_expr();
                names.insert(name.to_str(), (e, Some(out)));
            }
            else {
                names.insert(name.to_str(), (e, None));
            }
        } else {
            let e = p.parse_expr();
            if p.token == token::RARROW {
                p.bump();
                let out = p.parse_expr();
                args.push((e, Some(out)));
            }
            else {
                args.push((e, None));
            }
        }
    }
    let mut pieces: ~[@ast::Expr] = ~[];
    let asm_str = asm.map(|i| i.get().to_owned()).concat();
    let mut parser = fmt::parse::Parser::new(asm_str);
    loop {
        match parser.next() {
            Some(piece) => {
                if parser.errors.len() > 0 { break }
                verify_piece(&piece);
                // let piece = trans_piece(&piece);
                // pieces.push(piece);
            }
            None => break
        }
    }
    match parser.errors.shift() {
        Some(error) => {
            cx.span_err(sp, "invalid format string: " + error);
            return MacResult::dummy_expr(sp);
        }
        None => {}
    }
    // MRExpr(quote_expr!(cx, 1i))
    MRExpr(@ast::Expr {
        id: ast::DUMMY_NODE_ID,
        node: ast::ExprInlineAsm(ast::InlineAsm {
            asm: token::intern_and_get_ident(asm.map(|i| i.get().to_owned()).concat()),
            asm_str_style: asm_str_style.unwrap(),
            clobbers: token::intern_and_get_ident(~""),
            inputs: ~[],
            outputs: ~[],
            volatile: false,
            alignstack: false,
            dialect: ast::AsmIntel
        }),
        span: sp
    })
}

fn verify_piece(p: &fmt::parse::Piece) {
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