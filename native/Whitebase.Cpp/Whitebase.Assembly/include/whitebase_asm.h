#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

int whitebase_asm_add(int left, int right);

double whitebase_asm_add_f64_scalar(double lhs, double rhs);

void whitebase_asm_add_f32_scalar(
    const float* lhs,
    const float* rhs,
    float* output,
    size_t length
);

void whitebase_asm_add_f32_avx(
    const float* lhs,
    const float* rhs,
    float* output,
    size_t length
);

#ifdef __cplusplus
}
#endif