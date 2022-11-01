use std::ops::SubAssign;
use crate::modint::ModularInteger;

// template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
// std::vector<ModularInteger<T_NARROW, T_WIDE, MODULUS>>
// modular_inverse_table(std::size_t size) noexcept {
//     using ModInt = ModularInteger<T_NARROW, T_WIDE, MODULUS>;
//     std::vector<ModInt> result(size);
//     for (std::size_t i = 0; i < size; ++i) {
//         result[i] = ModInt(i + 1).inv();
//     }
//     return result;
// }
fn modular_inverse_table(size: usize) -> Vec<ModularInteger> {
    (0..(size as u32)).map(|i| ModularInteger::new(i+1).inv()).collect()
}

pub struct PowerSumAccumulator {
    pub inverse_table: Vec<ModularInteger>,
    pub power_sums: Vec<ModularInteger>,
    pub count: u16,
}

impl PowerSumAccumulator {
    pub fn new(size: usize) -> Self {
        Self {
            inverse_table: modular_inverse_table(size),
            power_sums: (0..size).map(|_| ModularInteger::zero()).collect(),
            count: 0,
        }
    }

    pub fn insert(&mut self, value: u32) {
        let size = self.power_sums.len();
        let x = ModularInteger::new(value);
        let mut y = x;
        for i in 0..(size-1) {
            self.power_sums[i] += y;
            y *= x;
        }
        self.power_sums[size - 1] += y;
        self.count += 1;
    }

    pub fn to_polynomial_coefficients(self, coeffs: &mut Vec<ModularInteger>) {
        let size = coeffs.len();
        coeffs[0] = -self.power_sums[0];
        for i in 1..size {
            for j in 0..i {
                coeffs[i] = coeffs[i] - self.power_sums[j] * coeffs[i - j - 1];
            }
            coeffs[i] -= self.power_sums[i];
            coeffs[i] *= self.inverse_table[i];
        }
    }
}

//     constexpr void insert(T_NARROW value) noexcept {
//         const std::size_t size = power_sums.size();
//         const ModInt x{value};
//         ModInt y = x;
//         for (std::size_t i = 0; i < size - 1; ++i) {
//             power_sums[i] += y;
//             y *= x;
//         }
//         power_sums[size - 1] += y;
//         count++;
//     }

//     constexpr void insert(
//         const std::vector<ModInt> &power_tables,
//         std::size_t table_size,
//         T_NARROW value
//     ) noexcept {
//         const ModInt *power_table = power_tables.data() + (table_size * value);
//         const std::size_t size = power_sums.size();
//         for (std::size_t i = 0; i < size; ++i) {
//             power_sums[i] += ModInt(power_table[i]);
//         }
//         count++;
//     }

//     constexpr void clear() noexcept {
//         const std::size_t size = power_sums.size();
//         for (std::size_t i = 0; i < size; ++i) {
//             power_sums[i] = ModInt();
//         }
//         count = 0;
//     }

impl SubAssign for PowerSumAccumulator {
    fn sub_assign(&mut self, rhs: Self) {
        let size = self.power_sums.len();
        for i in 0..size {
            self.power_sums[i] -= rhs.power_sums[i];
        }
        self.count -= rhs.count;
    }
}
