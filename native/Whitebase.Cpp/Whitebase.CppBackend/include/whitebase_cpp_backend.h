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

    [[nodiscard]]
    bool is_avx_available() noexcept;

    [[nodiscard]]
    bool add_f32_avx(
        const float* lhs,
        const float* rhs,
        float* output,
        std::size_t length
    ) noexcept;
}