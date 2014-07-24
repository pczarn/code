RUSTC := /usr/bin/rustc
# RUSTC := rustc-stage2
# RUSTC := /home/piotr/Desktop/src/rust/x86_64-unknown-linux-gnu/stage2/bin/rustc

all: lib

lib: lib.rs
	$(RUSTC) -Z verbose -O $^ --out-dir .
