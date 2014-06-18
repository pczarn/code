#![feature(phase, asm)]

#[phase(plugin)]
extern crate asm_format;

fn main() {
    let mut c = 0;
    let b = 13;

    unsafe {
        asm_format!(volatile, rax,
            "add {0:r}, {n:<r};
            add %rbx, {n:<r}", n = 7 -> c, "{rbx}" = b);

        asm!("nop")
        println!("{}", c)

        let mut i = 1;
        asm!("inc $0" : "={eax}"(i) : "{eax}"(i) :: "volatile")
        println!("{}", i);
    }
}
