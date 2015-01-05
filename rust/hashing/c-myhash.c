#include "stddef.h"
#include "stdio.h"

void __attribute__ ((noinline))
print(char *state, size_t state_len) {
    for(size_t i = 0; i < state_len; i++) {
        printf("%d ", (int)state[i]);
    }
    printf("\n");
}

int main(int argc, char const *argv[])
{
    char val_slice[] = {12, 23, 34, 45, 12, 23, 34, 45};
    size_t val_slice_len = sizeof(val_slice);

    __asm__ volatile("" : "+r"(val_slice), "+m"(val_slice_len));

    char *val_ptr = &val_slice[0];

    char *state = val_ptr;
    size_t state_len = 1;

    for(int i = 0; i < val_slice_len; i++) {
        if(&state[state_len] == val_ptr) {
            state_len += 1;
        } else {
            // read
            state = val_ptr;
            state_len = 1;
        }   

        val_ptr++;
    }

    print(state, state_len);

    return 0;
}
