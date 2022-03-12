// psum.c
#include <pari/pari.h>

int32_t find_integer_monic_polynomial_roots_libpari(
    int64_t *roots, const int64_t *coeffs, long field, size_t degree
) {
    size_t i;
    GEN vec, p, res, f;
    pari_init(1000000, 0);
    paristack_setsize(1000000, 100000000);

    // Initialize mod polynomial and factor
    vec = const_vecsmall(degree + 1, 0);
    for (i = 0; i < degree+1; i++) {
        vec[i+1] = coeffs[i];
    }
    p = gtopoly(vec, 0);
    res = factormod0(p, stoi(field), 0);

    // Copy results to roots vector
    for (i = 0; i < degree; i++) {
        f = gcoeff(res, i+1, 1);
        if (degpol(f) != 1) {
            // error: cannot be factored
            return -1;
        }
        roots[i] = field - itos(constant_coeff(f)[2]);
    }

    pari_close();
    return 0;
}
