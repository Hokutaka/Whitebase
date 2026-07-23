#pragma once

#if defined(_WIN32)
#define WHITEBASE_API __declspec(dllimport)
#else
#define WHITEBASE_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

    WHITEBASE_API int whitebase_add(
        int left,
        int right);

#ifdef __cplusplus
}
#endif