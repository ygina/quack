#ifndef QUACK_H
#define QUACK_H

#ifdef __cplusplus
extern "C" {
#endif

void quack_global_config_set_max_power_sum_threshold(size_t threshold);

typedef struct PowerSumQuackU32 PowerSumQuackU32;

PowerSumQuackU32* quack_new(size_t threshold);
size_t quack_threshold(const PowerSumQuackU32* quack);
unsigned int quack_count(const PowerSumQuackU32* quack);
unsigned int quack_last_value(const PowerSumQuackU32* quack);
void quack_insert(PowerSumQuackU32* quack, unsigned int value);
void quack_remove(PowerSumQuackU32* quack, unsigned int value);
size_t quack_decode_with_log(const PowerSumQuackU32* quack, const unsigned int* log, size_t len, unsigned int* out_buffer, size_t out_buffer_size);
PowerSumQuackU32* quack_sub(PowerSumQuackU32* lhs, PowerSumQuackU32* rhs);
void quack_free(PowerSumQuackU32* quack);

typedef struct CoefficientVectorU32 CoefficientVectorU32;

CoefficientVectorU32* quack_to_coeffs(const PowerSumQuackU32* quack);
unsigned int quack_coeffs_eval(CoefficientVectorU32* coeffs, unsigned int x);
void quack_coeffs_free(CoefficientVectorU32* coeffs);

#ifdef __cplusplus
}
#endif

#endif // QUACK_H