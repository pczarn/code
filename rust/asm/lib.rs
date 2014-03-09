#[feature(phase, asm)];

#[phase(syntax)]
extern crate asm_ext;

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
    }
}
