#pragma once

#include <cstddef>

namespace whitebase::cpp_backend
{
    void add_f32_scalar(
        const float* lhs,
        const float* rhs,
        float* output,
        std::size_t length
    ) noexcept;

    void add_f64_array_scalar(
        const double* lhs,
        const double* rhs,
        double* output,
        std::size_t length
    ) noexcept;

    [[nodiscard]]
    double add_f64_scalar(double lhs, double rhs) noexcept;

    [[nodiscard]]
    double sum_f64_scalar(
        const double* input,
        std::size_t length
    ) noexcept;

    [[nodiscard]]
    bool is_avx_available() noexcept;

    [[nodiscard]]
    bool add_f32_avx(
        const float* lhs,
        const float* rhs,
        float* output,
        std::size_t length
    ) noexcept;

    [[nodiscard]]
    bool add_f64_array_avx(
        const double* lhs,
        const double* rhs,
        double* output,
        std::size_t length
    ) noexcept;

    [[nodiscard]]
    bool sum_f64_avx(
        const double* input,
        std::size_t length,
        double* output
    ) noexcept;
}