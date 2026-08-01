#include <array>
#include <bit>
#include <cstdint>
#include <iostream>

#include "whitebase_asm.h"

int main()
{
    constexpr int left = 2;
    constexpr int right = 3;

    const int result =
        whitebase_asm_add(left, right);

    std::cout
        << "Whitebase Assembly smoke test\n"
        << left << " + " << right << " = " << result << '\n';

    if (result != 5)
    {
        std::cerr << "Unexpected integer add result.\n";
        return 1;
    }

    const double scalar_f64_result =
        whitebase_asm_add_f64_scalar(0.1, 0.2);

    if (std::bit_cast<std::uint64_t>(scalar_f64_result) !=
        0x3fd3333333333334ULL)
    {
        std::cerr << "Unexpected scalar f64 add result.\n";
        return 1;
    }

    std::cout << "Scalar f64 add passed.\n";

    constexpr std::size_t length = 10;

    const std::array<float, length> lhs{
        1.0F, 2.0F, 3.0F, 4.0F, 5.0F,
        6.0F, 7.0F, 8.0F, 9.0F, 10.0F
    };

    const std::array<float, length> rhs{
        10.0F, 20.0F, 30.0F, 40.0F, 50.0F,
        60.0F, 70.0F, 80.0F, 90.0F, 100.0F
    };

    const std::array<float, length> expected{
        11.0F, 22.0F, 33.0F, 44.0F, 55.0F,
        66.0F, 77.0F, 88.0F, 99.0F, 110.0F
    };

    std::array<float, length> output{};

    whitebase_asm_add_f32_scalar(
        lhs.data(),
        rhs.data(),
        output.data(),
        output.size()
    );

    if (output != expected)
    {
        std::cerr << "Unexpected scalar f32 array result.\n";
        return 1;
    }

    std::cout << "Scalar f32 array add passed.\n";

    std::array<float, length> avx_output{};

    whitebase_asm_add_f32_avx(
        lhs.data(),
        rhs.data(),
        avx_output.data(),
        avx_output.size()
    );

    if (avx_output != expected)
    {
        std::cerr << "Unexpected AVX f32 array result.\n";
        return 1;
    }

    std::cout << "AVX f32 array add passed.\n";

    constexpr std::size_t f64_length = 6;

    const std::array<double, f64_length> f64_lhs{
        0.1, 1.0, 2.0, 3.0, 4.0, 5.0
    };

    const std::array<double, f64_length> f64_rhs{
        0.2, 10.0, 20.0, 30.0, 40.0, 50.0
    };

    const std::array<std::uint64_t, f64_length> f64_expected_bits{
        0x3fd3333333333334ULL,
        std::bit_cast<std::uint64_t>(11.0),
        std::bit_cast<std::uint64_t>(22.0),
        std::bit_cast<std::uint64_t>(33.0),
        std::bit_cast<std::uint64_t>(44.0),
        std::bit_cast<std::uint64_t>(55.0)
    };

    std::array<double, f64_length> f64_scalar_output{};

    whitebase_asm_add_f64_array_scalar(
        f64_lhs.data(),
        f64_rhs.data(),
        f64_scalar_output.data(),
        f64_scalar_output.size()
    );

    for (std::size_t index = 0; index < f64_length; ++index)
    {
        if (std::bit_cast<std::uint64_t>(f64_scalar_output[index])
            != f64_expected_bits[index])
        {
            std::cerr << "Unexpected scalar f64 array result.\n";
            return 1;
        }
    }

    std::cout << "Scalar f64 array add passed.\n";

    std::array<double, f64_length> f64_avx_output{};

    whitebase_asm_add_f64_array_avx(
        f64_lhs.data(),
        f64_rhs.data(),
        f64_avx_output.data(),
        f64_avx_output.size()
    );

    if (f64_avx_output != f64_scalar_output)
    {
        std::cerr << "Unexpected AVX f64 array result.\n";
        return 1;
    }

    std::cout << "AVX f64 array add passed.\n";

    return 0;
}