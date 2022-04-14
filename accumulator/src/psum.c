// psum.c
#include <pari/pari.h>

int32_t find_integer_monic_polynomial_roots_libpari(
    uint32_t *roots, const uint32_t *coeffs, long field, size_t degree
) {
    size_t i;
    uint32_t j, m;
    GEN vec, p, res, f;
    pari_init(1000000, 0);
    paristack_setsize(1000000, 100000000);

    // Initialize mod polynomial and factor
    vec = const_vecsmall(degree + 1, 0);
    for (i = 0; i < degree+1; i++) {
        vec[i+1] = coeffs[i];
    }
    p = gtopoly(vec, 0);
    res = factormod0(p, utoi(field), 0);

    // Copy results to roots vector
    int n = 0;
    for (i = 0; i < nbrows(res); i++) {
        f = gcoeff(res, i+1, 1);
        m = itou(gcoeff(res, i+1, 2));
        if (degpol(f) != 1) {
            // error: cannot be factored
            return -1;
        }
        // TODO: Masot added cast to shut gcc up
        for (j = 0; j < m; j++) {
            roots[n++] = field - itou((void*)constant_coeff(f)[2]);
        }
    }

    pari_close();
    return 0;
}
