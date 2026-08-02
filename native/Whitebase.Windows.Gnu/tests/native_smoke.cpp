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

        return arrays_match(output_f32, expected_f32)
            && arrays_match(output_f64, expected_f64)
            && std::abs(scalar - 0.75) <= 1.0e-12;
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

        return arrays_match(output_f32, expected_f32)
            && arrays_match(output_f64, expected_f64)
            && std::abs(scalar - 0.75) <= 1.0e-12;
    }
}

int main()
{
    const bool gcc_passed = test_gcc_scalar();
    const bool nasm_passed = test_nasm_scalar();

    std::cout
        << "Windows GCC Scalar: "
        << (gcc_passed ? "PASSED" : "FAILED")
        << '\n';

    std::cout
        << "Windows NASM Scalar: "
        << (nasm_passed ? "PASSED" : "FAILED")
        << '\n';

    return gcc_passed && nasm_passed ? 0 : 1;
}
