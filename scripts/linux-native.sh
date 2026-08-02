#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPOSITORY_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
SOURCE_DIR="${REPOSITORY_ROOT}/native/Whitebase.Linux"
CXX_COMPILER="${CXX:-g++}"

usage() {
    cat <<'EOF'
Usage: ./scripts/linux-native.sh <command>

Commands:
  build    Configure and build the Debug native libraries.
  check    Build Debug and run the native smoke test.
  release  Build Release and run the native smoke test.
  clean    Remove Linux native build outputs.
EOF
}

require_command() {
    local command_name="$1"

    if ! command -v "${command_name}" >/dev/null 2>&1; then
        echo "[FAIL] Required command was not found: ${command_name}" >&2
        exit 1
    fi
}

check_environment() {
    if [[ "$(uname -s)" != "Linux" ]]; then
        echo "[FAIL] Whitebase.Linux requires Linux." >&2
        exit 1
    fi

    if [[ "$(uname -m)" != "x86_64" ]]; then
        echo "[FAIL] Whitebase.Linux currently requires x86_64." >&2
        exit 1
    fi

    require_command cmake
    require_command ctest
    require_command nasm
    require_command "${CXX_COMPILER}"
}

configure_and_build() {
    local configuration="$1"
    local build_dir="${SOURCE_DIR}/build/${configuration}"

    echo "[Whitebase Linux Native] Configure ${configuration}"
    cmake \
        -S "${SOURCE_DIR}" \
        -B "${build_dir}" \
        -DCMAKE_BUILD_TYPE="${configuration}" \
        -DCMAKE_CXX_COMPILER="${CXX_COMPILER}"

    echo "[Whitebase Linux Native] Build ${configuration}"
    cmake --build "${build_dir}" --parallel
}

run_tests() {
    local configuration="$1"
    local build_dir="${SOURCE_DIR}/build/${configuration}"

    echo "[Whitebase Linux Native] Test ${configuration}"
    ctest --test-dir "${build_dir}" --output-on-failure

    echo "[Whitebase Linux Native] Backend status ${configuration}"
    "${build_dir}/whitebase_linux_native_smoke"
}

main() {
    local command="${1:-}"

    case "${command}" in
        build)
            check_environment
            configure_and_build Debug
            ;;
        check)
            check_environment
            configure_and_build Debug
            run_tests Debug
            ;;
        release)
            check_environment
            configure_and_build Release
            run_tests Release
            ;;
        clean)
            rm -rf "${SOURCE_DIR}/build"
            echo "[Whitebase Linux Native] Removed build outputs."
            ;;
        -h|--help|help)
            usage
            ;;
        *)
            usage >&2
            exit 2
            ;;
    esac
}

main "$@"
