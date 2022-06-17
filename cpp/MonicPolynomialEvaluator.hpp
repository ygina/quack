#ifndef MONIC_POLYNOMIAL_EVALUATOR_HPP_INCLUDED
#define MONIC_POLYNOMIAL_EVALUATOR_HPP_INCLUDED

#include <array>   // for std::array
#include <cstddef> // for std::size_t
#include <cstdint> // for std::uint*_t
#include <vector>  // for std::vector

#include "ModularInteger.hpp" // for ModularInteger


template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
constexpr std::size_t count_trailing_zeros(
    const std::vector<ModularInteger<T_NARROW, T_WIDE, MODULUS>> &coeffs
) noexcept {
    const std::size_t size = coeffs.size();
    std::size_t result = 0;
    std::size_t i = size - 1;
    while (i < size && !coeffs[i--]) { ++result; }
    return result;
}


template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
struct MonicPolynomialEvaluator {

    using ModInt = ModularInteger<T_NARROW, T_WIDE, MODULUS>;

    static constexpr ModInt eval(
        const std::vector<ModInt> &coeffs,
        T_NARROW x
    ) noexcept {
        const std::size_t size = coeffs.size();
        const ModularInteger<T_NARROW, T_WIDE, MODULUS> x_mod(x);
        ModularInteger<T_NARROW, T_WIDE, MODULUS> result = x_mod;
        for (std::size_t i = 0; i < size - 1; ++i) {
            result += coeffs[i];
            result *= x_mod;
        }
        return result + coeffs[size - 1];
    }

    static constexpr ModInt eval(
        const std::vector<ModInt> &power_tables,
        std::size_t table_size,
        const std::vector<ModInt> &coeffs,
        T_NARROW x
    ) noexcept {
        const std::size_t size = coeffs.size();
        const ModInt *power_table = power_tables.data() + (table_size * x);
        std::uint64_t result = static_cast<std::uint64_t>(power_table[size - 1].value);
        for (std::size_t i = 0; i < size - 1; ++i) {
            result += static_cast<std::uint64_t>(coeffs[i].value) *
                      static_cast<std::uint64_t>(power_table[size - i - 2].value);
        }
        result += static_cast<std::uint64_t>(coeffs[size - 1].value);
        return ModInt(result % MODULUS);
    }

}; // struct MonicPolynomialEvaluator


template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
void power_table(
    std::vector<ModularInteger<T_NARROW, T_WIDE, MODULUS>> &result,
    T_NARROW x, std::size_t size
) noexcept {
    if (size > 0) {
        // pre-compute x^1 to x^size
        const ModularInteger<T_NARROW, T_WIDE, MODULUS> x_mod(x);
        ModularInteger<T_NARROW, T_WIDE, MODULUS> acc = x_mod;
        result.push_back(x_mod);
        for (std::size_t i = 1; i < size; ++i) {
            acc *= x_mod;
            result.push_back(acc);
        }
    }
}


template <std::uint16_t MODULUS>
std::vector<ModularInteger<std::uint16_t, std::uint32_t, MODULUS>>
power_tables_16(std::size_t size) noexcept {
    using ModInt = ModularInteger<std::uint16_t, std::uint32_t, MODULUS>;
    std::vector<ModInt> result;
    for (std::size_t i = 0; i < 65536; ++i) {
        power_table<std::uint16_t, std::uint32_t, MODULUS>(result, i, size);
    }
    return result;
}


#endif // MONIC_POLYNOMIAL_EVALUATOR_HPP_INCLUDED
