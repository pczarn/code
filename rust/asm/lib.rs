#[feature(phase, asm)];

#[phase(syntax)]
extern crate asm_ext;
// mod asm_ext;

fn main() {
	assert_eq!(2, exported_macro!());
	let mut c = ~0;
	let b = 123;
	// println!("{}", 1);
	unsafe {
		asm_format!(volatile, rax,
			"mov rax, {a:r};"
			"add rax, {a:i}", /*"{r0}" = 1 -> 2,*/ a = 123 -> c)
		asm!("nop")
		println!("{}", c)
	}
}