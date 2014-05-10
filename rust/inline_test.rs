#![feature(phase, asm)]

#[phase(syntax)]
extern crate asm_format;

fn main() {
    let mut c = 0;
    let b = 13;

    unsafe {
        // TODO: {a:<r} should be written as {a=:r} or {a:=r}

        asm_format!(volatile, rax,
            "mov {a:r}, %rax;"
            "add %rbx, %rax;"
            "mov %rax, {a:<r}", a = 7 -> c, "{rbx}" = b)

        asm!("nop")
        println!("{}", c)

        // let v = ~[1u32, 2, 3, 4];
        // let src = v.as_ptr();
        // let dest = &mut [10u32, 11, 12, 13];
        // let n = 3;
        // let mut pd: *mut u8;
        // let mut ps: *u8;
        // asm!("rep movsb" : "={edi}"(pd), "={esi}"(ps) : "{edi}"(dest), "{esi}"(src), "{ecx}"(0))
        // asm!("rep movsl" : "={edi}"(pd), "={esi}"(ps) : "{edi}"(pd), "{esi}"(ps), "{ecx}"(n >> 2))
        // asm!("rep movsb" :: "{edi}"(pd), "{esi}"(ps), "{ecx}"(n % 4))

        let mut i = 1;
        // i = 1;
        // i += 1;
        // asm!("inc $0" : "={eax}"(i) : "{eax}"(123))
        asm!("inc $0" : "={eax}"(i) : "{eax}"(i) : "volatile")
            // {eax} = i
            // / (inc {eax})
            // i = {eax}
        // asm!("inc $0" :: "{eax}"(i))
        println!("{}", i);
    }
}
