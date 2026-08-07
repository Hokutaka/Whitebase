#include "whitebase_linux_native.h"

extern "C"
{
    void whitebase_cpp_add_f32_scalar(
        const float* lhs,
        const float* rhs,
        float* output,
        const size_t length
    )
    {
        for (size_t index = 0; index < length; ++index)
        {
            output[index] = lhs[index] + rhs[index];
        }
    }

    void whitebase_cpp_add_f64_array_scalar(
        const double* lhs,
        const double* rhs,
        double* output,
        const size_t length
    )
    {
        for (size_t index = 0; index < length; ++index)
        {
            output[index] = lhs[index] + rhs[index];
        }
    }

    double whitebase_cpp_add_f64_scalar(
        const double lhs,
        const double rhs
    )
    {
        return lhs + rhs;
    }

    double whitebase_cpp_sum_f64_scalar(
        const double* input,
        const size_t length
    )
    {
        double sum = 0.0;
        for (size_t index = 0; index < length; ++index)
        {
            sum += input[index];
        }

        return sum;
    }
}
