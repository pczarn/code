#[feature(phase)];

#[phase(syntax)]
extern crate asm_ext;
// mod asm_ext;

fn main() {
	assert_eq!(2, exported_macro!());
	// println!("{}", 1);
	unsafe {
		asm_format!(volatile, "call {a:<rw}"" {a:<r}rax", "{r0}" = 1 -> 2, a = b -> c)
	}
}