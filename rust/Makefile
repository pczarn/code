RUSTC ?= rustc-stage1

all: #inline_test macro_test
	#./inline_test
	#./macro_test
	$(RUSTC) simple.rs
	./simple

inline_test: inline_test.rs a
	$(RUSTC) -O inline_test.rs -L asm/

a: asm/lib.rs
	make -C asm/

macro_test: macro_test.rs macro_regex
	$(RUSTC) --emit=ir,link macro_test.rs -L macro_regex/

macro_regex: macro_regex/lib.rs
	make -C macro_regex/
	
