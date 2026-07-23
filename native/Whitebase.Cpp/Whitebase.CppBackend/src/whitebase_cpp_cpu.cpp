#include "whitebase_cpp_backend.h"

#include <intrin.h>

namespace whitebase::cpp_backend
{
    bool is_avx_available() noexcept
    {
        int registers[4]{};
        __cpuid(registers, 1);

        constexpr int osxsave_bit = 1 << 27;
        constexpr int avx_bit = 1 << 28;

        const bool osxsave_supported =
            (registers[2] & osxsave_bit) != 0;

        const bool avx_supported =
            (registers[2] & avx_bit) != 0;

        if (!osxsave_supported || !avx_supported)
        {
            return false;
        }

        const unsigned __int64 xcr0 = _xgetbv(0);

        constexpr unsigned __int64 xmm_ymm_state = 0x6;

        return (xcr0 & xmm_ymm_state) == xmm_ymm_state;
    }
}