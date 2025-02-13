#include <stdio.h>
#include "quack.h"

// The threshold is the maximum number of elements that can be decoded.
#define THRESHOLD 10
#define BUFFER_SIZE 5

int main() {
    // Set the maximum threshold for lazy performance optimizations.
    quack_global_config_set_max_power_sum_threshold(THRESHOLD);

    // Insert some elements in the first quACK.
    PowerSumQuackU32* q1 = quack_new(THRESHOLD);
    quack_insert(q1, 1);
    quack_insert(q1, 2);
    quack_insert(q1, 3);
    quack_insert(q1, 4);
    quack_insert(q1, 5);

    // Insert a subset of the same elements in the second quACK.
    PowerSumQuackU32* q2 = quack_new(THRESHOLD);
    quack_insert(q2, 2);
    quack_insert(q2, 5);

    // Subtract the second quACK from the first and decode the elements.
    PowerSumQuackU32* q3 = quack_sub(q1, q2);

    const unsigned int log[] = {1, 2, 3, 4, 5};
    unsigned int result[BUFFER_SIZE];
    size_t len = quack_decode_with_log(q3, log, 5, result, BUFFER_SIZE);

    // Print the decoded result
    printf("Expected: 1 3 4\n");
    printf("Actual: ");
    for (size_t i = 0; i < len; i++) {
        printf("%u ", result[i]);
    }
    printf("\n");

    // Free memory
    // Note: q1 and q2 are consumed in the Rust function
    quack_free(q3);
    return 0;
}