#include <array>
#include <iostream>

#include "whitebase_cpp_backend.h"

int main()
{
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

    whitebase::cpp_backend::add_f32_scalar(
        lhs.data(),
        rhs.data(),
        output.data(),
        output.size()
    );

    if (output != expected)
    {
        std::cerr << "C++ Scalar f32 array add failed.\n";
        return 1;
    }

    std::cout << "C++ Scalar f32 array add passed.\n";

    std::array<float, length> avx_output{};

    const bool avx_executed =
        whitebase::cpp_backend::add_f32_avx(
            lhs.data(),
            rhs.data(),
            avx_output.data(),
            avx_output.size()
        );

    if (!avx_executed)
    {
        std::cout << "C++ AVX is unavailable; test skipped.\n";
        return 0;
    }

    if (avx_output != expected || avx_output != output)
    {
        std::cerr << "C++ AVX f32 array add failed.\n";
        return 1;
    }

    std::cout << "C++ AVX f32 array add passed.\n";

    return 0;
}