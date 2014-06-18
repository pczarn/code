# RUSTC := /usr/bin/rustc
RUSTC ?= rustc-stage1
# RUSTC := /home/piotr/Desktop/src/rust/x86_64-unknown-linux-gnu/stage2/bin/rustc

all: lib

lib: lib.rs
	$(RUSTC) -O $^ --out-dir .
