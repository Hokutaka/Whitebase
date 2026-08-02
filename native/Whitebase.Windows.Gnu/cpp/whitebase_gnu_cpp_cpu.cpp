#include "whitebase_windows_gnu_native.h"

#include <cpuid.h>
#include <cstdint>

namespace
{
    bool cpu_and_os_support_avx() noexcept
    {
        unsigned int eax = 0;
        unsigned int ebx = 0;
        unsigned int ecx = 0;
        unsigned int edx = 0;

        if (__get_cpuid(1, &eax, &ebx, &ecx, &edx) == 0)
        {
            return false;
        }

        constexpr unsigned int osxsave_bit = 1U << 27;
        constexpr unsigned int avx_bit = 1U << 28;
        if ((ecx & osxsave_bit) == 0 || (ecx & avx_bit) == 0)
        {
            return false;
        }

        std::uint32_t xcr0_low = 0;
        std::uint32_t xcr0_high = 0;

        __asm__ volatile(
            "xgetbv"
            : "=a"(xcr0_low), "=d"(xcr0_high)
            : "c"(0)
        );

        const std::uint64_t xcr0 =
            (static_cast<std::uint64_t>(xcr0_high) << 32)
            | xcr0_low;

        constexpr std::uint64_t xmm_ymm_state = UINT64_C(0x6);
        return (xcr0 & xmm_ymm_state) == xmm_ymm_state;
    }
}

extern "C" int whitebase_gnu_cpp_is_avx_available(void)
{
    static const int available = cpu_and_os_support_avx() ? 1 : 0;
    return available;
}
