#![feature(globs, macro_rules, default_type_params, asm, if_let, old_orphan_check)]

extern crate core;
extern crate test;

//use core::prelude::*;
use test::black_box;

use core::default::Default;

use std::hash::{Hash, Hasher, Writer};
use std::slice;
use std::mem;
use std::num::Int;

// /// `SipState` computes a SipHash 2-4 hash over a stream of bytes.
// struct SipState {
//     k0: u64,
//     k1: u64,
//     length: uint, // how many bytes we've processed
//     v0: u64,      // hash state
//     v1: u64,
//     v2: u64,
//     v3: u64,
//     tail: u64, // unprocessed bytes le
//     ntail: uint,  // how many bytes in tail are valid
// }

// impl Copy for SipState {}

// // sadly, these macro definitions can't appear later,
// // because they're needed in the following defs;
// // this design could be improved.

// macro_rules! u8to64_le {
//     ($buf:expr, $i:expr) =>
//     ($buf[0+$i] as u64 |
//      $buf[1+$i] as u64 << 8 |
//      $buf[2+$i] as u64 << 16 |
//      $buf[3+$i] as u64 << 24 |
//      $buf[4+$i] as u64 << 32 |
//      $buf[5+$i] as u64 << 40 |
//      $buf[6+$i] as u64 << 48 |
//      $buf[7+$i] as u64 << 56);
//     ($buf:expr, $i:expr, $len:expr) =>
//     ({
//         let mut t = 0;
//         let mut out = 0u64;
//         while t < $len {
//             out |= $buf[t+$i] as u64 << t*8;
//             t += 1;
//         }
//         out
//     });
// }

// macro_rules! rotl {
//     ($x:expr, $b:expr) =>
//     (($x << $b) | ($x >> (64 - $b)))
// }

// macro_rules! compress {
//     ($v0:expr, $v1:expr, $v2:expr, $v3:expr) =>
//     ({
//         $v0 += $v1; $v1 = rotl!($v1, 13); $v1 ^= $v0;
//         $v0 = rotl!($v0, 32);
//         $v2 += $v3; $v3 = rotl!($v3, 16); $v3 ^= $v2;
//         $v0 += $v3; $v3 = rotl!($v3, 21); $v3 ^= $v0;
//         $v2 += $v1; $v1 = rotl!($v1, 17); $v1 ^= $v2;
//         $v2 = rotl!($v2, 32);
//     })
// }

// impl SipState {
//     /// Creates a `SipState` that is keyed off the provided keys.
//     #[inline]
//     fn new() -> SipState {
//         SipState::new_with_keys(0, 0)
//     }

//     /// Creates a `SipState` that is keyed off the provided keys.
//     #[inline]
//     fn new_with_keys(key0: u64, key1: u64) -> SipState {
//         let mut state = SipState {
//             k0: key0,
//             k1: key1,
//             length: 0,
//             v0: 0,
//             v1: 0,
//             v2: 0,
//             v3: 0,
//             tail: 0,
//             ntail: 0,
//         };
//         state.reset();
//         state
//     }

//     /// Resets the state to its initial state.
//     #[inline]
//     fn reset(&mut self) {
//         self.length = 0;
//         self.v0 = self.k0 ^ 0x736f6d6570736575;
//         self.v1 = self.k1 ^ 0x646f72616e646f6d;
//         self.v2 = self.k0 ^ 0x6c7967656e657261;
//         self.v3 = self.k1 ^ 0x7465646279746573;
//         self.ntail = 0;
//     }

//     // /// Returns the computed hash.
//     // #[inline(always)]
//     // fn result(&self) -> u64 {
//     //     let mut v0 = self.v0;
//     //     let mut v1 = self.v1;
//     //     let mut v2 = self.v2;
//     //     let mut v3 = self.v3;

//     //     let b: u64 = ((self.length as u64 & 0xff) << 56) | self.tail;

//     //     v3 ^= b;
//     //     compress!(v0, v1, v2, v3);
//     //     compress!(v0, v1, v2, v3);
//     //     v0 ^= b;

//     //     v2 ^= 0xff;
//     //     compress!(v0, v1, v2, v3);
//     //     compress!(v0, v1, v2, v3);
//     //     compress!(v0, v1, v2, v3);
//     //     compress!(v0, v1, v2, v3);

//     //     v0 ^ v1 ^ v2 ^ v3
//     // }
// }

// impl Writer for SipState {
//     //#[inline]
//     fn write(&mut self, msg: &[u8]) {
//         // println!("{}", msg);
//         let length = msg.len();
//         self.length += length;

//         let mut needed = 0u;

//         if self.ntail != 0 {
//             needed = 8 - self.ntail;
//             if length < needed {
//                 self.tail |= u8to64_le!(msg, 0, length) << 8*self.ntail;
//                 self.ntail += length;
//                 return
//             }

//             let m = self.tail | u8to64_le!(msg, 0, needed) << 8*self.ntail;

//             self.v3 ^= m;
//             compress!(self.v0, self.v1, self.v2, self.v3);
//             compress!(self.v0, self.v1, self.v2, self.v3);
//             self.v0 ^= m;

//             self.ntail = 0;
//         }

//         // Buffered tail is now flushed, process new input.
//         let len = length - needed;
//         let end = len & (!0x7);
//         let left = len & 0x7;

//         let mut i = needed;
//         while i < end {
//             let mi = u8to64_le!(msg, i);

//             self.v3 ^= mi;
//             compress!(self.v0, self.v1, self.v2, self.v3);
//             compress!(self.v0, self.v1, self.v2, self.v3);
//             self.v0 ^= mi;

//             i += 8;
//         }

//         self.tail = u8to64_le!(msg, i, left);
//         self.ntail = left;
//     }
// }

// impl Clone for SipState {
//     #[inline]
//     fn clone(&self) -> SipState {
//         *self
//     }
// }

// impl Default for SipState {
//     #[inline]
//     fn default() -> SipState {
//         SipState::new()
//     }
// }

// /// `SipHasher` computes the SipHash algorithm from a stream of bytes.
// #[deriving(Clone)]
// #[allow(missing_copy_implementations)]
// struct SipHasher {
//     k0: u64,
//     k1: u64,
// }

// impl SipHasher {
//     /// Creates a `Sip`.
//     #[inline]
//     fn new() -> SipHasher {
//         SipHasher::new_with_keys(0, 0)
//     }

//     /// Creates a `Sip` that is keyed off the provided keys.
//     #[inline]
//     fn new_with_keys(key0: u64, key1: u64) -> SipHasher {
//         SipHasher {
//             k0: key0,
//             k1: key1,
//         }
//     }
// }

// // impl Hasher<SipState> for SipHasher {
// //     #[inline]
// //     fn hash<Sized? T: Hash<SipState>>(&self, value: &T) -> u64 {
// //         let mut state = SipState::new_with_keys(self.k0, self.k1);
// //         value.hash(&mut state);
// //         state.result()
// //     }
// // }

// // impl Default for SipHasher {
// //     #[inline]
// //     fn default() -> SipHasher {
// //         SipHasher::new()
// //     }
// // }

// // /// Hashes a value using the SipHash algorithm.
// // #[inline]
// // fn hash<Sized? T: Hash<SipState>>(value: &T) -> u64 {
// //     let mut state = SipState::new();
// //     value.hash(&mut state);
// //     state.result()
// // }

// /// Hashes a value using the SipHash algorithm.
// // #[inline]
// fn myhash<Sized? T: MyHash<SipState>>(value: &T) -> u64 {
//     let mut state = SipState::new();
//     // let mut sli = None;
//     let mut sli: &[u8] = &[];
//     let mut sli = Some(sli);
//     value.myhash(&mut state, &mut sli);

//     println!("{} {}", sli, state.length);

//     let mut b: u64 = if let Some(msg) = sli {
//         unsafe {
//             state.write(slice::from_raw_buf(mem::transmute(&msg.as_ptr()), msg.len() & !7));
//         }
//         let length = msg.len() & 7;
//         (((state.length + msg.len()) as u64 & 0xff) << 56) | u8to64_le!(msg, msg.len() & !7, length)
//     } else {
//         (state.length as u64 & 0xff) << 56
//     };

//     let mut v0 = state.v0;
//     let mut v1 = state.v1;
//     let mut v2 = state.v2;
//     let mut v3 = state.v3;

//     v3 ^= b;
//     compress!(v0, v1, v2, v3);
//     compress!(v0, v1, v2, v3);
//     v0 ^= b;

//     v2 ^= 0xff;
//     compress!(v0, v1, v2, v3);
//     compress!(v0, v1, v2, v3);
//     compress!(v0, v1, v2, v3);
//     compress!(v0, v1, v2, v3);

//     v0 ^ v1 ^ v2 ^ v3
// }

// // /// Hashes a value with the SipHash algorithm with the provided keys.
// // #[inline]
// // fn hash_with_keys<Sized? T: Hash<SipState>>(k0: u64, k1: u64, value: &T) -> u64 {
// //     let mut state = SipState::new_with_keys(k0, k1);
// //     value.hash(&mut state);
// //     state.result()
// // }

// struct CustomSlice<'a> { sli: &'a [u8] }

// impl<'a, S: Writer> Hash<S> for CustomSlice<'a> {
//     #[inline(always)]
//     fn hash(&self, state: &mut S) {
//         self.sli.len().hash(state);
//         for &v in self.sli.iter() {
//             state.write(&[v]);
//         }
//     }
// }

// trait MyHash<S = SipState> for Sized? {
//     fn myhash<'a>(&'a self, state: &mut S, sli: &mut Option<&'a [u8]>);
// }

// impl<'a, S: Writer> MyHash<S> for &'a [u8] {
//     #[inline(always)]
//     fn myhash<'b>(&'b self, state: &mut S, sli: &mut Option<&'b [u8]>) {
//         //if !sli.is_empty() { state.write(*sli); }
//         let mut sli2 = None;
//         let len2 = self.len();
//         len2.myhash(state, &mut sli2);
//         if let Some(sli2) = sli2 { state.write(sli2); }
//         for v in self.iter() {
//             v.myhash(state, sli);
//             //state.write(&[v]);
//         }
//     }
// }

// macro_rules! impl_hash {
//     ($ty:ident, $uty:ident) => {
//         impl<S: Writer> MyHash<S> for $ty {
//             #[inline]
//             fn myhash<'a>(&'a self, state: &mut S, sli: &mut Option<&'a [u8]>) {
//                 if (*self as $uty).to_le() == (*self as $uty) {
//                     let a = unsafe {
//                         slice::from_raw_buf(mem::transmute(&self), mem::size_of::<$uty>())
//                     };
//                     // let m = match sli {
//                     //     &Some(sli) => a.as_slice().as_ptr() == unsafe { sli.as_ptr().offset(sli.len() as int) },
//                     //     _ => false
//                     // };
//                     match sli { &Some(ref mut sli) if a.as_slice().as_ptr() == unsafe { sli.as_ptr().offset(sli.len() as int) } => {
//                         *sli = unsafe {
//                             slice::from_raw_buf(mem::transmute(&sli.as_ptr()), sli.len() + a.len())
//                         };
//                     } &Some(ref mut sl) => {
//                         state.write(*sl);
//                         *sl = a;
//                     } &None => {
//                         *sli = Some(a);
//                     } }
//                 } else {
//                     let a: [u8; ::core::$ty::BYTES] = unsafe {
//                         mem::transmute((*self as $uty).to_le() as $ty)
//                     };
//                     if let Some(sli) = *sli { state.write(sli); }
//                     state.write(a.as_slice());
//                     *sli = None;
//                 }
//             }
//         }
//     }
// }

// // fn myhash<'a>(&'a self, state: &mut S, msg: &mut Option<&'a [u8]>) {
// //     if (*self as T).to_le() == *self as T { unsafe {
// //         let a = slice::from_raw_buf(mem::transmute(&self), mem::size_of::<T>());
// //         let a_ptr = a.as_ptr();

// //         match msg {
// //             &Some(ref mut m) if a_ptr == m.as_ptr().offset(m.len() as int) => {
// //                 *msg = slice::from_raw_buf(mem::transmute(&m.as_ptr()),
// //                                            m.len() + a.len());
// //             }
// //             _ => {
// //                 if let Some(msg_part) = *msg { state.write(msg_part); }
// //                 *msg = Some(a);
// //             }
// //         }
// //     } } else {
// //         let a: [u8, ..::core::$ty::BYTES] = unsafe {
// //             mem::transmute((*self as T).to_le() as $ty)
// //         };
// //         if let Some(msg_part) = *msg { state.write(msg_part); }
// //         state.write(a.as_slice());
// //         *msg = None;
// //     }
// // }

// // for primitive integer types
// // impl<S: Writer> MyHash<S> for T
// // fn myhash<'a>(&'a self, state: &mut S, msg: &mut Option<&'a [u8]>) {
// //     if (*self as T).to_le() == *self as T {
// //         let a = unsafe {
// //             slice::from_raw_buf(mem::transmute(self), mem::size_of::<T>())
// //         };

// //         let msg_end = unsafe { msg.as_ptr().offset(sli.len() as int) };
// // let a_ptr = a.as_slice().as_ptr();

// //         match msg {
// //             &Some(ref mut msg_part) if a_ptr == msg_end => {
// //                 *msg = unsafe {
// //                     slice::from_raw_buf(mem::transmute(&msg_part.as_ptr()),
// //                                         msg_part.len() + a.len())
// //                 };
// //             }
// //             _ => {
// //                 if let Some(msg_part) = *msg { state.write(msg_part); }
// //                 *msg = Some(a);
// //             }
// //         }
// //     } else {
// //         let a: [u8; ::core::$ty::BYTES] = unsafe {
// //             mem::transmute((*self as T).to_le() as $ty)
// //         };
// //         if let Some(msg_part) = *msg { state.write(msg_part); }
// //         state.write(a.as_slice());
// //         *msg = None;
// //     }
// // }

// impl_hash!(u8, u8);
// impl_hash!(u16, u16);
// impl_hash!(u32, u32);
// impl_hash!(u64, u64);
// impl_hash!(uint, uint);
// impl_hash!(i8, u8);
// impl_hash!(i16, u16);
// impl_hash!(i32, u32);
// impl_hash!(i64, u64);
// impl_hash!(int, uint);

// struct IState<'a> {
//     r: Option<&'a uint>,
// }

// struct Inner<T> {
//     inner: T,
//     smth: uint,
// }

// impl<T> Inner<T> {
//     fn foo<'a>(&'a self, state: &mut IState<'a>) {
//         state.r = Some(&self.smth);
//     }
// }

    // // Test for slice join
    // let sli1 = val.sli.slice_to(2);
    // let sli2 = val.sli.slice_from(2);
    // let sli1end = unsafe { sli1.as_ptr().offset(sli1.len() as int) };
    // if sli1end == sli2.as_ptr() {
    //     unsafe { asm!("" :: "r"(valr) :: "volatile"); }
    // }
    // let mut ist = IState { r: None };
    // let inne = Inner { inner: 1u, smth: 2u };
    // inne.foo(&mut ist);
    
    // let h = myhash(valr);
    //let h = hash(&valr.sli[0]);
    // let mut h = hash(valr);

        // let m = match sli {
        //     &Some(sli) => a.as_slice().as_ptr() == unsafe { sli.as_ptr().offset(sli.len() as int) },
        //     _ => false
        // };

// fn main() {
//     let mut val_slice: &[u8] = &[12, 23, 34, 45];
//     // let mut val_raw = (val_slice.as_ptr(), val_slice.len());
//     let mut val = &mut val_slice;
//     unsafe { asm!("" : "+r"(val) ::: "volatile"); }
//     // let (val_ptr, val_len) = *val;

//     let mut state: Option<MySlice> = None;

//     let mut val_ptr = val.as_ptr();

//     // for v in val.slice_to(4).iter() { unsafe {
//         // let val1 = val.as_ptr();
//     for idx in range(0, 4) { unsafe {
//         // let next_sli = slice::from_raw_buf(mem::transmute(&v), 1);
//         // let val_ptr: *const u8 = val.as_ptr().offset(idx);

//         match state {
//             Some(ref mut current) => {
//                 // let current_end = current_sli.as_ptr()
//                 //                              .offset(current_sli.len() as int);

//                 if val_ptr == current.end {
//                     current.len += 1;
//                     current.end = current.end.offset(1);
//                     // *current = slice::from_raw_buf(mem::transmute(&current_sli.as_ptr()),
//                     //                                    current_sli.len() + next_sli.len());
//                 } else {
//                     black_box(&*current);
//                     *current = MySlice {
//                         start: val_ptr,
//                         len: 1,
//                         end: val_ptr.offset(1)
//                     };
//                 }
//             }
//             None => {
//                 state = Some(MySlice {
//                     start: val_ptr,
//                     len: 1,
//                     end: val_ptr.offset(1)
//                 });
//             }
//         }
//         val_ptr = val_ptr.offset(1);
//     } }

//     black_box(&state);
//     // unsafe { asm!("" :: "r"(&state) :: "volatile"); }
//     // println!("{}", state);
//     // unsafe { asm!("" :: "r"(&output) :: "volatile"); }

//     //let mut h = hash(valr);
//     //unsafe { asm!("" :: "r"(h) :: "volatile"); }
// }

// fn main() {
//     let mut val_slice: &[u8] = &[12, 23, 34, 45, 12, 23, 34, 45, 12, 23, 34, 45, 12, 23, 34, 45, 12, 23, 34, 45, 12, 23, 34, 45, 12, 23, 34, 45];
//     let mut val = &mut val_slice;
//     unsafe { asm!("" : "+r"(val) ::: "volatile"); }
 
//     let mut state: Option<MySlice> = None;
 
//     let mut val_ptr = val.as_ptr();
 
//     let res = val.iter().fold(state, |acc, elem| unsafe {
//         let val_ptr: *const u8 = elem;

//         match acc {
//             Some(mut current) => {
//                 if val_ptr == current.start.offset(current.len as int) {
//                     current.len += 1;
//                     Some(current)
//                     // current.end = current.end.offset(1);
//                 } else {
//                     // black_box(&current);
//                     Some(MySlice {
//                         start: val_ptr,
//                         len: 1,
//                         // end: val_ptr.offset(1)
//                     })
//                 }

//             }
//             None => {
//                 Some(MySlice {
//                     start: val_ptr,
//                     len: 1,
//                     // end: val_ptr.offset(1)
//                 })
//             }
//         }
//         // val_ptr = val_ptr.offset(1);
//         // r
//     });
 
//     black_box(&res);
//     println!("{}", res);
// }

#[deriving(Show)]
struct MySlice {
    start: *const u8,
    len: uint,
    end: *const u8,
}

#[inline(never)]
fn merge_slice(val: &[u8]) -> MySlice {
    let mut val_ptr = val.as_ptr();
 
    let mut len = val.len();

    // if len == 0 {
    //     return MySlice {
    //         start: val_ptr,
    //         len: 0,
    //         end: val_ptr,
    //     };
    // }

    // len -= 1;

    let mut state = unsafe {
        MySlice {
            start: val_ptr,
            len: 0,
            end: val_ptr // to the end of the 0 item
        }
    };
    
    // val_ptr = val_ptr.offset(1);

    unsafe {
        // let val_end = val_ptr.offset(val.len() as int);//val_ptr + val.len();//val_ptr.offset(val.len() as int);
        // let mut i = 0;
        // let mut val_ptr = val_ptr.offset(1);
        while len != 0 {
            // state = MySlice {
            //     start: val_ptr,
            //     len: 1,
            //     end: val_ptr.offset(1)
            // };

            // len -= 1;

            let mut flen = 0;
            let mut tmp1 = state.end;
            let mut tmp2 = val_ptr;
            loop {
                // state.end = tmp1;
                // val_ptr = tmp2;
                let tmp1 = tmp1.offset(1); // state advance to the end
                let tmp2 = tmp2.offset(1); // val ptr to the next item
                // if len == 0 {
                //     state.len += flen;
                //     state.end = tmp1;
                //     break;
                len -= 1;
                flen += 1;
                // tmp1 = tmp3;
                // tmp2 = tmp4;

                if tmp1 != tmp2 {
                    state.len += flen; // len to the end
                    state.end = tmp1;
                    val_ptr = tmp2;

                    state = MySlice {
                        start: val_ptr,
                        len: 0,
                        end: val_ptr,
                    };
                    break;
                }

                if len == 0 {
                    state.len += flen;
                    state.end = tmp1;
                    return state;
                }
            }

            // if tmp1 == tmp2 {
            //     state.end = tmp1;
            //     val_ptr = tmp2;
            // }

            // state.end = state.end.offset(flen as int);
            // val_ptr = val_ptr.offset(flen as int);
            // state.len += flen;
        }
    }

    state
}

fn main() {
    let mut val_slice: &[u8] = &[12, 23, 34, 45, 12];

    black_box(merge_slice(val_slice));

    // println!("{}", state);

    // let mut val = &mut val_slice;
    // unsafe { asm!("" : "+r"(val) ::: "volatile"); }
 

    // for _ in range(1, val.len()) { unsafe {
    // // for elem in val.slice_from(1).iter() { unsafe {
    //     // let val_ptr: *const u8 = elem;

    //     if val_ptr == state.end {
    //         state.len += 1;
    //         state.end = state.end.offset(1);
    //         val_ptr = val_ptr.offset(1);
    //     } else {
    //         // black_box(&*current);
    //         state = MySlice {
    //             start: val_ptr,
    //             len: 1,
    //             end: val_ptr.offset(1)
    //         };
    //         val_ptr = 
    //     }
    // } }
 
}
