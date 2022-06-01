#ifndef POWER_SUM_ACCUMULATOR_HPP_INCLUDED
#define POWER_SUM_ACCUMULATOR_HPP_INCLUDED

#include <array>   // for std::array
#include <cstddef> // for std::size_t

#include "ModularInteger.hpp" // for ModularInteger


template <typename T_NARROW, typename T_WIDE,
          T_NARROW MODULUS, std::size_t SIZE>
consteval std::array<ModularInteger<T_NARROW, T_WIDE, MODULUS>, SIZE>
modular_inverse_table() noexcept {
    using ModInt = ModularInteger<T_NARROW, T_WIDE, MODULUS>;
    std::array<ModInt, SIZE> result;
    for (std::size_t i = 0; i < SIZE; ++i) {
        result[i] = ModInt(i + 1).inv();
    }
    return result;
}


template <typename T_NARROW, typename T_WIDE,
          T_NARROW MODULUS, std::size_t SIZE>
class PowerSumAccumulator {

    static_assert(SIZE > 0, "Size of a PowerSumAccumulator must be nonzero");

    using ModInt = ModularInteger<T_NARROW, T_WIDE, MODULUS>;

    static constexpr std::array<ModInt, SIZE> inverse_table =
        modular_inverse_table<T_NARROW, T_WIDE, MODULUS, SIZE>();

    std::array<ModInt, SIZE> power_sums;

public:

    constexpr PowerSumAccumulator() noexcept {
        for (std::size_t i = 0; i < SIZE; ++i) {
            power_sums[i] = ModInt();
        }
    }

    constexpr PowerSumAccumulator(const PowerSumAccumulator &other) noexcept {
        for (std::size_t i = 0; i < SIZE; ++i) {
            power_sums[i] = other.power_sums[i];
        }
    }

    constexpr void insert(T_NARROW value) noexcept {
        const ModInt x{value};
        ModInt y = x;
        for (std::size_t i = 0; i < SIZE - 1; ++i) {
            power_sums[i] += y;
            y *= x;
        }
        power_sums[SIZE - 1] += y;
    }

    constexpr void clear() noexcept {
        for (std::size_t i = 0; i < SIZE; ++i) {
            power_sums[i] = ModInt();
        }
    }

    constexpr PowerSumAccumulator &operator-=(
        const PowerSumAccumulator &other
    ) noexcept {
        for (std::size_t i = 0; i < SIZE; ++i) {
            power_sums[i] -= other.power_sums[i];
        }
        return *this;
    }

    constexpr std::array<ModInt, SIZE>
    to_polynomial_coefficients() const noexcept {
        std::array<ModInt, SIZE> coeffs;
        coeffs[0] = -power_sums[0];
        for (std::size_t i = 1; i < SIZE; ++i) {
            for (std::size_t j = 0; j < i; ++j) {
                coeffs[i] -= power_sums[j] * coeffs[i - j - 1];
            }
            coeffs[i] -= power_sums[i];
            coeffs[i] *= inverse_table[i];
        }
        return coeffs;
    }

}; // class PowerSumAccumulator


#endif // POWER_SUM_ACCUMULATOR_HPP_INCLUDED
