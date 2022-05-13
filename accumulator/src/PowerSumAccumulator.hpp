#ifndef POWER_SUM_ACCUMULATOR_HPP
#define POWER_SUM_ACCUMULATOR_HPP

#include <array>   // for std::array
#include <cstddef> // for std::size_t


template <typename T_NARROW, typename T_WIDE,
          T_NARROW MODULUS, std::size_t SIZE>
class PowerSumAccumulator {

    static_assert(SIZE > 0);

    std::array<ModularInteger<T_NARROW, T_WIDE, MODULUS>, SIZE> power_sums;

public:

    constexpr PowerSumAccumulator() noexcept {
        for (std::size_t i = 0; i < SIZE; ++i) {
            power_sums[i] = ModularInteger<T_NARROW, T_WIDE, MODULUS>();
        }
    }

    constexpr PowerSumAccumulator(const PowerSumAccumulator &other) noexcept {
        for (std::size_t i = 0; i < SIZE; ++i) {
            power_sums[i] = other.power_sums[i];
        }
    }

    constexpr void insert(T_NARROW value) noexcept {
        const ModularInteger<T_NARROW, T_WIDE, MODULUS> x{value};
        ModularInteger<T_NARROW, T_WIDE, MODULUS> y = x;
        for (std::size_t i = 0; i < SIZE - 1; ++i) {
            power_sums[i] += y;
            y *= x;
        }
        power_sums[SIZE - 1] += y;
    }

    constexpr void clear() noexcept {
        for (std::size_t i = 0; i < SIZE; ++i) {
            power_sums[i] = ModularInteger<T_NARROW, T_WIDE, MODULUS>();
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

    constexpr std::array<ModularInteger<T_NARROW, T_WIDE, MODULUS>, SIZE>
    to_polynomial_coefficients() const noexcept {
        std::array<ModularInteger<T_NARROW, T_WIDE, MODULUS>, SIZE> coeffs;
        coeffs[0] = -power_sums[0];
        for (std::size_t i = 1; i < SIZE; ++i) {
            for (std::size_t j = 0; j < i; ++j) {
                coeffs[i] -= power_sums[j] * coeffs[i - j - 1];
            }
            coeffs[i] -= power_sums[i];
            coeffs[i] *= ModularInteger<T_NARROW, T_WIDE, MODULUS>(
                static_cast<T_NARROW>(i + 1)
            ).inv();
        }
        return coeffs;
    }

}; // class PowerSumAccumulator


template <typename T_NARROW, typename T_WIDE,
          T_NARROW MODULUS, std::size_t SIZE>
constexpr std::size_t count_trailing_zeros(
    const std::array<ModularInteger<T_NARROW, T_WIDE, MODULUS>, SIZE> &coeffs
) noexcept {
    std::size_t result = 0;
    std::size_t i = SIZE - 1;
    while (i < SIZE && !coeffs[i]) {
        ++result;
        --i;
    }
    return result;
}


template <typename T_NARROW, typename T_WIDE,
          T_NARROW MODULUS, std::size_t SIZE>
constexpr ModularInteger<T_NARROW, T_WIDE, MODULUS>
evaluate_monic_polynomial(
    const std::array<ModularInteger<T_NARROW, T_WIDE, MODULUS>, SIZE> &coeffs,
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


#endif // POWER_SUM_ACCUMULATOR_HPP
