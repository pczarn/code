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

int func4(int parm) {
	asm ("add $0xff, %0; add %1, %0": "+r" (parm) : "i"(123) : "cc");
	return parm;
}

int main() {
	f1();
}