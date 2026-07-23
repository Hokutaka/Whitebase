#include <iostream>

#include "whitebase.h"

int main()
{
    constexpr int left = 2;
    constexpr int right = 3;

    const int result =
        whitebase_add(left, right);

    std::cout
        << "Whitebase C ABI smoke test\n"
        << left
        << " + "
        << right
        << " = "
        << result
        << '\n';

    if (result != 5)
    {
        std::cerr << "Unexpected result.\n";
        return 1;
    }

    return 0;
}