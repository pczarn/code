all:
	# g++ hello.cpp -o hello -D
	make -C clang/asm/
	make -C rust/asm/
