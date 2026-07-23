#include <array>
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

    return 0;
}