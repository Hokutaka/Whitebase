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
}
