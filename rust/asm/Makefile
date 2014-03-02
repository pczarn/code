all: lib
	./lib

libasm_ext: asm_ext.rs
	/usr/local/bin/rustc --crate-type=dylib -O asm_ext.rs #-L /usr/lib/rustlib/x86_64-unknown-linux-gnu/lib/

lib: lib.rs libasm_ext
	/usr/local/bin/rustc lib.rs -L . #-L /usr/lib/rustlib/x86_64-unknown-linux-gnu/lib/