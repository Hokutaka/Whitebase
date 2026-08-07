#include "whitebase_linux_native.h"

#include <immintrin.h>

extern "C"
{
    int whitebase_cpp_add_f32_avx(
        const float* lhs,
        const float* rhs,
        float* output,
        const size_t length
    )
    {
        if (whitebase_cpp_is_avx_available() == 0)
        {
            return 0;
        }

        constexpr size_t lane_count = 8;
        const size_t vectorized_length =
            length / lane_count * lane_count;

        size_t index = 0;

        for (; index < vectorized_length; index += lane_count)
        {
            const __m256 lhs_values = _mm256_loadu_ps(lhs + index);
            const __m256 rhs_values = _mm256_loadu_ps(rhs + index);
            const __m256 result = _mm256_add_ps(lhs_values, rhs_values);

            _mm256_storeu_ps(output + index, result);
        }

        for (; index < length; ++index)
        {
            output[index] = lhs[index] + rhs[index];
        }

        _mm256_zeroupper();

        return 1;
    }

    int whitebase_cpp_add_f64_array_avx(
        const double* lhs,
        const double* rhs,
        double* output,
        const size_t length
    )
    {
        if (whitebase_cpp_is_avx_available() == 0)
        {
            return 0;
        }

        constexpr size_t lane_count = 4;
        const size_t vectorized_length =
            length / lane_count * lane_count;

        size_t index = 0;

        for (; index < vectorized_length; index += lane_count)
        {
            const __m256d lhs_values = _mm256_loadu_pd(lhs + index);
            const __m256d rhs_values = _mm256_loadu_pd(rhs + index);
            const __m256d result = _mm256_add_pd(lhs_values, rhs_values);

            _mm256_storeu_pd(output + index, result);
        }

        for (; index < length; ++index)
        {
            output[index] = lhs[index] + rhs[index];
        }

                _mm256_zeroupper();
        return 1;
    }

    int whitebase_cpp_sum_f64_avx(
        const double* input,
        const size_t length,
        double* output
    )
    {
        if (whitebase_cpp_is_avx_available() == 0)
        {
            return 0;
        }

        constexpr size_t lane_count = 4;
        const size_t vectorized_length =
            length / lane_count * lane_count;

        __m256d accumulator = _mm256_setzero_pd();
        size_t index = 0;
        for (; index < vectorized_length; index += lane_count)
        {
            const __m256d values = _mm256_loadu_pd(input + index);
            accumulator = _mm256_add_pd(accumulator, values);
        }

        double lanes[lane_count] {};
        _mm256_storeu_pd(lanes, accumulator);

        double sum = lanes[0] + lanes[1] + lanes[2] + lanes[3];
        for (; index < length; ++index)
        {
            sum += input[index];
        }

        *output = sum;
        _mm256_zeroupper();
        return 1;
    }
}

