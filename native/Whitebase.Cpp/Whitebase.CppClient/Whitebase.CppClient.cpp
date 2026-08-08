#include <array>
#include <cstdint>
#include <iostream>

#include "whitebase.h"

namespace
{
    bool require_status(
        const std::int32_t status,
        const std::int32_t expected,
        const char* operation)
    {
        if (status == expected)
        {
            return true;
        }

        std::cerr
            << operation
            << " returned status "
            << status
            << ", expected "
            << expected
            << ".\n";
        return false;
    }
}

int main()
{
    constexpr std::array<double, 10> input{
        1.0, 2.0, 3.0, 4.0, 5.0,
        6.0, 7.0, 8.0, 9.0, 10.0,
    };

    std::int32_t scalar_supports_sum = 0;
    if (!require_status(
            whitebase_backend_supports(
                WHITEBASE_BACKEND_ASSEMBLY_SCALAR,
                WHITEBASE_OPERATION_SUM_F64,
                &scalar_supports_sum),
            WHITEBASE_STATUS_OK,
            "whitebase_backend_supports(Assembly Scalar)"))
    {
        return 1;
    }

    if (scalar_supports_sum == 0)
    {
        std::cerr << "Assembly Scalar does not report SumF64 support.\n";
        return 1;
    }

    double scalar_sum = -1.0;
    if (!require_status(
            whitebase_sum_f64(
                WHITEBASE_BACKEND_ASSEMBLY_SCALAR,
                input.data(),
                input.size(),
                &scalar_sum),
            WHITEBASE_STATUS_OK,
            "whitebase_sum_f64(Assembly Scalar)"))
    {
        return 1;
    }

    if (scalar_sum != 55.0)
    {
        std::cerr << "Unexpected Assembly Scalar SumF64 result.\n";
        return 1;
    }

    std::cout << "C API -> Core -> Assembly Scalar SumF64 passed.\n";

    std::int32_t avx_available = 0;
    if (!require_status(
            whitebase_backend_is_available(
                WHITEBASE_BACKEND_ASSEMBLY_AVX,
                &avx_available),
            WHITEBASE_STATUS_OK,
            "whitebase_backend_is_available(Assembly AVX)"))
    {
        return 1;
    }

    double avx_sum = -1234.0;
    const std::int32_t avx_status =
        whitebase_sum_f64(
            WHITEBASE_BACKEND_ASSEMBLY_AVX,
            input.data(),
            input.size(),
            &avx_sum);

    if (avx_available != 0)
    {
        if (!require_status(
                avx_status,
                WHITEBASE_STATUS_OK,
                "whitebase_sum_f64(Assembly AVX)"))
        {
            return 1;
        }

        if (avx_sum != 55.0)
        {
            std::cerr << "Unexpected Assembly AVX SumF64 result.\n";
            return 1;
        }

        std::cout << "C API -> Core -> Assembly AVX SumF64 passed.\n";
    }
    else
    {
        if (!require_status(
                avx_status,
                WHITEBASE_STATUS_BACKEND_UNAVAILABLE,
                "whitebase_sum_f64(Assembly AVX unavailable)"))
        {
            return 1;
        }

        if (avx_sum != -1234.0)
        {
            std::cerr << "Unavailable Assembly AVX modified the SumF64 output.\n";
            return 1;
        }

        std::cout << "Assembly AVX unavailable path passed.\n";
    }

    return 0;
}
