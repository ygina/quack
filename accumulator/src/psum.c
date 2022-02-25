// psum.c
#include <stdio.h>
#include <stdlib.h>
#include <gmp.h>
#include <time.h>

#define PRINT_TRUE
#define PRINT_SOLN
#define PRINT_TIME

static struct timespec TIME;

static void start_time() {
    clock_gettime(CLOCK_MONOTONIC, &TIME);
}

static void print_time(const char *msg) {
    #ifdef PRINT_TIME
    {
        struct timespec end;
        clock_gettime(CLOCK_MONOTONIC, &end);
        struct timespec diff;
        if ((end.tv_nsec - TIME.tv_nsec) < 0) {
            diff.tv_sec = end.tv_sec - TIME.tv_sec - 1;
            diff.tv_nsec = 1000000000 + end.tv_nsec - TIME.tv_nsec;
        } else {
            diff.tv_sec = end.tv_sec - TIME.tv_sec;
            diff.tv_nsec = end.tv_nsec - TIME.tv_nsec;
        }
        printf("%s: %ld.%09ld\n", msg, diff.tv_sec, diff.tv_nsec);
    }
    #endif
}

// x^degree + coefficients[0]*x^{degree-1} + ... + coefficients[degree - 1]
void evaluate_monic_polynomial(
    mpz_t result, mpz_t temp,
    const mpz_t *coeffs, size_t degree, mpz_t x
) {
    mpz_set_ui(temp, 1);
    mpz_set_ui(result, 0);
    for (size_t i = degree; i --> 0;) {
        mpz_addmul(result, temp, coeffs[i]);
        mpz_mul(temp, temp, x);
    }
    mpz_add(result, result, temp); // monic leading term
}

// d*x^{d-1} + (d-1)*coefficients[0]*x^{d-2} + ... + coefficients[d - 2]
void evaluate_monic_polynomial_derivative(
    mpz_t result, mpz_t temp1, mpz_t temp2,
    const mpz_t *coeffs, size_t degree, mpz_t x
) {
    mpz_set_ui(temp1, 1);
    mpz_set_ui(result, 0);
    for (size_t i = degree - 1; i --> 0;) {
        mpz_mul_ui(temp2, coeffs[i], degree - i - 1);
        mpz_addmul(result, temp1, temp2);
        mpz_mul(temp1, temp1, x);
    }
    mpz_addmul_ui(result, temp1, degree); // monic leading term
}

// coeffs(x) = coeffs(x) / (x - r), discarding remainder
void divide_root_from_monic_polynomial(
    mpz_t *coeffs, size_t degree, mpz_t r
) {
    mpz_add(coeffs[0], coeffs[0], r);
    for (size_t i = 1; i < degree - 1; i++) {
        mpz_addmul(coeffs[i], r, coeffs[i - 1]);
    }
}

void find_integer_monic_polynomial_roots(
    mpz_t *roots, mpz_t *coeffs, size_t degree
) {
    mpz_t x, f, df, t, u;
    mpz_inits(x, f, df, t, u, NULL);
    while (degree > 1) {
        mpz_set_ui(x, rand());
        for (;;) {
            evaluate_monic_polynomial(f, t, coeffs, degree, x);
            if (!mpz_sgn(f)) {
                mpz_set(*roots++, x);
                divide_root_from_monic_polynomial(coeffs, degree--, x);
                break;
            }
            evaluate_monic_polynomial_derivative(df, t, u, coeffs, degree, x);
            mpz_fdiv_q_2exp(u, df, 1);
            mpz_add(u, u, f);
            mpz_fdiv_q(u, u, df);
            if (!mpz_sgn(u)) break;
            mpz_sub(x, x, u);
        }
    }
    mpz_neg(*roots++, coeffs[0]);
    mpz_clears(x, f, df, t, u, NULL);
}

void compute_polynomial_coefficients(
    mpz_t *coeffs, const mpz_t *power_sums, size_t n_values
) {
    mpz_set(coeffs[0], power_sums[0]);
    for (size_t i = 1; i < n_values; i++) {
        for (size_t j = 0; j < i; j++) {
            if (j % 2) {
                mpz_submul(coeffs[i], power_sums[j], coeffs[(i - 1) - j]);
            } else {
                mpz_addmul(coeffs[i], power_sums[j], coeffs[(i - 1) - j]);
            }
        }
        if (i % 2) {
            mpz_sub(coeffs[i], coeffs[i], power_sums[i]);
        } else {
            mpz_add(coeffs[i], coeffs[i], power_sums[i]);
        }
        mpz_fdiv_q_ui(coeffs[i], coeffs[i], i + 1);
    }
    for (size_t i = 0; i < n_values; i++) {
        if (!(i % 2)) mpz_neg(coeffs[i], coeffs[i]);
    }
}

void compute_polynomial_coefficients_wrapper(
    int64_t *coeffs, const int64_t *power_sums, size_t n_values
) {
    mpz_t *coeffs_mpz = malloc(n_values * sizeof(mpz_t));
    for (size_t i = 0; i < n_values; i++) mpz_init(coeffs_mpz[i]);
    mpz_t *power_sums_mpz = malloc(n_values * sizeof(mpz_t));
    for (size_t i = 0; i < n_values; i++) {
        mpz_init(power_sums_mpz[i]);
        mpz_set_ui(power_sums_mpz[i], power_sums[i]);
    };
    compute_polynomial_coefficients(coeffs_mpz, power_sums_mpz, n_values);
    for (size_t i = 0; i < n_values; i++) {
        coeffs[i] = mpz_get_ui(coeffs_mpz[i]);
    };
    for (size_t i = 0; i < n_values; i++) mpz_clear(power_sums_mpz[i]);
    free(power_sums_mpz);
    for (size_t i = 0; i < n_values; i++) mpz_clear(coeffs_mpz[i]);
    free(coeffs_mpz);
}
