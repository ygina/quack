#ifndef MODULAR_INTEGER_HPP_INCLUDED
#define MODULAR_INTEGER_HPP_INCLUDED

#include <limits>      // for std::numeric_limits
#include <type_traits> // for std::is_integral, std::is_unsigned


template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
struct ModularInteger {

    static_assert(std::is_integral_v<T_NARROW> && std::is_unsigned_v<T_NARROW>,
                  "T_NARROW must be an unsigned integer type");
    static_assert(std::is_integral_v<T_WIDE> && std::is_unsigned_v<T_WIDE>,
                  "T_WIDE must be an unsigned integer type");
    static_assert(std::numeric_limits<T_WIDE>::max() >
                  std::numeric_limits<T_NARROW>::max(),
                  "T_WIDE must be wider than T_NARROW");
    // MODULUS must be a prime number.
    // T_WIDE must be able to represent the square of (MODULUS - 1).

    T_NARROW value;

    constexpr ModularInteger() noexcept :
        value(static_cast<T_NARROW>(0)) {}

    explicit constexpr ModularInteger(T_NARROW n) noexcept :
        value((n >= MODULUS) ? (n - MODULUS) : n) {}

    constexpr ModularInteger(const ModularInteger &) noexcept = default;

    constexpr ModularInteger(ModularInteger &&) noexcept = default;

    constexpr ModularInteger &operator=(const ModularInteger &rhs) noexcept {
        value = rhs.value;
        return *this;
    }

    constexpr ModularInteger &operator=(ModularInteger &&rhs) noexcept {
        value = rhs.value;
        return *this;
    }

    constexpr operator bool() const noexcept {
        return (value != static_cast<T_NARROW>(0));
    }

    constexpr ModularInteger &operator+=(const ModularInteger &rhs) noexcept {
        const T_WIDE sum = static_cast<T_WIDE>(value) +
                           static_cast<T_WIDE>(rhs.value);
        value = (sum >= MODULUS) ? static_cast<T_NARROW>(sum - MODULUS)
                                 : static_cast<T_NARROW>(sum);
        return *this;
    }

    constexpr ModularInteger operator+(const ModularInteger &rhs) const noexcept {
        ModularInteger result = *this;
        result += rhs;
        return result;
    }

    constexpr ModularInteger &operator-=(const ModularInteger &rhs) noexcept {
        const T_WIDE neg_rhs = static_cast<T_WIDE>(MODULUS) -
                               static_cast<T_WIDE>(rhs.value);
        const T_WIDE diff = static_cast<T_WIDE>(value) + neg_rhs;
        value = (diff >= MODULUS) ? static_cast<T_NARROW>(diff - MODULUS)
                                  : static_cast<T_NARROW>(diff);
        return *this;
    }

    constexpr ModularInteger operator-(const ModularInteger &rhs) const noexcept {
        ModularInteger result = *this;
        result -= rhs;
        return result;
    }

    constexpr ModularInteger operator-() const noexcept {
        if (value) {
            return ModularInteger(static_cast<T_NARROW>(MODULUS - value));
        } else {
            return *this;
        }
    }

    constexpr ModularInteger &operator*=(const ModularInteger &other) noexcept {
        const T_WIDE prod = static_cast<T_WIDE>(value) *
                            static_cast<T_WIDE>(other.value);
        value = static_cast<T_NARROW>(prod % MODULUS);
        return *this;
    }

    constexpr ModularInteger operator*(const ModularInteger &rhs) const noexcept {
        ModularInteger result = *this;
        result *= rhs;
        return result;
    }

    template <T_NARROW POWER>
    constexpr ModularInteger pow() const noexcept {
        if constexpr (POWER == 0) {
            return ModularInteger(static_cast<T_NARROW>(1));
        } else if constexpr (POWER == 1) {
            return *this;
        } else {
            ModularInteger result = pow<(POWER >> 1)>();
            result *= result;
            if constexpr (POWER & 1) {
                result *= *this;
            }
            return result;
        }
    }

    consteval ModularInteger inv() const noexcept {
        return pow<(MODULUS - 2)>();
    }

}; // struct ModularInteger


#endif // MODULAR_INTEGER_HPP_INCLUDED
