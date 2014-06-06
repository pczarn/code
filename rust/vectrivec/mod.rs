#![feature(asm)]
extern crate debug;
extern crate test;

use std::vec::Vec;
use std::mem::uninit;
use std::mem::init;
use std::kinds::marker;

use test::Bencher;

static SIZE: uint = 8;
static MASK: uint = SIZE - 1;

type TriAry<H, K, V> = ([H, ..SIZE], [K, ..SIZE], [V, ..SIZE]);

struct TriAryIter<'a, H, K, V> {
    end: *mut TriAry<H, K, V>,
    ptr: *mut TriAry<H, K, V>,
    key: *mut TriAry<H, K, V>,
    val: *mut TriAry<H, K, V>,
    marker: marker::ContravariantLifetime<'a>,
    marker2: marker::NoCopy
}

impl<'a, H, K, V> Iterator<AryIter<'a, H, K, V>> for TriAryIter<'a, H, K, V> {
    fn next(&mut self) -> Option<AryIter<H, K, V>> {
        if self.ptr == self.end {
            None
        } else {
            unsafe {
                // align issue?
                let next_ptr = self.ptr.offset(1);
                let r = AryIter {
                    ptr: self.ptr as *mut H,
                    key: self.key as *mut K,
                    val: self.val as *mut V,
                    end: next_ptr as *mut V,
                    marker: marker::ContravariantLifetime,
                    marker2: marker::NoCopy
                };
                self.ptr = next_ptr;
                self.key = self.key.offset(1);
                self.val = self.val.offset(1);
                Some(r)
            }
        }
    }
}

struct AryIter<'a, H, K, V> {
    ptr: *mut H,
    key: *mut K,
    val: *mut V,
    end: *mut V,
    marker: marker::ContravariantLifetime<'a>,
    marker2: marker::NoCopy
}

impl<'a, H, K, V> Iterator<(&'a mut H, &'a mut K, &'a mut V)> for AryIter<'a, H, K, V> {
    fn next(&mut self) -> Option<(&'a mut H, &'a mut K, &'a mut V)> {
        if self.val >= self.end { //or eq
            None
        } else {
            unsafe {
                // align issue?
                let r = (
                    &mut *self.ptr,
                    &mut *self.key,
                    &mut *self.val,
                );
                self.ptr = self.ptr.offset(1);
                self.key = self.key.offset(1);
                self.val = self.val.offset(1);
                Some(r)
            }
        }
    }
}

// impl<H, K, V> Vec<TriAry<H, K, V>> {
//     fn iter_vecs(&self) {

//     }
// }

// fn main() {
//     let mut v: Vec<TriAry<u64, uint, uint>> = Vec::new();
//     // v.reserve_exact(32);
//     for i in range(0, 32) {
//         unsafe {
//             v.push(uninit());
//         }
//     }
//     unsafe {
//         let ptr = v.as_mut_ptr();
//         let len = v.len();
//         let koff = (*ptr).mut1() as *mut [uint, ..8] as uint - ptr as uint;
//         let voff = (*ptr).mut2() as *mut [uint, ..8] as uint - ptr as uint;
//         let mut iter = TriAryIter {
//             end: ptr.offset(len as int),
//             ptr: ptr,
//             key: (ptr as *mut u8).offset(koff as int) as *mut TriAry<u64, uint, uint>,
//             val: (ptr as *mut u8).offset(voff as int) as *mut TriAry<u64, uint, uint>,
//         };

//         for _ in range(0, 40000000) {
//             for mut x in iter {
//                 for (h, k, v) in x {
//                     *h += 1;
//                     *k += *h as uint;
//                     // println!("{:?}", h);
//                 }
//             }
//         }
//     }
// }

fn mk_iter<'a, H, K, V>(v: &'a mut Vec<TriAry<H, K, V>>) -> TriAryIter<'a, H, K, V> {
    let r = unsafe {
        let ptr = v.as_mut_ptr();
        let len = v.len();
        let koff = (*ptr).mut1() as *mut [K, ..8] as uint - ptr as uint;
        let voff = (*ptr).mut2() as *mut [V, ..8] as uint - ptr as uint;
        TriAryIter {
            end: ptr.offset(len as int),
            ptr: ptr,
            key: (ptr as *mut u8).offset(koff as int) as *mut TriAry<H, K, V>,
            val: (ptr as *mut u8).offset(voff as int) as *mut TriAry<H, K, V>,
            marker: marker::ContravariantLifetime,
            marker2: marker::NoCopy
        }
    };
    r
}

fn mk_trivec() -> Vec<TriAry<u64, u16, u8>> {
    // let mut v: Vec<TriAry<u64, u8, u8>> = Vec::new();
    let mut v = Vec::new();
    // v.reserve_exact(32);
    for i in range(0, 128) {
        unsafe {
            // v.push(uninit());
            v.push(init());
        }
    }
    v
}

#[bench]
fn nested_iter(b: &mut Bencher) {
    let mut v = mk_trivec();
    // let mut iterref = &mut iter;
    // unsafe {
        // asm!("" : "+r"(iterref));
    // }

    let mut n = 0;
    b.iter(|| {
        {let mut iterref = mk_iter(&mut v);
        for mut x in iterref {
            // println!("{:?}", x);
            for (h, k, val) in x {
                *h += 1;
                *k = (*k + (*val as u16)) | (*val as u16);
                // *k += 3;
                *val += 2;
                // test::black_box(*k);
            }
        }}
        // println!("{:?}", v.get(0));
    });
    // println!("{:?}", v.get(0));
    test::black_box(&v);
    // println!("{:?}", n);
}

#[bench]
fn flatmap_iter(b: &mut Bencher) {
    // let (mut v, mut iter) = mk_trivec();
    let mut v = mk_trivec();

    let mut n = 0;
    b.iter(|| {
        let mut iter = mk_iter(&mut v);
        for (h, k, v) in iter.flat_map(|i| i) {
            *h += 1;
            *k = (*k + *v as u16) | *v as u16;
            *v += 2;
            // test::black_box(*k);
        }
    });
    // println!("{}", n);
    // println!("{:?}", v.get(0));
    test::black_box(&v);
}

#[bench]
fn trivec_iter(b: &mut Bencher) {
    // let (mut v, mut iter) = mk_trivec();
    let mut v = mk_trivec();
    let len = v.len() * 8;
    let (h, k, val) = unsafe {
        (
            v.as_mut_ptr() as *mut u64,
            (v.as_mut_ptr() as *mut u16).offset(len as int),
            (v.as_mut_ptr() as *mut u8).offset((len * 2) as int),
        )
    };

    b.iter(|| {
        // let mut iter = mk_iter(&mut v);
        for i in range(0, len) {
            let i = i as int;
            unsafe {
                *h.offset(i) += 1;
                *k.offset(i) = (*k.offset(i) + (*val.offset(i) as u16)) | (*val.offset(i) as u16);
                *val.offset(i) += 2;
            }
        }
    });
    // println!("{}", n);
    // println!("{:?}", v.get(0));
    test::black_box(&v);
}
