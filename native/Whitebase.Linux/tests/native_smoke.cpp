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

    bool test_cpp_backend()
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

    bool test_assembly_backend()
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
}

int main()
{
    const bool cpp_passed = test_cpp_backend();
    const bool assembly_passed = test_assembly_backend();

    std::cout << "C++ GCC Scalar: "
              << (cpp_passed ? "PASSED" : "FAILED")
              << '\n';

    std::cout << "Assembly NASM Scalar: "
              << (assembly_passed ? "PASSED" : "FAILED")
              << '\n';

    return cpp_passed && assembly_passed ? 0 : 1;
}
