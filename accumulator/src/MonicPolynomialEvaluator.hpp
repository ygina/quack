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

    MonicPolynomialEvaluator(std::size_t) {}

    constexpr ModInt eval(
        const std::vector<ModInt> &coeffs,
        T_NARROW x
    ) const noexcept {
        const std::size_t size = coeffs.size();
        const ModularInteger<T_NARROW, T_WIDE, MODULUS> x_mod(x);
        ModularInteger<T_NARROW, T_WIDE, MODULUS> result = x_mod;
        for (std::size_t i = 0; i < size - 1; ++i) {
            result += coeffs[i];
            result *= x_mod;
        }
        return result + coeffs[size - 1];
    }

}; // struct MonicPolynomialEvaluator


#define USE_LOOKUP_TABLE_16
#ifdef USE_LOOKUP_TABLE_16


template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
void power_table(
    std::vector<ModularInteger<T_NARROW, T_WIDE, MODULUS>> &dest,
    T_NARROW x, std::size_t size
) noexcept {
    std::vector<ModularInteger<T_NARROW, T_WIDE, MODULUS>> result(size);
    if (size > 0) {
        const ModularInteger<T_NARROW, T_WIDE, MODULUS> x_mod(x);
        if (size == 1) {
            result[0] = x_mod;
        } else {
            result[size - 2] = x_mod;
            for (std::size_t i = size - 2; i > 0; --i) {
                result[i - 1] = result[i] * x_mod;
            }
            result[size - 1] = result[0] * x_mod;
        }
    }
    for (const auto item : result) {
        dest.push_back(item);
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


template <std::uint16_t MODULUS>
struct MonicPolynomialEvaluator<std::uint16_t, std::uint32_t, MODULUS> {

    using ModInt = ModularInteger<std::uint16_t, std::uint32_t, MODULUS>;

    const std::vector<ModInt> power_tables;

    MonicPolynomialEvaluator(std::size_t size) :
        power_tables(power_tables_16<MODULUS>(size)) {}

    constexpr ModInt eval(
        const std::vector<ModInt> &coeffs,
        std::uint16_t x
    ) const noexcept {
        const std::size_t size = coeffs.size();
        const ModInt *power_table = power_tables.data() + size * x;
        std::uint64_t result = 0;
        for (std::size_t i = 0; i < size - 1; ++i) {
            result += static_cast<std::uint64_t>(coeffs[i].value) *
                      static_cast<std::uint64_t>(power_table[i].value);
        }
        result += static_cast<std::uint64_t>(coeffs[size - 1].value);
        result += static_cast<std::uint64_t>(power_table[size - 1].value);
        return ModInt(result % MODULUS);
    }

}; // struct MonicPolynomialEvaluator


#endif // USE_LOOKUP_TABLE_16


#endif // MONIC_POLYNOMIAL_EVALUATOR_HPP_INCLUDED
