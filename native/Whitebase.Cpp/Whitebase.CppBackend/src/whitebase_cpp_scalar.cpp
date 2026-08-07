#include "whitebase_cpp_backend.h"

namespace whitebase::cpp_backend
{
    void add_f32_scalar(
        const float* lhs,
        const float* rhs,
        float* output,
        const std::size_t length
    ) noexcept
    {
        for (std::size_t index = 0; index < length; ++index)
        {
            output[index] = lhs[index] + rhs[index];
        }
    }

    void add_f64_array_scalar(
        const double* lhs,
        const double* rhs,
        double* output,
        const std::size_t length
    ) noexcept
    {
        for (std::size_t index = 0; index < length; ++index)
        {
            output[index] = lhs[index] + rhs[index];
        }
    }

    double add_f64_scalar(const double lhs, const double rhs) noexcept
    {
        return lhs + rhs;
    }

    double sum_f64_scalar(
        const double* input,
        const std::size_t length
    ) noexcept
    {
        double sum = 0.0;
        for (std::size_t index = 0; index < length; ++index)
        {
            sum += input[index];
        }

        return sum;
    }
}