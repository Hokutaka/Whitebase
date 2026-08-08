#pragma once

#include <stddef.h>
#include <stdint.h>

#if defined(_WIN32)
#define WHITEBASE_API __declspec(dllimport)
#else
#define WHITEBASE_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef enum whitebase_status
{
    WHITEBASE_STATUS_OK = 0,
    WHITEBASE_STATUS_INVALID_ARGUMENT = 1,
    WHITEBASE_STATUS_UNKNOWN_BACKEND = 2,
    WHITEBASE_STATUS_BACKEND_NOT_REGISTERED = 3,
    WHITEBASE_STATUS_BACKEND_UNAVAILABLE = 4,
    WHITEBASE_STATUS_OPERATION_UNSUPPORTED = 5,
    WHITEBASE_STATUS_BACKEND_FAILURE = 6,
    WHITEBASE_STATUS_INTERNAL_PANIC = 7,
    WHITEBASE_STATUS_UNKNOWN_OPERATION = 8
} whitebase_status;

typedef enum whitebase_backend
{
    WHITEBASE_BACKEND_RUST_SCALAR = 0,
    WHITEBASE_BACKEND_RUST_SIMD = 1,
    WHITEBASE_BACKEND_CPP_SCALAR = 2,
    WHITEBASE_BACKEND_CPP_AVX = 3,
    WHITEBASE_BACKEND_ASSEMBLY_SCALAR = 4,
    WHITEBASE_BACKEND_ASSEMBLY_AVX = 5,
    WHITEBASE_BACKEND_WINDOWS_GNU_CPP_SCALAR = 6,
    WHITEBASE_BACKEND_WINDOWS_GNU_CPP_AVX = 7,
    WHITEBASE_BACKEND_WINDOWS_GNU_ASSEMBLY_SCALAR = 8,
    WHITEBASE_BACKEND_WINDOWS_GNU_ASSEMBLY_AVX = 9
} whitebase_backend;

typedef enum whitebase_operation
{
    WHITEBASE_OPERATION_ADD_F32 = 0,
    WHITEBASE_OPERATION_ADD_F64 = 1,
    WHITEBASE_OPERATION_ADD_SCALAR_F64 = 2,
    WHITEBASE_OPERATION_SUM_F64 = 3
} whitebase_operation;

WHITEBASE_API int32_t whitebase_backend_is_available(
    uint32_t backend,
    int32_t* available);

WHITEBASE_API int32_t whitebase_backend_supports(
    uint32_t backend,
    uint32_t operation,
    int32_t* supported);

WHITEBASE_API int32_t whitebase_add_f32(
    uint32_t backend,
    const float* lhs,
    const float* rhs,
    float* output,
    size_t length);

WHITEBASE_API int32_t whitebase_add_f64(
    uint32_t backend,
    const double* lhs,
    const double* rhs,
    double* output,
    size_t length);

WHITEBASE_API int32_t whitebase_add_scalar_f64(
    uint32_t backend,
    double lhs,
    double rhs,
    double* output);

WHITEBASE_API int32_t whitebase_sum_f64(
    uint32_t backend,
    const double* input,
    size_t length,
    double* output);

#ifdef __cplusplus
}
#endif
