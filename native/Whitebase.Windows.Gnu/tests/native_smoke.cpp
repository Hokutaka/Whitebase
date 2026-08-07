#include "whitebase_windows_gnu_native.h"

#include <array>
#include <cmath>
#include <cstddef>
#include <iostream>

namespace
{
    template <typename Value, std::size_t Length>
    bool arrays_match(
        const std::array<Value, Length>& actual,
        const std::array<Value, Length>& expected
    )
    {
        for (std::size_t index = 0; index < Length; ++index)
        {
            if (std::abs(actual[index] - expected[index]) > static_cast<Value>(1.0e-6))
            {
                return false;
            }
        }

        return true;
    }

    bool test_gcc_scalar()
    {
        constexpr std::array<float, 5> lhs_f32{1.0F, -2.0F, 3.5F, 0.0F, 8.0F};
        constexpr std::array<float, 5> rhs_f32{4.0F, 5.0F, -1.5F, 2.0F, -3.0F};
        constexpr std::array<float, 5> expected_f32{5.0F, 3.0F, 2.0F, 2.0F, 5.0F};
        std::array<float, 5> output_f32{};

        whitebase_gnu_cpp_add_f32_scalar(
            lhs_f32.data(),
            rhs_f32.data(),
            output_f32.data(),
            output_f32.size()
        );

        constexpr std::array<double, 5> lhs_f64{1.0, -2.0, 3.5, 0.0, 8.0};
        constexpr std::array<double, 5> rhs_f64{4.0, 5.0, -1.5, 2.0, -3.0};
        constexpr std::array<double, 5> expected_f64{5.0, 3.0, 2.0, 2.0, 5.0};
        std::array<double, 5> output_f64{};

        whitebase_gnu_cpp_add_f64_array_scalar(
            lhs_f64.data(),
            rhs_f64.data(),
            output_f64.data(),
            output_f64.size()
        );

        const double scalar = whitebase_gnu_cpp_add_f64_scalar(0.25, 0.5);
        const double sum = whitebase_gnu_cpp_sum_f64_scalar(
            lhs_f64.data(),
            lhs_f64.size()
        );
        constexpr std::array<double, 0> empty{};
        const double empty_sum = whitebase_gnu_cpp_sum_f64_scalar(
            empty.data(),
            empty.size()
        );

        return arrays_match(output_f32, expected_f32)
            && arrays_match(output_f64, expected_f64)
            && std::abs(scalar - 0.75) <= 1.0e-12
            && std::abs(sum - 10.5) <= 1.0e-12
            && empty_sum == 0.0;
    }

    bool test_gcc_avx(const bool avx_available)
    {
        constexpr std::array<float, 10> lhs_f32{
            1.0F, 2.0F, 3.0F, 4.0F, 5.0F,
            6.0F, 7.0F, 8.0F, 9.0F, 10.0F,
        };
        constexpr std::array<float, 10> rhs_f32{
            10.0F, 20.0F, 30.0F, 40.0F, 50.0F,
            60.0F, 70.0F, 80.0F, 90.0F, 100.0F,
        };
        constexpr std::array<float, 10> expected_f32{
            11.0F, 22.0F, 33.0F, 44.0F, 55.0F,
            66.0F, 77.0F, 88.0F, 99.0F, 110.0F,
        };
        std::array<float, 10> output_f32{};

        const int f32_executed = whitebase_gnu_cpp_add_f32_avx(
            lhs_f32.data(),
            rhs_f32.data(),
            output_f32.data(),
            output_f32.size()
        );

        constexpr std::array<double, 6> lhs_f64{0.1, 1.0, 2.0, 3.0, 4.0, 5.0};
        constexpr std::array<double, 6> rhs_f64{0.2, 10.0, 20.0, 30.0, 40.0, 50.0};
        constexpr std::array<double, 6> expected_f64{0.3, 11.0, 22.0, 33.0, 44.0, 55.0};
        std::array<double, 6> output_f64{};

        const int f64_executed = whitebase_gnu_cpp_add_f64_array_avx(
            lhs_f64.data(),
            rhs_f64.data(),
            output_f64.data(),
            output_f64.size()
        );

        constexpr std::array<double, 10> sum_input{
            1.0, 2.0, 3.0, 4.0, 5.0,
            6.0, 7.0, 8.0, 9.0, 10.0,
        };
        double sum_output = -1234.0;
        const int sum_executed = whitebase_gnu_cpp_sum_f64_avx(
            sum_input.data(),
            sum_input.size(),
            &sum_output
        );

        if (!avx_available)
        {
            return f32_executed == 0
                && f64_executed == 0
                && sum_executed == 0
                && sum_output == -1234.0
                && arrays_match(output_f32, std::array<float, 10>{})
                && arrays_match(output_f64, std::array<double, 6>{});
        }

        return f32_executed != 0
            && f64_executed != 0
            && sum_executed != 0
            && std::abs(sum_output - 55.0) <= 1.0e-12
            && arrays_match(output_f32, expected_f32)
            && arrays_match(output_f64, expected_f64);
    }

    bool test_nasm_scalar()
    {
        constexpr std::array<float, 5> lhs_f32{1.0F, -2.0F, 3.5F, 0.0F, 8.0F};
        constexpr std::array<float, 5> rhs_f32{4.0F, 5.0F, -1.5F, 2.0F, -3.0F};
        constexpr std::array<float, 5> expected_f32{5.0F, 3.0F, 2.0F, 2.0F, 5.0F};
        std::array<float, 5> output_f32{};

        whitebase_gnu_asm_add_f32_scalar(
            lhs_f32.data(),
            rhs_f32.data(),
            output_f32.data(),
            output_f32.size()
        );

        constexpr std::array<double, 5> lhs_f64{1.0, -2.0, 3.5, 0.0, 8.0};
        constexpr std::array<double, 5> rhs_f64{4.0, 5.0, -1.5, 2.0, -3.0};
        constexpr std::array<double, 5> expected_f64{5.0, 3.0, 2.0, 2.0, 5.0};
        std::array<double, 5> output_f64{};

        whitebase_gnu_asm_add_f64_array_scalar(
            lhs_f64.data(),
            rhs_f64.data(),
            output_f64.data(),
            output_f64.size()
        );

        const double scalar = whitebase_gnu_asm_add_f64_scalar(0.25, 0.5);
        const double sum = whitebase_gnu_asm_sum_f64_scalar(
            lhs_f64.data(),
            lhs_f64.size()
        );
        constexpr std::array<double, 0> empty{};
        const double empty_sum = whitebase_gnu_asm_sum_f64_scalar(
            empty.data(),
            empty.size()
        );


        return arrays_match(output_f32, expected_f32)
            && arrays_match(output_f64, expected_f64)
            && std::abs(scalar - 0.75) <= 1.0e-12
            && std::abs(sum - 10.5) <= 1.0e-12
            && empty_sum == 0.0;
    }

    bool test_nasm_avx(const bool avx_available)
    {
        constexpr std::array<float, 10> lhs_f32{
            1.0F, 2.0F, 3.0F, 4.0F, 5.0F,
            6.0F, 7.0F, 8.0F, 9.0F, 10.0F,
        };
        constexpr std::array<float, 10> rhs_f32{
            10.0F, 20.0F, 30.0F, 40.0F, 50.0F,
            60.0F, 70.0F, 80.0F, 90.0F, 100.0F,
        };
        constexpr std::array<float, 10> expected_f32{
            11.0F, 22.0F, 33.0F, 44.0F, 55.0F,
            66.0F, 77.0F, 88.0F, 99.0F, 110.0F,
        };
        std::array<float, 10> output_f32{};

        const int f32_executed = whitebase_gnu_asm_add_f32_avx(
            lhs_f32.data(),
            rhs_f32.data(),
            output_f32.data(),
            output_f32.size()
        );

        constexpr std::array<double, 6> lhs_f64{0.1, 1.0, 2.0, 3.0, 4.0, 5.0};
        constexpr std::array<double, 6> rhs_f64{0.2, 10.0, 20.0, 30.0, 40.0, 50.0};
        constexpr std::array<double, 6> expected_f64{0.3, 11.0, 22.0, 33.0, 44.0, 55.0};
        std::array<double, 6> output_f64{};

        const int f64_executed = whitebase_gnu_asm_add_f64_array_avx(
            lhs_f64.data(),
            rhs_f64.data(),
            output_f64.data(),
            output_f64.size()
        );

        constexpr std::array<double, 10> sum_input{
            1.0, 2.0, 3.0, 4.0, 5.0,
            6.0, 7.0, 8.0, 9.0, 10.0,
        };
        double sum_output = -1234.0;
        const int sum_executed = whitebase_gnu_asm_sum_f64_avx(
            sum_input.data(),
            sum_input.size(),
            &sum_output
        );

        if (!avx_available)
        {
            return f32_executed == 0
                && f64_executed == 0
                && sum_executed == 0
                && sum_output == -1234.0
                && arrays_match(output_f32, std::array<float, 10>{})
                && arrays_match(output_f64, std::array<double, 6>{});
        }

        return f32_executed != 0
            && f64_executed != 0
            && sum_executed != 0
            && std::abs(sum_output - 55.0) <= 1.0e-12
            && arrays_match(output_f32, expected_f32)
            && arrays_match(output_f64, expected_f64);
    }

    const char* avx_status(const bool available, const bool passed)
    {
        if (!passed)
        {
            return "FAILED";
        }

        return available ? "PASSED" : "SKIPPED";
    }
}

int main()
{
    const bool avx_available = whitebase_gnu_cpp_is_avx_available() != 0;

    const bool gcc_scalar_passed = test_gcc_scalar();
    const bool gcc_avx_passed = test_gcc_avx(avx_available);
    const bool nasm_scalar_passed = test_nasm_scalar();
    const bool nasm_avx_passed = test_nasm_avx(avx_available);

    std::cout
        << "Windows GCC Scalar: "
        << (gcc_scalar_passed ? "PASSED" : "FAILED")
        << '\n';
    std::cout
        << "Windows GCC AVX: "
        << avx_status(avx_available, gcc_avx_passed)
        << '\n';
    std::cout
        << "Windows NASM Scalar: "
        << (nasm_scalar_passed ? "PASSED" : "FAILED")
        << '\n';
    std::cout
        << "Windows NASM AVX: "
        << avx_status(avx_available, nasm_avx_passed)
        << '\n';

    return gcc_scalar_passed
        && gcc_avx_passed
        && nasm_scalar_passed
        && nasm_avx_passed
        ? 0
        : 1;
}
