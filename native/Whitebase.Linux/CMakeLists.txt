cmake_minimum_required(VERSION 3.20)

project(WhitebaseLinuxNative LANGUAGES CXX ASM_NASM)

if(NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
    message(FATAL_ERROR "Whitebase.Linux supports Linux only.")
endif()

if(NOT CMAKE_SYSTEM_PROCESSOR MATCHES "^(x86_64|amd64|AMD64)$")
    message(FATAL_ERROR "Whitebase.Linux currently supports x86_64 only.")
endif()

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
set(CMAKE_ASM_NASM_OBJECT_FORMAT elf64)

add_library(whitebase_cpp_backend STATIC
    cpp/whitebase_cpp_scalar.cpp
)

target_include_directories(whitebase_cpp_backend
    PUBLIC
        "${CMAKE_CURRENT_SOURCE_DIR}/include"
)

target_compile_options(whitebase_cpp_backend
    PRIVATE
        -Wall
        -Wextra
        -Wpedantic
)

if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    target_compile_options(whitebase_cpp_backend
        PRIVATE
            -fno-tree-vectorize
            -fno-tree-slp-vectorize
    )
elseif(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    target_compile_options(whitebase_cpp_backend
        PRIVATE
            -fno-vectorize
            -fno-slp-vectorize
    )
endif()

set_target_properties(whitebase_cpp_backend PROPERTIES
    OUTPUT_NAME whitebase_cpp_backend
)

add_library(whitebase_assembly STATIC
    assembly/whitebase_asm_scalar.asm
)

set_target_properties(whitebase_assembly PROPERTIES
    OUTPUT_NAME whitebase_assembly
)

add_executable(whitebase_linux_native_smoke
    tests/native_smoke.cpp
)

target_include_directories(whitebase_linux_native_smoke
    PRIVATE
        "${CMAKE_CURRENT_SOURCE_DIR}/include"
)

target_link_libraries(whitebase_linux_native_smoke
    PRIVATE
        whitebase_cpp_backend
        whitebase_assembly
)

enable_testing()

add_test(
    NAME whitebase_linux_native_smoke
    COMMAND whitebase_linux_native_smoke
)
