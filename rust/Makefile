RUSTC := rustc-stage2

all: inline_test
	./inline_test
	# stage1-rustc simple.rs
	# ./simple

inline_test: inline_test.rs
	make -C asm/
	$(RUSTC) --emit=ir,link inline_test.rs -L asm/
