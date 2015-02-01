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

struct VecTriArys<'a, H, K, V> {
    end: *mut TriAry<H, K, V>,
    ptr: *mut TriAry<H, K, V>,
    key: *mut TriAry<H, K, V>,
    val: *mut TriAry<H, K, V>,
    marker: marker::ContravariantLifetime<'a>,
    marker2: marker::NoCopy
}

impl<'a, H, K, V> Iterator<TriAryItems<'a, H, K, V>> for VecTriArys<'a, H, K, V> {
    fn next(&mut self) -> Option<TriAryItems<H, K, V>> {
        if self.ptr == self.end {
            None
        } else {
            unsafe {
                // align issue?
                let next_ptr = self.ptr.offset(1);
                let r = TriAryItems {
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

struct TriAryItems<'a, H, K, V> {
    ptr: *mut H,
    key: *mut K,
    val: *mut V,
    end: *mut V,
    marker: marker::ContravariantLifetime<'a>,
    marker2: marker::NoCopy
}

impl<'a, H, K, V> Iterator<(&'a mut H, &'a mut K, &'a mut V)> for TriAryItems<'a, H, K, V> {
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

struct VecTriAryItems<'a, H, K, V> {
    ptr: *mut H,
    key: *mut K,
    val: *mut V,
    seg_ptr: *mut TriAry<H, K, V>,
    seg_key: *mut TriAry<H, K, V>,
    seg_val: *mut TriAry<H, K, V>,
    segments_end: *mut TriAry<H, K, V>,
    marker: marker::ContravariantLifetime<'a>,
    marker2: marker::NoCopy
}

impl<'a, H, K, V> Iterator<(&'a mut H, &'a mut K, &'a mut V)> for VecTriAryItems<'a, H, K, V> {
    fn next(&mut self) -> Option<(&'a mut H, &'a mut K, &'a mut V)> {
        // #![inline]
        if self.val >= self.seg_ptr as *mut V { //or eq?
            // use 'likely' (expect)?
            if self.seg_ptr == self.segments_end {
                return None;
            } else {
                self.ptr = self.seg_ptr as *mut H;
                self.key = self.seg_key as *mut K;
                self.val = self.seg_val as *mut V;
                unsafe {
                    self.seg_ptr = self.seg_ptr.offset(1);
                    self.seg_key = self.seg_key.offset(1);
                    self.seg_val = self.seg_val.offset(1);
                }
            }
        }
        // cur val < next seg
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
//         let mut iter = VecTriArys {
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

fn mk_iter<'a, H, K, V>(v: &'a mut Vec<TriAry<H, K, V>>) -> VecTriArys<'a, H, K, V> {
    let r = unsafe {
        let ptr = v.as_mut_ptr();
        let len = v.len();
        let koff = (*ptr).mut1() as *mut [K, ..8] as uint - ptr as uint;
        let voff = (*ptr).mut2() as *mut [V, ..8] as uint - ptr as uint;
        VecTriArys {
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

fn mk_iter2<'a, H, K, V>(v: &'a mut Vec<TriAry<H, K, V>>) -> VecTriAryItems<'a, H, K, V> {
    unsafe {
        let ptr = v.as_mut_ptr();
        let next_ptr = ptr.offset(1);
        let len = v.len();
        let koff = (*ptr).mut1() as *mut [K, ..8] as uint - ptr as uint;
        let voff = (*ptr).mut2() as *mut [V, ..8] as uint - ptr as uint;
        let key = (ptr as *mut u8).offset(koff as int) as *mut TriAry<H, K, V>;
        let val = (ptr as *mut u8).offset(voff as int) as *mut TriAry<H, K, V>;
        VecTriAryItems {
            ptr: ptr as *mut H,
            key: key as *mut K,
            val: val as *mut V,
            seg_ptr: next_ptr,
            seg_key: key.offset(1),
            seg_val: val.offset(1),
            segments_end: ptr.offset(len as int),
            marker: marker::ContravariantLifetime,
            marker2: marker::NoCopy
        }
    }
}

fn mk_trivec() -> Vec<TriAry<u64, u64, u64>> {
    let mut v = Vec::new();
    // v.reserve_exact(32);
    for i in range(0, 64) {
        unsafe {
            // v.push(uninit());
            v.push(init());
        }
    }
    v
}

fn consume<'a, H, K, V>(i:VecTriArys<'a, H, K, V>) {}

#[bench]
fn fast_iter(b: &mut Bencher) {
    let mut v = mk_trivec();
    // let mut iterref = &mut iter;
    // unsafe {
        // asm!("" : "+r"(iterref));
    // }

    let mut n = 0;
    b.iter(|| {
        let mut iter = mk_iter2(&mut v);
        for (h, k, val) in iter {
            // println!("{:?}", x);
                *h += 1;
                *k = (*k + *val) | *val;
                // *k += 3;
                *val += 2;
                // println!("{:?}", iterref);
                // test::black_box(*k);
                // consume(iterref);
                // return;
                // iterref.next();
        }
        // println!("{:?}", v.get(0));
    });
    // println!("{:?}", v.get(0));
    test::black_box(&v);
    // println!("{:?}", n);
}

#[bench]
fn fast_iter_counter(b: &mut Bencher) {
    let mut v = mk_trivec();
    // let mut iterref = &mut iter;
    // unsafe {
        // asm!("" : "+r"(iterref));
    // }

    let size = 64*999;
    b.iter(|| {
        let mut n = 0;
        let mut iter = mk_iter2(&mut v);
        for (h, k, val) in iter {
            // println!("{:?}", x);
                *h += 1;
                *k = (*k + *val) | *val;
                // *k += 3;
                *val += 2;
                n += 1;
                if n > size {
                    return;
                }
        }
        // println!("{:?}", v.get(0));
    });
    // println!("{:?}", v.get(0));
    test::black_box(&v);
    // println!("{:?}", n);
}

#[bench]
fn fast_unwrap_iter_range(b: &mut Bencher) {
    let mut v = mk_trivec();
    // let mut iterref = &mut iter;
    // unsafe {
        // asm!("" : "+r"(iterref));
    // }

    let size = 64*999;
    b.iter(|| {
        let mut iter = mk_iter2(&mut v);
        for i in range(0, 63*8-3) {
            let (h, k, val) = iter.next().unwrap();
            // println!("{:?}", x);
                *h += i as u64;
                *k = (*k + *val) | *val;
                // *k += 3;
                *val += 2;
        }
        // println!("{:?}", v.get(0));
    });
    // println!("{:?}", v.get(0));
    test::black_box(&v);
    // println!("{:?}", n);
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
        {
            let mut iterref = mk_iter(&mut v);
        for mut x in iterref {
            // println!("{:?}", x);
            for (h, k, val) in x {
                *h += 1;
                *k = (*k + *val) | *val;
                // *k += 3;
                *val += 2;
                // println!("{:?}", iterref);
                // test::black_box(*k);
                // consume(iterref);
                // return;
                // iterref.next();
            }
            // if x.next() != None {
            //     fail!()
            // }
        }
            // assert!(iterref.next(), None);
        }
        // println!("{:?}", v.get(0));
    });
    // println!("{:?}", v.get(0));
    test::black_box(&v);
    // println!("{:?}", n);
}

#[bench]
fn nested_iter_counter(b: &mut Bencher) {
    let mut v = mk_trivec();
    // let mut iterref = &mut iter;
    // unsafe {
        // asm!("" : "+r"(iterref));
    // }
    let size = 64*8-3;
    b.iter(|| {
        let mut n = 0;
        let mut iterref = mk_iter(&mut v);
        for mut x in iterref {
            for (h, k, val) in x {
                *h += 1;
                *k = (*k + *val) | *val;
                // *k += 3;
                *val += 2;
                n += 1;
                if n > size {
                    return;
                }
            }
        }
    });
    test::black_box(&v);
}

#[bench]
fn nested_unwrap_iter(b: &mut Bencher) {
    let mut v = mk_trivec();
    // let mut iterref = &mut iter;
    // unsafe {
        // asm!("" : "+r"(iterref));
    // }

    let mut n = 0;
    b.iter(|| {
        {let mut iterref = mk_iter(&mut v);
        let mut triple = iterref.next().unwrap();
        loop {
            let (h, k, val) = unsafe {
                match triple.next() {
                    Some(t) => t,
                    None => {
                        match iterref.next() {
                            Some(iter) => {
                                triple = iter;
                                triple.next().unwrap()
                            }
                            None => break
                        }
                    }
                }
            };
            // println!("{:?}", x);
            // for (h, k, val) in x {
                *h += 1;
                *k = (*k + *val) | *val;
                // *k += 3;
                *val += 2;
                // println!("{:?}", iterref);
                // test::black_box(*k);
                // consume(iterref);
                // return;
                // iterref.next();
            // }
        }}
        // println!("{:?}", v.get(0));
    });
    // println!("{:?}", v.get(0));
    test::black_box(&v);
    // println!("{:?}", n);
}

#[bench]
fn nested_unwrap_iter_range(b: &mut Bencher) {
    let mut v = mk_trivec();
    b.iter(|| {
        {let mut iterref = mk_iter(&mut v);
        let mut triple = iterref.next().unwrap();
        for i in range(0, 64*8-3) {
            let (h, k, val) = unsafe {
                match triple.next() {
                    Some(t) => t,
                    None => {
                        match iterref.next() {
                            Some(iter) => {
                                triple = iter;
                                triple.next().unwrap()
                            }
                            None => break
                        }
                    }
                }
            };
                *h += i as u64;
                *k = (*k + *val) | *val;
                *val += 2;
        }}
    });
    test::black_box(&v);
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
            *k = (*k + *v) | *v;
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
            (v.as_mut_ptr() as *mut uint).offset(len as int),
            (v.as_mut_ptr() as *mut uint).offset((len * 2) as int),
        )
    };

    b.iter(|| {
        // let mut iter = mk_iter(&mut v);
        for i in range(0, len) {
            let i = i as int;
            unsafe {
                *h.offset(i) += 1;
                *k.offset(i) = (*k.offset(i) + *val.offset(i) as uint) | *val.offset(i);
                *val.offset(i) += 2;
            }
        }
    });
    // println!("{}", n);
    // println!("{:?}", v.get(0));
    test::black_box(&v);
}
