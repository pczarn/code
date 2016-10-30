#include "stddef.h"
#include "stdio.h"

void __attribute__ ((noinline))
print(char *state, size_t state_len) {
    for(size_t i = 0; i < state_len; i++) {
        printf("%d ", (int)state[i]);
    }
    printf("\n");
}

struct MySlice
{
    char *ptr;
    size_t len;
};

// MySlice __attribute__ ((noinline))
// merge_contiguous() {

// }

int main(int argc, char const *argv[])
{
    char val_slice[] = {12, 23, 34, 45, 12, 23, 34, 45};

    size_t val_slice_len = sizeof(val_slice);

    __asm__ volatile("" : "+m"(val_slice), "+m"(val_slice_len));

    char *val_ptr = &val_slice[0];

    char *state = val_ptr;
    size_t state_len = 0;

    while(val_slice_len--) {
        state_len += 1;
        val_ptr++;

        if(&state[state_len] != val_ptr) {
            // read.
            // continue
            state = val_ptr;
            state_len = 0;
        }   

    }

    print(state, state_len);

    return 0;
}
