#![crate_name = "macro_regex"]
#![crate_type="dylib"]
// #![feature(managed_boxes, globs, macro_registrar, macro_rules, quote)]
#![feature(globs, plugin_registrar, macro_rules, quote, managed_boxes)]
#![experimental]

extern crate syntax;
extern crate rustc;
extern crate debug;

use syntax::ast::{Name, TokenTree, Item, MetaItem, CrateConfig};
use syntax::codemap::Span;
use syntax::ext::base::*;
use syntax::parse::token;

use syntax::ast;
use syntax::ext::base;
use syntax::parse;
use syntax::parse::{parser, ParseSess};
use syntax::parse::parser::Parser;
use syntax::parse::token::{Token, Nonterminal, };
use syntax::parse::token::InternedString;
use syntax::parse::attr::ParserAttr;

use syntax::ast::{MatchTok, MatchSeq, MatchNonterminal};

use rustc::plugin::Registry;

use std::mem;
use std::gc::GC;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("macro_regex", add_new_extension);
}

fn add_new_extension(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree])
                   -> Box<MacResult> {
    // if !tts.is_empty() {
    //     cx.span_fatal(sp, "make_a_1 takes no arguments");
    // }
    // let arg_reader = new_tt_reader(&cx.parse_sess().span_diagnostic,
    //                                None,
    //                                arg.clone());
    let mut p = parse::new_parser_from_tts(cx.parse_sess(),
                                           cx.cfg(),
                                           tts.iter()
                                              .map(|x| (*x).clone())
                                              .collect());
    let mtch = p.parse_matchers();
    let prog = Program::new(mtch);
    println!("{}", prog.insts);
    // let tail = p.parse_seq_to_end(&token::EOF, parse::common::seq_sep_none(), |p| p.bump_and_get());
    // println!("{}", tail);
    // p.parse_all_token_trees().as_slice()
    println!("{}", run(&prog, &mut p, 0, prog.insts.len()));
    MacExpr::new(quote_expr!(cx, 1i))
}

// fn expand_into_foo(cx: &mut ExtCtxt, sp: Span, attr: @MetaItem, it: @Item)
//                    -> @Item {
//     @Item {
//         attrs: it.attrs.clone(),
//         ..(*quote_item!(cx, enum Foo { Bar, Baz }).unwrap()).clone()
//     }
// }

type InstIdx = uint;

#[deriving(Show, Clone)]
pub enum Inst {
    // When a Match instruction is executed, the current thread is successful.
    Match,

    OneTerminal(Token),

    // Matches a nonterminal.
    OneNonterminal(ast::Ident, ast::Ident, uint),

    // Saves the current position in the input string to the Nth save slot.
    Save(uint),

    // Jumps to the instruction at the index given.
    Jump(InstIdx),

    // Jumps to the instruction at the first index given. If that leads to
    // a failing state, then the instruction at the second index given is
    // tried.
    Split(InstIdx, InstIdx),
}

/// Program represents a compiled regular expression. Once an expression is
/// compiled, its representation is immutable and will never change.
///
/// All of the data in a compiled expression is wrapped in "MaybeStatic" or
/// "MaybeOwned" types so that a `Program` can be represented as static data.
/// (This makes it convenient and efficient for use with the `regex!` macro.)
#[deriving(Clone)]
pub struct Program {
    /// A sequence of instructions.
    pub insts: Vec<Inst>,
}

impl Program {
    /// Compiles a Regex given its AST.
    pub fn new(ast: Vec<ast::Matcher>) -> Program {
        let mut c = Compiler {
            insts: Vec::with_capacity(100),
            names: Vec::with_capacity(10),
        };

        c.insts.push(Save(0));
        for m in ast.move_iter() {
            c.compile(m);
        }
        c.insts.push(Save(1));
        c.insts.push(Match);

        let Compiler { insts, names } = c;
        let prog = Program {
            insts: insts,
        };
        prog
    }

    // Returns the total number of capture groups in the regular expression.
    // This includes the zeroth capture.
    // pub fn num_captures(&self) -> uint {
    //     let mut n = 0;
    //     for inst in self.insts.iter() {
    //         match *inst {
    //             Save(c) => n = cmp::max(n, c+1),
    //             _ => {}
    //         }
    //     }
    //     // There's exactly 2 Save slots for every capture.
    //     n / 2
    // }
}

struct Compiler<'r> {
    insts: Vec<Inst>,
    names: Vec<Option<String>>,
}

// The compiler implemented here is extremely simple. Most of the complexity
// in this crate is in the parser or the VM.
// The only tricky thing here is patching jump/split instructions to point to
// the right instruction.
impl<'r> Compiler<'r> {
    fn compile(&mut self, ast: ast::Matcher) {
        match ast.node {
            MatchTok(tok) => {
                self.push(OneTerminal(tok))
            }
            MatchSeq(seq, sep, true, lo, hi) => {
                let j1 = self.insts.len();
                let split = self.empty_split();
                let j2 = self.insts.len();
                for mtch in seq.move_iter() {
                    self.compile(mtch);
                }
                let jmp = self.empty_jump();
                let j3 = self.insts.len();

                self.set_jump(jmp, j1);
                // if g.is_greedy() {
                    self.set_split(split, j2, j3);
                // } else {
                //     self.set_split(split, j3, j2);
                // }
            }
            MatchSeq(seq, sep, false, lo, hi) => {
                let j1 = self.insts.len();
                for mtch in seq.move_iter() {
                    self.compile(mtch);
                }
                let split = self.empty_split();
                let j2 = self.insts.len();

                // if g.is_greedy() {
                    self.set_split(split, j1, j2);
                // } else {
                    // self.set_split(split, j2, j1);
                // }
            }
            MatchNonterminal(name, ty, pos) => {
                self.push(OneNonterminal(name, ty, pos));
            }
            // Nothing => {},
            // Literal(c, flags) => self.push(OneChar(c, flags)),
            // Dot(nl) => self.push(Any(nl)),
            // Class(ranges, flags) =>
            //     self.push(CharClass(ranges, flags)),
            // Begin(flags) => self.push(EmptyBegin(flags)),
            // End(flags) => self.push(EmptyEnd(flags)),
            // WordBoundary(flags) => self.push(EmptyWordBoundary(flags)),
            // Capture(cap, name, x) => {
            //     let len = self.names.len();
            //     if cap >= len {
            //         self.names.grow(10 + cap - len, &None)
            //     }
            //     *self.names.get_mut(cap) = name;

            //     self.push(Save(2 * cap));
            //     self.compile(*x);
            //     self.push(Save(2 * cap + 1));
            // }
            // Cat(xs) => {
            //     for x in xs.move_iter() {
            //         self.compile(x)
            //     }
            // }
            // Alt(x, y) => {
            //     let split = self.empty_split(); // push: split 0, 0
            //     let j1 = self.insts.len();
            //     self.compile(*x);                // push: insts for x
            //     let jmp = self.empty_jump();    // push: jmp 0
            //     let j2 = self.insts.len();
            //     self.compile(*y);                // push: insts for y
            //     let j3 = self.insts.len();

            //     self.set_split(split, j1, j2);  // split 0, 0 -> split j1, j2
            //     self.set_jump(jmp, j3);         // jmp 0      -> jmp j3
            // }
            // Rep(x, ZeroOne, g) => {
            //     let split = self.empty_split();
            //     let j1 = self.insts.len();
            //     self.compile(*x);
            //     let j2 = self.insts.len();

            //     if g.is_greedy() {
            //         self.set_split(split, j1, j2);
            //     } else {
            //         self.set_split(split, j2, j1);
            //     }
            // }
            // Rep(x, ZeroMore, g) => {
            //     let j1 = self.insts.len();
            //     let split = self.empty_split();
            //     let j2 = self.insts.len();
            //     self.compile(*x);
            //     let jmp = self.empty_jump();
            //     let j3 = self.insts.len();

            //     self.set_jump(jmp, j1);
            //     if g.is_greedy() {
            //         self.set_split(split, j2, j3);
            //     } else {
            //         self.set_split(split, j3, j2);
            //     }
            // }
            // Rep(x, OneMore, g) => {
            //     let j1 = self.insts.len();
            //     self.compile(*x);
            //     let split = self.empty_split();
            //     let j2 = self.insts.len();

            //     if g.is_greedy() {
            //         self.set_split(split, j1, j2);
            //     } else {
            //         self.set_split(split, j2, j1);
            //     }
            // }
        }
    }

    /// Appends the given instruction to the program.
    #[inline]
    fn push(&mut self, x: Inst) {
        self.insts.push(x)
    }

    /// Appends an *empty* `Split` instruction to the program and returns
    /// the index of that instruction. (The index can then be used to "patch"
    /// the actual locations of the split in later.)
    #[inline]
    fn empty_split(&mut self) -> InstIdx {
        self.insts.push(Split(0, 0));
        self.insts.len() - 1
    }

    /// Sets the left and right locations of a `Split` instruction at index
    /// `i` to `pc1` and `pc2`, respectively.
    /// If the instruction at index `i` isn't a `Split` instruction, then
    /// `fail!` is called.
    #[inline]
    fn set_split(&mut self, i: InstIdx, pc1: InstIdx, pc2: InstIdx) {
        let split = self.insts.get_mut(i);
        match *split {
            Split(_, _) => *split = Split(pc1, pc2),
            _ => fail!("BUG: Invalid split index."),
        }
    }

    /// Appends an *empty* `Jump` instruction to the program and returns the
    /// index of that instruction.
    #[inline]
    fn empty_jump(&mut self) -> InstIdx {
        self.insts.push(Jump(0));
        self.insts.len() - 1
    }

    /// Sets the location of a `Jump` instruction at index `i` to `pc`.
    /// If the instruction at index `i` isn't a `Jump` instruction, then
    /// `fail!` is called.
    #[inline]
    fn set_jump(&mut self, i: InstIdx, pc: InstIdx) {
        let jmp = self.insts.get_mut(i);
        match *jmp {
            Jump(_) => *jmp = Jump(pc),
            _ => fail!("BUG: Invalid jump index."),
        }
    }
}

//-------------------------
//-------------------------

pub fn run<'r, 't>(prog: &'r Program, input: &'t mut Parser<'t>,
                   start: uint, end: uint) -> bool {
    Nfa {
        prog: prog,
        // input: input,
        start: start,
        end: end,
        ic: 0,
        parser: input,
    }.run()
}

struct Nfa<'r, 't> {
    prog: &'r Program,
    // input: &'t str,
    start: uint,
    end: uint,
    ic: uint,
    parser: &'t mut Parser<'t>,
}

// /// Indicates the next action to take after a single non-empty instruction
// /// is processed.
// pub enum StepState {
//     /// This is returned if and only if a Match instruction is reached and
//     /// we only care about the existence of a match. It instructs the VM to
//     /// quit early.
//     StepMatchEarlyReturn,
//     /// Indicates that a match was found. Thus, the rest of the states in the
//     /// *current* queue should be dropped (i.e., leftmost-first semantics).
//     /// States in the "next" queue can still be processed.
//     StepMatch,
//     /// No match was found. Continue with the next state in the queue.
//     StepContinue,
// }

impl<'r, 't> Nfa<'r, 't> {
    fn run(&mut self) -> bool {
        let mut matched = false;
        let ninsts = self.prog.insts.len();
        let mut clist = &mut Threads::new(ninsts, 0);
        let mut nlist = &mut Threads::new(ninsts, 0);

        // let mut groups = Vec::from_elem(ncaps * 2, None);

        // self.ic = self.start;
        // let mut next_ic = self.chars.set(self.start);
        let mut next_ic = 1;
        while self.ic <= self.end {
            if clist.size == 0 {
                // We have a match and we're done exploring alternatives.
                // Time to quit.
                if matched {
                    break
                }

                // If there are no threads to try, then we'll have to start
                // over at the beginning of the regex.
                // BUT, if there's a literal prefix for the program, try to
                // jump ahead quickly. If it can't be found, then we can bail
                // out early.
                // if self.prog.prefix.len() > 0 && clist.size == 0 {
                //     let needle = self.prog.prefix.as_slice().as_bytes();
                //     let haystack = self.input.as_bytes().slice_from(self.ic);
                //     match find_prefix(needle, haystack) {
                //         None => break,
                //         Some(i) => {
                //             self.ic += i;
                //             next_ic = self.chars.set(self.ic);
                //         }
                //     }
                // }
            }

            // // This simulates a preceding '.*?' for every regex by adding
            // // a state starting at the current position in the input for the
            // // beginning of the program only if we don't already have a match.
            if clist.size == 0 || (!matched) {
                self.add(clist, 0)
            }

            // Now we try to read the next character.
            // As a result, the 'step' method will look at the previous
            // character.
            self.ic = next_ic;
            next_ic = self.ic + 1;

            for i in range(0, clist.size) {
                let (pc, popt) = clist.pc(i); // grab pc of i-th current state
                if self.step(nlist, pc, popt) {
                    matched = true;
                }
                // match step_state {
                //     StepMatchEarlyReturn => return vec![Some(0), Some(0)],
                //     StepMatch => { matched = true; break },
                //     StepContinue => {},
                // }
            }
            self.parser.bump();
            mem::swap(&mut clist, &mut nlist);
            nlist.empty();
        }
        matched
    }

    fn step(&mut self, nlist: &mut Threads<'t>,
            pc: uint, parser: &mut Option<Parser<'t>>)
           -> bool {
        // println!("{}", self.prog.insts.get(pc));
                    // println!("{:?}", parser);
        match *self.prog.insts.get(pc) {
            Match => {
                // match self.which {
                //     Exists => {
                //         return StepMatchEarlyReturn
                //     }
                //     Location => {
                //         groups[0] = caps[0];
                //         groups[1] = caps[1];
                //         return StepMatch
                //     }
                //     Submatches => {
                //         for (slot, val) in groups.mut_iter().zip(caps.iter()) {
                //             *slot = *val;
                //         }
                //         return StepMatch
                //     }
                // }
                return true
            }
            OneTerminal(ref tok) => {
                let is_match = {
                    let parser = match parser {
                        &Some(ref mut p) => p,
                        &None => &mut *self.parser
                    };
                    println!("{:?} {:?}", parser.token, tok);
                    parser.token == *tok
                };
                if is_match {
                    self.add(nlist, pc+1);
                }
            }
            OneNonterminal(_, ty, _) => {
                // let parser = parser.map_or(&*self.parser, |ref p| p);
                let tt_opt = {
                    let parser = match parser {
                        &Some(ref mut p) => p,
                        &None => &mut *self.parser
                    };
                    if parse_nt(parser, token::get_ident(ty).get()).is_some() {
                        Some((parser.sess, parser.cfg.clone(), parser.parse_all_token_trees()))
                    } else {
                        None
                    }
                };

                tt_opt.map(|(sess, cfg, ttv)| {
                    self.add_with_parser(nlist, pc+1, sess, cfg.clone(), ttv.as_slice());
                    let p = parse::new_parser_from_tts(sess,
                                                       cfg.clone(),
                                                       ttv);
                    let parser = match parser {
                        &Some(ref mut p) => p,
                        &None => &mut *self.parser
                    };
                    mem::replace(parser, p);
                });
            }
            _ => {}
        }
        false
    }

    fn add(&self, nlist: &mut Threads, pc: uint) {
        if nlist.contains(pc) {
            return
        }
        // We have to add states to the threads list even if their empty.
        // TL;DR - It prevents cycles.
        // If we didn't care about cycles, we'd *only* add threads that
        // correspond to non-jumping instructions (OneChar, Any, Match, etc.).
        // But, it's possible for valid regexs (like '(a*)*') to result in
        // a cycle in the instruction list. e.g., We'll keep chasing the Split
        // instructions forever.
        // So we add these instructions to our thread queue, but in the main
        // VM loop, we look for them but simply ignore them.
        // Adding them to the queue prevents them from being revisited so we
        // can avoid cycles (and the inevitable stack overflow).
        //
        // We make a minor optimization by indicating that the state is "empty"
        // so that its capture groups are not filled in.
        match *self.prog.insts.get(pc) {
            // EmptyBegin(flags) => {
            //     let multi = flags & FLAG_MULTI > 0;
            //     nlist.add(pc, groups, true);
            //     if self.chars.is_begin()
            //        || (multi && self.char_is(self.chars.prev, '\n')) {
            //         self.add(nlist, pc + 1, groups)
            //     }
            // }
            // EmptyEnd(flags) => {
            //     let multi = flags & FLAG_MULTI > 0;
            //     nlist.add(pc, groups, true);
            //     if self.chars.is_end()
            //        || (multi && self.char_is(self.chars.cur, '\n')) {
            //         self.add(nlist, pc + 1, groups)
            //     }
            // }
            // EmptyWordBoundary(flags) => {
            //     nlist.add(pc, groups, true);
            //     if self.chars.is_word_boundary() == !(flags & FLAG_NEGATED > 0) {
            //         self.add(nlist, pc + 1, groups)
            //     }
            // }
            Save(slot) => {
                nlist.add(pc, true);
                // match self.which {
                //     Location if slot <= 1 => {
                //         let old = groups[slot];
                //         groups[slot] = Some(self.ic);
                //         self.add(nlist, pc + 1, groups);
                //         groups[slot] = old;
                //     }
                //     Submatches => {
                //         let old = groups[slot];
                //         groups[slot] = Some(self.ic);
                //         self.add(nlist, pc + 1, groups);
                //         groups[slot] = old;
                //     }
                //     Exists | Location => 
                    self.add(nlist, pc + 1);
                // }
            }
            Jump(to) => {
                nlist.add(pc, true);
                self.add(nlist, to)
            }
            Split(x, y) => {
                nlist.add(pc, true);
                self.add(nlist, x);
                self.add(nlist, y);
            }
            _ => {
                nlist.add(pc, false);
            }
        }
    }

    fn add_with_parser<'a>(&self, nlist: &mut Threads<'a>, pc: uint, sess: &'a ParseSess, cfg: CrateConfig, tt: &[TokenTree]) {
        if nlist.contains(pc) {
            return
        }
        match *self.prog.insts.get(pc) {
            Save(slot) => {
                nlist.add_with_parser(pc, sess, cfg.clone(), tt);
                self.add_with_parser(nlist, pc + 1, sess, cfg, tt);
            }
            Jump(to) => {
                nlist.add_with_parser(pc, sess, cfg.clone(), tt);
                self.add_with_parser(nlist, to, sess, cfg, tt)
            }
            Split(x, y) => {
                nlist.add_with_parser(pc, sess, cfg.clone(), tt);
                self.add_with_parser(nlist, x, sess, cfg.clone(), tt);
                self.add_with_parser(nlist, y, sess, cfg, tt);
            }
            _ => {
                nlist.add_with_parser(pc, sess, cfg, tt);
            }
        }
    }
}

// #[inline]
fn parse_nt(parser: &mut Parser, name: &str) -> Option<Nonterminal> {
    match name {
        "item" => match parser.parse_item(Vec::new()) {
          Some(i) => Some(token::NtItem(i)),
          None => None
        },
        "block" => Some(token::NtBlock(parser.parse_block())),
        "stmt" => Some(token::NtStmt(parser.parse_stmt(Vec::new()))),
        "pat" => Some(token::NtPat(parser.parse_pat())),
        "expr" => {
            if parser.token != token::EOF {
                Some(token::NtExpr(parser.parse_expr()))
            } else {
                None
            }
        }
        "ty" => Some(token::NtTy(parser.parse_ty(false /* no need to disambiguate*/))),
        // this could be handled like a token, since it is one
        "ident" => match parser.token {
          token::IDENT(sn,b) => { parser.bump(); Some(token::NtIdent(box sn,b)) }
          _ => {
              // let token_str = token::to_str(&p.token);
              // p.fatal((format!("expected ident, found {}",
                               // token_str.as_slice())).as_slice())
                None
          }
        },
        "path" => {
          Some(token::NtPath(box parser.parse_path(parser::LifetimeAndTypesWithoutColons).path))
        }
        "meta" => Some(token::NtMeta(parser.parse_meta_item())),
        "tt" => {
          parser.quote_depth += 1u; //but in theory, non-quoted tts might be useful
          let res = token::NtTT(box(GC) parser.parse_token_tree());
          parser.quote_depth -= 1u;
          Some(res)
        }
        "matchers" => Some(token::NtMatchers(parser.parse_matchers())),
        // _ => p.fatal("unsupported builtin nonterminal parser: ".to_owned() + name)
        _ => None
    }
}

// /// CharReader is responsible for maintaining a "previous" and a "current"
// /// character. This one-character lookahead is necessary for assertions that
// /// look one character before or after the current position.
// pub struct CharReader<'t> {
//     /// The previous character read. It is None only when processing the first
//     /// character of the input.
//     pub prev: Option<char>,
//     /// The current character.
//     pub cur: Option<char>,
//     input: &'t str,
//     next: uint,
// }

// impl<'t> CharReader<'t> {
//     /// Returns a new CharReader that advances through the input given.
//     /// Note that a CharReader has no knowledge of the range in which to search
//     /// the input.
//     pub fn new(input: &'t str) -> CharReader<'t> {
//         CharReader {
//             prev: None,
//             cur: None,
//             input: input,
//             next: 0,
//        }
//     }

//     /// Sets the previous and current character given any arbitrary byte
//     /// index (at a unicode codepoint boundary).
//     #[inline]
//     pub fn set(&mut self, ic: uint) -> uint {
//         self.prev = None;
//         self.cur = None;
//         self.next = 0;

//         if self.input.len() == 0 {
//             return 1
//         }
//         if ic > 0 {
//             let i = cmp::min(ic, self.input.len());
//             let prev = self.input.char_range_at_reverse(i);
//             self.prev = Some(prev.ch);
//         }
//         if ic < self.input.len() {
//             let cur = self.input.char_range_at(ic);
//             self.cur = Some(cur.ch);
//             self.next = cur.next;
//             self.next
//         } else {
//             self.input.len() + 1
//         }
//     }

//     /// Does the same as `set`, except it always advances to the next
//     /// character in the input (and therefore does half as many UTF8 decodings).
//     #[inline]
//     pub fn advance(&mut self) -> uint {
//         self.prev = self.cur;
//         if self.next < self.input.len() {
//             let cur = self.input.char_range_at(self.next);
//             self.cur = Some(cur.ch);
//             self.next = cur.next;
//         } else {
//             self.cur = None;
//             self.next = self.input.len() + 1;
//         }
//         self.next
//     }

//     /// Returns true if and only if this is the beginning of the input
//     /// (ignoring the range of the input to search).
//     #[inline]
//     pub fn is_begin(&self) -> bool { self.prev.is_none() }

//     /// Returns true if and only if this is the end of the input
//     /// (ignoring the range of the input to search).
//     #[inline]
//     pub fn is_end(&self) -> bool { self.cur.is_none() }

//     /// Returns true if and only if the current position is a word boundary.
//     /// (Ignoring the range of the input to search.)
//     pub fn is_word_boundary(&self) -> bool {
//         if self.is_begin() {
//             return is_word(self.cur)
//         }
//         if self.is_end() {
//             return is_word(self.prev)
//         }
//         (is_word(self.cur) && !is_word(self.prev))
//         || (is_word(self.prev) && !is_word(self.cur))
//     }
// }

struct Thread<'t> {
    pc: uint,
    // groups: Vec<Option<uint>>,
    parser: Option<Parser<'t>>
}

struct Threads<'t> {
    // which: MatchKind,
    queue: Vec<Thread<'t>>,
    sparse: Vec<uint>,
    size: uint,
}

impl<'t> Threads<'t> {
    // This is using a wicked neat trick to provide constant time lookup
    // for threads in the queue using a sparse set. A queue of threads is
    // allocated once with maximal size when the VM initializes and is reused
    // throughout execution. That is, there should be zero allocation during
    // the execution of a VM.
    //
    // See http://research.swtch.com/sparse for the deets.
    fn new<'a>(num_insts: uint, ncaps: uint) -> Threads<'a> {
        Threads {
            // which: which,
            queue: Vec::from_fn(num_insts, |_| {
                Thread {
                    pc: 0,
                    // groups: Vec::from_elem(ncaps * 2, None)
                    parser: None
                }
            }),
            sparse: Vec::from_elem(num_insts, 0u),
            size: 0,
        }
    }

    fn add(&mut self, pc: uint, _empty: bool) {
        let t = self.queue.get_mut(self.size);
        t.pc = pc;
        // match (empty, self.which) {
        //     (_, Exists) | (true, _) => {},
        //     (false, Location) => {
        //         *t.groups.get_mut(0) = groups[0];
        //         *t.groups.get_mut(1) = groups[1];
        //     }
        //     (false, Submatches) => {
        //         for (slot, val) in t.groups.mut_iter().zip(groups.iter()) {
        //             *slot = *val;
        //         }
        //     }
        // }
        *self.sparse.get_mut(pc) = self.size;
        self.size += 1;
    }

    fn add_with_parser(&mut self, pc: uint, sess: &'t ParseSess, cfg: CrateConfig, tts: &[TokenTree]) {
        let t = self.queue.get_mut(self.size);
        t.pc = pc;
        let p = parse::new_parser_from_tts(sess,
                                           cfg,
                                           tts.iter()
                                              .map(|x| (*x).clone())
                                              .collect());
        t.parser = Some(p);
        *self.sparse.get_mut(pc) = self.size;
        self.size += 1;
    }

    #[inline]
    fn contains(&self, pc: uint) -> bool {
        let s = *self.sparse.get(pc);
        s < self.size && self.queue.get(s).pc == pc
    }

    #[inline]
    fn empty(&mut self) {
        self.size = 0;
    }

    #[inline]
    fn pc<'a>(&'a mut self, i: uint) -> (uint, &'a mut Option<Parser<'t>>) {
        let &Thread { pc, parser: ref mut popt } = self.queue.get_mut(i);
        (pc, popt)
    }

    // #[inline]
    // fn groups<'r>(&'r mut self, i: uint) -> &'r mut [Option<uint>] {
    //     self.queue.get_mut(i).groups.as_mut_slice()
    // }
}

// /// Returns true if the character is a word character, according to the
// /// (Unicode friendly) Perl character class '\w'.
// /// Note that this is only use for testing word boundaries. The actual '\w'
// /// is encoded as a CharClass instruction.
// pub fn is_word(c: Option<char>) -> bool {
//     let c = match c {
//         None => return false,
//         Some(c) => c,
//     };
//     // Try the common ASCII case before invoking binary search.
//     match c {
//         '_' | '0' .. '9' | 'a' .. 'z' | 'A' .. 'Z' => true,
//         _ => PERLW.bsearch(|&(start, end)| {
//             if c >= start && c <= end {
//                 Equal
//             } else if start > c {
//                 Greater
//             } else {
//                 Less
//             }
//         }).is_some()
//     }
// }

// /// Given a character and a single character class range, return an ordering
// /// indicating whether the character is less than the start of the range,
// /// in the range (inclusive) or greater than the end of the range.
// ///
// /// If `casei` is `true`, then this ordering is computed case insensitively.
// ///
// /// This function is meant to be used with a binary search.
// #[inline]
// fn class_cmp(casei: bool, mut textc: char,
//              (mut start, mut end): (char, char)) -> Ordering {
//     if casei {
//         // FIXME: This is pretty ridiculous. All of this case conversion
//         // can be moved outside this function:
//         // 1) textc should be uppercased outside the bsearch.
//         // 2) the character class itself should be uppercased either in the
//         //    parser or the compiler.
//         // FIXME: This is too simplistic for correct Unicode support.
//         //        See also: char_eq
//         textc = textc.to_uppercase();
//         start = start.to_uppercase();
//         end = end.to_uppercase();
//     }
//     if textc >= start && textc <= end {
//         Equal
//     } else if start > textc {
//         Greater
//     } else {
//         Less
//     }
// }

// /// Returns the starting location of `needle` in `haystack`.
// /// If `needle` is not in `haystack`, then `None` is returned.
// ///
// /// Note that this is using a naive substring algorithm.
// #[inline]
// pub fn find_prefix(needle: &[u8], haystack: &[u8]) -> Option<uint> {
//     let (hlen, nlen) = (haystack.len(), needle.len());
//     if nlen > hlen || nlen == 0 {
//         return None
//     }
//     for (offset, window) in haystack.windows(nlen).enumerate() {
//         if window == needle {
//             return Some(offset)
//         }
//     }
//     None
// }

// [Save(0), OneTerminal(IDENT(Ident { name: 58, ctxt: 20 }, false)), OneNonterminal(Ident { name: 60, ctxt: 20 }), Save(1), Match]
// [FAT_ARROW, IDENT(Ident { name: 59, ctxt: 20 }, false)]
// [Save(0), OneNonterminal(Ident { name: 60, ctxt: 21 }), OneTerminal(COLON), OneNonterminal(Ident { name: 63, ctxt: 21 }), OneTerminal(LBRACE), Split(6, 16), OneTerminal(POUND), OneTerminal(LBRACKET), OneNonterminal(Ident { name: 65, ctxt: 21 }), OneTerminal(RBRACKET), OneNonterminal(Ident { name: 60, ctxt: 21 }), Split(12, 15), OneTerminal(EQ), OneNonterminal(Ident { name: 68, ctxt: 21 }), Jump(11), Jump(5), OneTerminal(RBRACE), Save(1), Match]
// [DOLLAR, IDENT(Ident { name: 66, ctxt: 21 }, false)]

// [Save(0), OneTerminal(FAT_ARROW), Save(1), Match]
// false
