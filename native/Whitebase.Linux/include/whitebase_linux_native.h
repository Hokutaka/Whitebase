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

#ifdef __cplusplus
}
#endif
