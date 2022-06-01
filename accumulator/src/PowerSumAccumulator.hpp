#ifndef POWER_SUM_ACCUMULATOR_HPP_INCLUDED
#define POWER_SUM_ACCUMULATOR_HPP_INCLUDED

#include <cstddef> // for std::size_t
#include <vector>  // for std::vector

#include "ModularInteger.hpp" // for ModularInteger


template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
std::vector<ModularInteger<T_NARROW, T_WIDE, MODULUS>>
modular_inverse_table(std::size_t size) noexcept {
    using ModInt = ModularInteger<T_NARROW, T_WIDE, MODULUS>;
    std::vector<ModInt> result(size);
    for (std::size_t i = 0; i < size; ++i) {
        result[i] = ModInt(i + 1).inv();
    }
    return result;
}


template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
class PowerSumAccumulator {

    using ModInt = ModularInteger<T_NARROW, T_WIDE, MODULUS>;

    std::vector<ModInt> inverse_table;
    std::vector<ModInt> power_sums;

public:

    constexpr PowerSumAccumulator(std::size_t size) noexcept :
        inverse_table(modular_inverse_table<T_NARROW, T_WIDE, MODULUS>(size)),
        power_sums(size)
    {
        for (std::size_t i = 0; i < size; ++i) {
            power_sums[i] = ModInt();
        }
    }

    constexpr PowerSumAccumulator(const PowerSumAccumulator &other) noexcept :
        inverse_table(other.inverse_table), power_sums(other.power_sums) {}

    constexpr void insert(T_NARROW value) noexcept {
        const std::size_t size = power_sums.size();
        const ModInt x{value};
        ModInt y = x;
        for (std::size_t i = 0; i < size - 1; ++i) {
            power_sums[i] += y;
            y *= x;
        }
        power_sums[size - 1] += y;
    }

    constexpr void clear() noexcept {
        const std::size_t size = power_sums.size();
        for (std::size_t i = 0; i < size; ++i) {
            power_sums[i] = ModInt();
        }
    }

    constexpr PowerSumAccumulator &operator-=(
        const PowerSumAccumulator &other
    ) noexcept {
        const std::size_t size = power_sums.size();
        for (std::size_t i = 0; i < size; ++i) {
            power_sums[i] -= other.power_sums[i];
        }
        return *this;
    }

    constexpr std::vector<ModInt> to_polynomial_coefficients() const noexcept {
        const std::size_t size = power_sums.size();
        std::vector<ModInt> coeffs(size);
        coeffs[0] = -power_sums[0];
        for (std::size_t i = 1; i < size; ++i) {
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
