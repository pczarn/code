all: test.s test.ll test

test: test.c
	clang $^ -o $@

test.s: test.c
	clang -mllvm --x86-asm-syntax=intel -S $^ -o $@

test.ll: test.c
	clang -S -emit-llvm $^ -o $@
