#![crate_id = "asm_format#0.11-pre"]
#![crate_type="dylib"]
#![feature(managed_boxes, globs, macro_registrar, macro_rules, quote)]
#![experimental]

extern crate syntax;
extern crate collections;
extern crate fmt_macros;

use syntax::ast;
use syntax::ext::base;
use syntax::parse;
use syntax::parse::token::InternedString;

use syntax::ast::{Name, TokenTree};
use syntax::codemap::Span;
use syntax::ext::base::*;
use syntax::parse::token;

use collections::HashMap;

use std::vec::Vec;
use std::strbuf::StrBuf;

#[macro_registrar]
pub fn macro_registrar(register: |Name, SyntaxExtension|) {
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
    Input(uint),
}

struct Context<'a, 'b> {
    asm_str: StrBuf,
    expr: ast::InlineAsm,
    dummy: Option<Box<MacResult>>,
    ecx: &'a mut ExtCtxt<'b>,

    // Parsed argument expressions and the types that we've found so far for
    // them.
    // TODO try to use HashMap<(uint, ~str), uint>
    args: Vec<(@ast::Expr, Option<@ast::Expr>, Option<uint>)>,
    // as used in the string. (args_idx, ty)
    arg_outputs: Vec<(uint, ~str)>,
    arg_inputs: Vec<(uint, ~str)>,

    // Names of parsed named expressions mapped to their index into args.
    names: HashMap<~str, uint>,

    // Updated as arguments are consumed or methods are entered
    next_argument: uint,
}

#[inline]
fn fetch_idx(arg: &mut Vec<(uint, ~str)>, key: (uint, ~str)) -> uint {
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
                method: None,
            }) => {
                let key = match pos {
                    ArgumentNext => {
                        let key = self.next_argument;
                        self.next_argument += 1;
                        key
                    }
                    ArgumentIs(n) => n,
                    ArgumentNamed(name) => match self.names.find(&name.to_owned()) {
                        Some(&n) => n,
                        None => {
                            self.ecx.span_err(sp, format!("there is no argument named `{}`",
                                                            name));
                            self.dummy = Some(DummyResult::expr(sp));
                            return String("");
                        }
                    }
                };

                match align {
                    AlignLeft => {
                        Output(fetch_idx(&mut self.arg_outputs, (key, ty.to_owned())))
                    }
                    AlignUnknown => {
                        Input(fetch_idx(&mut self.arg_inputs, (key, ty.to_owned())))
                    }
                    _ => fail!("invalid align")
                }
            }
            _ => fail!("methods not implemented")
        }
    }

    // fn verify_piece(&mut self, p: &AsmPiece) {
    //     match *p {
    //         fmt_macros::String(..) => {}
    //         fmt_macros::Argument(ref arg) => {
    //             // argument second, if it's an implicit positional parameter
    //             // it's written second, so it should come after width/precision.
    //             let pos = match arg.position {
    //                 fmt_macros::ArgumentNext => {
    //                     let i = self.next_arg;
    //                     self.next_arg += 1;
    //                     Exact(i)
    //                 }
    //                 fmt_macros::ArgumentIs(i) => Exact(i),
    //                 fmt_macros::ArgumentNamed(s) => Named(s.to_strbuf()),
    //             };

    //             self.verify_arg_type(pos);
    //         }
    //         fmt_macros::CurrentArgument => fail!("methods not impl"),
    //         _ => ()
    //     }
    // }

    // fn verify_arg_type(&mut self, arg: Position) {
    //     match arg {
    //         Exact(arg) => {
    //             if self.args.len() <= arg {
    //                 let msg = format!("invalid reference to argument `{}` (there \
    //                                 are {} arguments)", arg, self.args.len());
    //                 self.ecx.span_err(self.fmtsp, msg);
    //                 return;
    //             }
    //         }

    //         Named(name) => {
    //             let span = match self.names.find(&name) {
    //                 Some(e) => e.span,
    //                 None => {
    //                     let msg = format!("there is no argument named `{}`", name);
    //                     self.ecx.span_err(self.fmtsp, msg);
    //                     return;
    //                 }
    //             };
    //         }
    //     }
    // }

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
                self.ecx.span_err(sp, "invalid format string: " + error.as_slice());
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
        next_argument: 0,
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
                    clobs.push(option.get().to_owned());
                }

                p.expect(&token::COMMA);
            }
            _ => break
        }
    }

    let clobbers = format!("~\\{{}\\}", clobs.connect("},~{"));
    cx.expr.clobbers = token::intern_and_get_ident(clobbers);

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
                        cx.expr.outputs.push((token::intern_and_get_ident("=" + s.get()), out));
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
                        Some(&idx) => match cx.args.get(idx) {
                            &(prev, _, _) => {
                                cx.ecx.span_err(e.span,
                                                format!("duplicate argument named `{}`", name));
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
                        cx.expr.outputs.push((token::intern_and_get_ident("=" + *b), out));
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
                cx.expr.inputs.push((token::intern_and_get_ident(*b), out));
            }
            &(out, _, Some(idx)) if a < cx.args.len() => {
                cx.expr.inputs.push((token::intern_and_get_ident(idx.to_str()), out));
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
                cx.asm_str.push_str(i.to_str());
            }
        }
    }

    cx.into_expr(sp)
}
