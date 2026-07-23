#include "whitebase_cpp_backend.h"

#include <immintrin.h>

namespace whitebase::cpp_backend
{
    bool add_f32_avx(
        const float* lhs,
        const float* rhs,
        float* output,
        const std::size_t length
    ) noexcept
    {
        if (!is_avx_available())
        {
            return false;
        }

        constexpr std::size_t lane_count = 8;

        const std::size_t vectorized_length =
            length / lane_count * lane_count;

        std::size_t index = 0;

        for (; index < vectorized_length; index += lane_count)
        {
            const __m256 lhs_values =
                _mm256_loadu_ps(lhs + index);

            const __m256 rhs_values =
                _mm256_loadu_ps(rhs + index);

            const __m256 result =
                _mm256_add_ps(lhs_values, rhs_values);

            _mm256_storeu_ps(output + index, result);
        }

        for (; index < length; ++index)
        {
            output[index] = lhs[index] + rhs[index];
        }

        _mm256_zeroupper();


        return true;
    }
}
