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
}