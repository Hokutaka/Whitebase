cmake_minimum_required(VERSION 3.20)

project(WhitebaseWindowsGnuNative LANGUAGES CXX ASM_NASM)

if(NOT WIN32)
    message(FATAL_ERROR "Whitebase.Windows.Gnu supports Windows only.")
endif()

if(NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
    message(FATAL_ERROR "Whitebase.Windows.Gnu currently supports x86_64 only.")
endif()

if(NOT CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    message(FATAL_ERROR "Whitebase.Windows.Gnu requires GCC/MinGW-w64.")
endif()

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_ASM_NASM_OBJECT_FORMAT win64)

add_library(whitebase_windows_gnu_native SHARED
    cpp/whitebase_gnu_cpp_scalar.cpp
    cpp/whitebase_gnu_cpp_cpu.cpp
    cpp/whitebase_gnu_cpp_avx.cpp
    assembly/whitebase_gnu_asm_scalar.asm
    assembly/whitebase_gnu_asm_avx.asm
    exports/whitebase_windows_gnu_native.def
)

set_source_files_properties(
    cpp/whitebase_gnu_cpp_avx.cpp
    PROPERTIES
        COMPILE_OPTIONS "-mavx"
)

target_include_directories(whitebase_windows_gnu_native
    PUBLIC
        "${CMAKE_CURRENT_SOURCE_DIR}/include"
)

target_compile_options(whitebase_windows_gnu_native
    PRIVATE
        $<$<COMPILE_LANGUAGE:CXX>:-Wall>
        $<$<COMPILE_LANGUAGE:CXX>:-Wextra>
        $<$<COMPILE_LANGUAGE:CXX>:-Wpedantic>
        $<$<COMPILE_LANGUAGE:CXX>:-fno-tree-vectorize>
        $<$<COMPILE_LANGUAGE:CXX>:-fno-tree-slp-vectorize>
)

set_target_properties(whitebase_windows_gnu_native PROPERTIES
    OUTPUT_NAME whitebase_windows_gnu_native
    PREFIX ""
)

add_executable(whitebase_windows_gnu_native_smoke
    tests/native_smoke.cpp
)

target_link_libraries(whitebase_windows_gnu_native_smoke
    PRIVATE
        whitebase_windows_gnu_native
)

enable_testing()

add_test(
    NAME whitebase_windows_gnu_native_smoke
    COMMAND whitebase_windows_gnu_native_smoke
)
