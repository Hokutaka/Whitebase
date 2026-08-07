#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

void whitebase_cpp_add_f32_scalar(
    const float* lhs,
    const float* rhs,
    float* output,
    size_t length
);

void whitebase_cpp_add_f64_array_scalar(
    const double* lhs,
    const double* rhs,
    double* output,
    size_t length
);

double whitebase_cpp_add_f64_scalar(double lhs, double rhs);
double whitebase_cpp_sum_f64_scalar(const double* input, size_t length);

int whitebase_cpp_is_avx_available(void);

int whitebase_cpp_add_f32_avx(
    const float* lhs,
    const float* rhs,
    float* output,
    size_t length
);

int whitebase_cpp_add_f64_array_avx(
    const double* lhs,
    const double* rhs,
    double* output,
    size_t length
);
int whitebase_cpp_sum_f64_avx(
    const double* input,
    size_t length,
    double* output
);

void whitebase_asm_add_f32_scalar(
    const float* lhs,
    const float* rhs,
    float* output,
    size_t length
);

void whitebase_asm_add_f64_array_scalar(
    const double* lhs,
    const double* rhs,
    double* output,
    size_t length
);

double whitebase_asm_add_f64_scalar(double lhs, double rhs);

/*
 * Returns 1 when the AVX operation was executed.
 * Returns 0 without modifying output when AVX is unavailable.
 */
int whitebase_asm_add_f32_avx(
    const float* lhs,
    const float* rhs,
    float* output,
    size_t length
);

/*
 * Returns 1 when the AVX operation was executed.
 * Returns 0 without modifying output when AVX is unavailable.
 */
int whitebase_asm_add_f64_array_avx(
    const double* lhs,
    const double* rhs,
    double* output,
    size_t length
);

#ifdef __cplusplus
}
#endif
