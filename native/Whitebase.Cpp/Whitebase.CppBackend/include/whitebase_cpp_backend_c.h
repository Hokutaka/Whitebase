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

int whitebase_cpp_is_avx_available(void);

int whitebase_cpp_add_f32_avx(
    const float* lhs,
    const float* rhs,
    float* output,
    size_t length
);

#ifdef __cplusplus
}
#endif