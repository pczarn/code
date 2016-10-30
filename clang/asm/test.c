__attribute((noinline, naked)) int NakedTest(int value, int value2)
{
	asm("");
}

asm("BareTest:");

static __attribute__((used)) int var1;
int f1() {
	int out;
	// asm("mov var1, %0; mov 123, %1" : "=r" (out), "=g" (out));
	return out;
}

int f2(int p1, int p2) {
	int temp1, temp2;
	int out;

	/* '=&' so temp's don't overlap with inputs */
	asm ("mov %3, %1\n\t"
		"mov %4, %2\n\t"
		"shr $10, %1\n\t"
		"shl $10, %2\n\t"
		"add %3, %1 \n\t"
		"lea (%4, %2, 1), %0\n\t"
		"xor %1, %0\n\t"
		: "=r" (out), "=&r" (temp1), "=&r" (temp2): "r" (p1), "r" (p2) : "cc");

	return out;
}

int *func4a(int *p) {
	printf("in-out %d\n", *p);
	return p;
}

int func4(int parm) {
	int *p = &parm;
	asm ("add $0xff, %0; add %1, %0": "+r" (*func4a(p)) : "i"(123) : "cc");
	// asm ("add $0xff, %0; add %1, %0": "=r"(*func4a(p)) : "i"(123), "r"(*func4a(p)) : "cc");
	return parm;
}

// double in_out_early(double p1)
// {
// 	double out = 0.1;
// 	asm ("fadd %1\n\t"
// 		: "+&t" (out) : "f" (p1));
// 	return out;
// }

int in_out_early(int p1)
{
	int out = 7;
	asm (
		  "add %1, %0\n\t"
		  "add %10, %0 \n\t"
		  "add %2, %0\n\t"
		: "+&r"(out) : "r"(out), "r"(p1), "r"(out), "r"(out), "r"(out), "r"(out), "r"(out), "r"(out), "r"(out), "r"(out), "r"(out), "r"(out), "r"(out) : "cc");
	return out;
}

int main() {
	f1();
	func4(1234);
	in_out_early(0.2);
}