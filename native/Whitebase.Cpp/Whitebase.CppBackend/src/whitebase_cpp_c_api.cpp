#include "whitebase_cpp_backend.h"
#include "whitebase_cpp_backend_c.h"

extern "C"
{
    void whitebase_cpp_add_f32_scalar(
        const float* lhs,
        const float* rhs,
        float* output,
        const size_t length
    )
    {
        whitebase::cpp_backend::add_f32_scalar(
            lhs,
            rhs,
            output,
            length
        );
    }

    double whitebase_cpp_add_f64_scalar(const double lhs, const double rhs)
    {
        return whitebase::cpp_backend::add_f64_scalar(lhs, rhs);
    }

    int whitebase_cpp_is_avx_available(void)
    {
        return whitebase::cpp_backend::is_avx_available()
            ? 1
            : 0;
    }

    int whitebase_cpp_add_f32_avx(
        const float* lhs,
        const float* rhs,
        float* output,
        const size_t length
    )
    {
        return whitebase::cpp_backend::add_f32_avx(
            lhs,
            rhs,
            output,
            length
        )
            ? 1
            : 0;
    }
}