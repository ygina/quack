#ifndef MONIC_POLYNOMIAL_EVALUATOR_HPP_INCLUDED
#define MONIC_POLYNOMIAL_EVALUATOR_HPP_INCLUDED

#include <array>   // for std::array
#include <cstddef> // for std::size_t
#include <cstdint> // for std::uint*_t

#include "ModularInteger.hpp" // for ModularInteger


template <typename T_NARROW, typename T_WIDE,
          T_NARROW MODULUS, std::size_t SIZE>
constexpr std::size_t count_trailing_zeros(
    const std::array<ModularInteger<T_NARROW, T_WIDE, MODULUS>, SIZE> &coeffs
) noexcept {
    std::size_t result = 0;
    std::size_t i = SIZE - 1;
    while (i < SIZE && !coeffs[i--]) { ++result; }
    return result;
}


template <typename T_NARROW, typename T_WIDE,
          T_NARROW MODULUS, std::size_t SIZE>
consteval std::array<ModularInteger<T_NARROW, T_WIDE, MODULUS>, SIZE>
power_table(T_NARROW x) noexcept {
    std::array<ModularInteger<T_NARROW, T_WIDE, MODULUS>, SIZE> result;
    const ModularInteger<T_NARROW, T_WIDE, MODULUS> x_mod(x);
    result[SIZE - 2] = x_mod;
    for (std::size_t i = SIZE - 2; i > 0; --i) {
        result[i - 1] = result[i] * x_mod;
    }
    result[SIZE - 1] = result[0] * x_mod;
    return result;
}


template <std::uint16_t MODULUS, std::size_t SIZE>
consteval std::array<
    std::array<ModularInteger<std::uint16_t, std::uint32_t, MODULUS>, SIZE>,
    65536
> power_tables_16() noexcept {
    using ModInt = ModularInteger<std::uint16_t, std::uint32_t, MODULUS>;
    std::array<std::array<ModInt, SIZE>, 65536> result;
    for (std::size_t i = 0; i < 65536; ++i) {
        result[i] = power_table<std::uint16_t, std::uint32_t, MODULUS, SIZE>(i);
    }
    return result;
}


template <typename T_NARROW, typename T_WIDE,
          T_NARROW MODULUS, std::size_t SIZE>
struct MonicPolynomialEvaluator {

    using ModInt = ModularInteger<T_NARROW, T_WIDE, MODULUS>;

    static constexpr ModInt eval(
        const std::array<ModInt, SIZE> &coeffs,
        T_NARROW x
    ) noexcept {
        const ModularInteger<T_NARROW, T_WIDE, MODULUS> x_mod(x);
        ModularInteger<T_NARROW, T_WIDE, MODULUS> result = x_mod;
        for (std::size_t i = 0; i < SIZE - 1; ++i) {
            result += coeffs[i];
            result *= x_mod;
        }
        return result + coeffs[SIZE - 1];
    }

}; // struct MonicPolynomialEvaluator


template <std::uint16_t MODULUS, std::size_t SIZE>
struct MonicPolynomialEvaluator<std::uint16_t, std::uint32_t, MODULUS, SIZE> {

    using ModInt = ModularInteger<std::uint16_t, std::uint32_t, MODULUS>;

    static constexpr auto power_tables = power_tables_16<MODULUS, SIZE>();

    static constexpr ModInt eval(
        const std::array<ModInt, SIZE> &coeffs,
        std::uint16_t x
    ) noexcept {
        const std::array<ModInt, SIZE> power_table = power_tables[x];
        std::uint64_t result = 0;
        for (std::size_t i = 0; i < SIZE - 1; ++i) {
            result += static_cast<std::uint64_t>(coeffs[i].value) *
                      static_cast<std::uint64_t>(power_table[i].value);
        }
        result += static_cast<std::uint64_t>(coeffs[SIZE - 1].value);
        result += static_cast<std::uint64_t>(power_table[SIZE - 1].value);
        return ModInt(result % MODULUS);
    }

}; // struct MonicPolynomialEvaluator


#endif // MONIC_POLYNOMIAL_EVALUATOR_HPP_INCLUDED
