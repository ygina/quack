// psum.c
#include <stdlib.h>
#include <gmp.h>

void compute_polynomial_coefficients_wrapper(
    int64_t *coeffs, const int64_t *power_sums, size_t n_values
) {
    mpz_t *coeffs_mpz = malloc(n_values * sizeof(mpz_t));
    mpz_t *power_sums_mpz = malloc(n_values * sizeof(mpz_t));
    // TODO
    free(power_sums_mpz);
    free(coeffs_mpz);
}
