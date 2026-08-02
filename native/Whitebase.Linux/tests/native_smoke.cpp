#include "whitebase_linux_native.h"

#include <algorithm>
#include <array>
#include <bit>
#include <cstdint>
#include <iostream>

namespace
{
    template <typename T, std::size_t Length>
    bool matches(
        const std::array<T, Length>& actual,
        const std::array<T, Length>& expected
    )
    {
        return std::equal(actual.begin(), actual.end(), expected.begin());
    }

    bool test_cpp_scalar_backend()
    {
        const std::array<float, 4> lhs_f32 { 1.0F, 2.0F, 3.0F, 4.0F };
        const std::array<float, 4> rhs_f32 { 10.0F, 20.0F, 30.0F, 40.0F };
        const std::array<float, 4> expected_f32 { 11.0F, 22.0F, 33.0F, 44.0F };
        std::array<float, 4> output_f32 {};

        whitebase_cpp_add_f32_scalar(
            lhs_f32.data(),
            rhs_f32.data(),
            output_f32.data(),
            output_f32.size()
        );

        const std::array<double, 4> lhs_f64 { 0.1, 1.0, 2.0, 3.0 };
        const std::array<double, 4> rhs_f64 { 0.2, 10.0, 20.0, 30.0 };
        const std::array<double, 4> expected_f64 { 0.1 + 0.2, 11.0, 22.0, 33.0 };
        std::array<double, 4> output_f64 {};

        whitebase_cpp_add_f64_array_scalar(
            lhs_f64.data(),
            rhs_f64.data(),
            output_f64.data(),
            output_f64.size()
        );

        const double scalar = whitebase_cpp_add_f64_scalar(0.1, 0.2);

        return matches(output_f32, expected_f32)
            && matches(output_f64, expected_f64)
            && std::bit_cast<std::uint64_t>(scalar) == UINT64_C(0x3fd3333333333334);
    }

    bool test_cpp_avx_backend()
    {
        const std::array<float, 10> lhs_f32 {
            1.0F, 2.0F, 3.0F, 4.0F, 5.0F,
            6.0F, 7.0F, 8.0F, 9.0F, 10.0F
        };
        const std::array<float, 10> rhs_f32 {
            10.0F, 20.0F, 30.0F, 40.0F, 50.0F,
            60.0F, 70.0F, 80.0F, 90.0F, 100.0F
        };
        const std::array<float, 10> expected_f32 {
            11.0F, 22.0F, 33.0F, 44.0F, 55.0F,
            66.0F, 77.0F, 88.0F, 99.0F, 110.0F
        };
        std::array<float, 10> output_f32 {};

        if (whitebase_cpp_add_f32_avx(
            lhs_f32.data(),
            rhs_f32.data(),
            output_f32.data(),
            output_f32.size()
        ) == 0)
        {
            return false;
        }

        const std::array<double, 6> lhs_f64 { 0.1, 1.0, 2.0, 3.0, 4.0, 5.0 };
        const std::array<double, 6> rhs_f64 { 0.2, 10.0, 20.0, 30.0, 40.0, 50.0 };
        const std::array<double, 6> expected_f64 { 0.1 + 0.2, 11.0, 22.0, 33.0, 44.0, 55.0 };
        std::array<double, 6> output_f64 {};

        if (whitebase_cpp_add_f64_array_avx(
            lhs_f64.data(),
            rhs_f64.data(),
            output_f64.data(),
            output_f64.size()
        ) == 0)
        {
            return false;
        }

        return matches(output_f32, expected_f32)
            && matches(output_f64, expected_f64);
    }

    bool test_assembly_scalar_backend()
    {
        const std::array<float, 4> lhs_f32 { 1.0F, 2.0F, 3.0F, 4.0F };
        const std::array<float, 4> rhs_f32 { 10.0F, 20.0F, 30.0F, 40.0F };
        const std::array<float, 4> expected_f32 { 11.0F, 22.0F, 33.0F, 44.0F };
        std::array<float, 4> output_f32 {};

        whitebase_asm_add_f32_scalar(
            lhs_f32.data(),
            rhs_f32.data(),
            output_f32.data(),
            output_f32.size()
        );

        const std::array<double, 4> lhs_f64 { 0.1, 1.0, 2.0, 3.0 };
        const std::array<double, 4> rhs_f64 { 0.2, 10.0, 20.0, 30.0 };
        const std::array<double, 4> expected_f64 { 0.1 + 0.2, 11.0, 22.0, 33.0 };
        std::array<double, 4> output_f64 {};

        whitebase_asm_add_f64_array_scalar(
            lhs_f64.data(),
            rhs_f64.data(),
            output_f64.data(),
            output_f64.size()
        );

        const double scalar = whitebase_asm_add_f64_scalar(0.1, 0.2);

        return matches(output_f32, expected_f32)
            && matches(output_f64, expected_f64)
            && std::bit_cast<std::uint64_t>(scalar) == UINT64_C(0x3fd3333333333334);
    }

    bool test_assembly_avx_backend()
    {
        const std::array<float, 10> lhs_f32 {
            1.0F, 2.0F, 3.0F, 4.0F, 5.0F,
            6.0F, 7.0F, 8.0F, 9.0F, 10.0F
        };
        const std::array<float, 10> rhs_f32 {
            10.0F, 20.0F, 30.0F, 40.0F, 50.0F,
            60.0F, 70.0F, 80.0F, 90.0F, 100.0F
        };
        const std::array<float, 10> expected_f32 {
            11.0F, 22.0F, 33.0F, 44.0F, 55.0F,
            66.0F, 77.0F, 88.0F, 99.0F, 110.0F
        };
        std::array<float, 10> output_f32 {};

        whitebase_asm_add_f32_avx(
            lhs_f32.data(),
            rhs_f32.data(),
            output_f32.data(),
            output_f32.size()
        );

        const std::array<double, 6> lhs_f64 { 0.1, 1.0, 2.0, 3.0, 4.0, 5.0 };
        const std::array<double, 6> rhs_f64 { 0.2, 10.0, 20.0, 30.0, 40.0, 50.0 };
        const std::array<double, 6> expected_f64 { 0.1 + 0.2, 11.0, 22.0, 33.0, 44.0, 55.0 };
        std::array<double, 6> output_f64 {};

        whitebase_asm_add_f64_array_avx(
            lhs_f64.data(),
            rhs_f64.data(),
            output_f64.data(),
            output_f64.size()
        );

        return matches(output_f32, expected_f32)
            && matches(output_f64, expected_f64);
    }

    const char* avx_status(const bool available, const bool passed)
    {
        if (!available)
        {
            return "SKIPPED";
        }

        return passed ? "PASSED" : "FAILED";
    }
}

int main()
{
    const bool avx_available = whitebase_cpp_is_avx_available() != 0;

    const bool cpp_scalar_passed = test_cpp_scalar_backend();
    const bool assembly_scalar_passed = test_assembly_scalar_backend();

    const bool cpp_avx_passed =
        !avx_available || test_cpp_avx_backend();

    const bool assembly_avx_passed =
        !avx_available || test_assembly_avx_backend();

    std::cout << "C++ GCC Scalar: "
              << (cpp_scalar_passed ? "PASSED" : "FAILED")
              << '\n';

    std::cout << "C++ GCC AVX: "
              << avx_status(avx_available, cpp_avx_passed)
              << '\n';

    std::cout << "Assembly NASM Scalar: "
              << (assembly_scalar_passed ? "PASSED" : "FAILED")
              << '\n';

    std::cout << "Assembly NASM AVX: "
              << avx_status(avx_available, assembly_avx_passed)
              << '\n';

    return cpp_scalar_passed
        && cpp_avx_passed
        && assembly_scalar_passed
        && assembly_avx_passed
        ? 0
        : 1;
}
