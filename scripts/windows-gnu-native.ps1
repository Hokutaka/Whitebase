param(
    [Parameter(Position = 0)]
    [ValidateSet("build", "check", "release", "clean", "help")]
    [string]$Command = "help"
)

$ErrorActionPreference = "Stop"

$ScriptDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepositoryRoot = Split-Path -Parent $ScriptDirectory
$SourceDirectory = Join-Path $RepositoryRoot "native\Whitebase.Windows.Gnu"

$Ucrt64Bin = if ($env:WHITEBASE_UCRT64_BIN) {
    $env:WHITEBASE_UCRT64_BIN
}
else {
    "C:\msys64\ucrt64\bin"
}

$Ucrt64Compiler = Join-Path $Ucrt64Bin "g++.exe"

if (Test-Path $Ucrt64Compiler) {
    $remainingPath = @(
        $env:Path -split ";" |
            Where-Object {
                $_ -and
                $_.TrimEnd("\") -ine $Ucrt64Bin.TrimEnd("\")
            }
    )

    $env:Path = (@($Ucrt64Bin) + $remainingPath) -join ";"
}

function Show-Usage {
    @"
Usage: .\scripts\windows-gnu-native.ps1 <command>
Commands:
  build    Configure and build the Debug native library.
  check    Build Debug and run the native smoke test.
  release  Build Release and run the native smoke test.
  clean    Remove Windows GNU native build outputs.
"@ | Write-Host
}

function Assert-CommandAvailable([string]$Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command was not found: $Name"
    }
}

function Confirm-Environment {
    if ($env:OS -ne "Windows_NT") {
        throw "Whitebase.Windows.Gnu requires Windows."
    }

    foreach ($commandName in @("cmake", "ctest", "ninja", "g++", "nasm")) {
        Assert-CommandAvailable $commandName
    }

    $target = (& g++ -dumpmachine).Trim()

    if ($LASTEXITCODE -ne 0) {
        throw "Failed to query the GCC target."
    }

    if ($target -ne "x86_64-w64-mingw32") {
        throw "Whitebase.Windows.Gnu requires x86_64 MinGW-w64 GCC. Found: $target"
    }
}

function Configure-And-Build([string]$Configuration) {
    $buildDirectory = Join-Path $SourceDirectory "build\$Configuration"

    Write-Host "[Whitebase Windows GNU Native] Configure $Configuration"
    & cmake `
        -S $SourceDirectory `
        -B $buildDirectory `
        -G Ninja `
        "-DCMAKE_BUILD_TYPE=$Configuration" `
        "-DCMAKE_CXX_COMPILER=g++.exe" `
        "-DCMAKE_ASM_NASM_COMPILER=nasm.exe"

    if ($LASTEXITCODE -ne 0) {
        throw "CMake configuration failed."
    }

    Write-Host "[Whitebase Windows GNU Native] Build $Configuration"
    & cmake --build $buildDirectory --parallel

    if ($LASTEXITCODE -ne 0) {
        throw "Native build failed."
    }
}

function Run-Tests([string]$Configuration) {
    $buildDirectory = Join-Path $SourceDirectory "build\$Configuration"
    $smokeExecutable = Join-Path $buildDirectory "whitebase_windows_gnu_native_smoke.exe"

    Write-Host "[Whitebase Windows GNU Native] Test $Configuration"
    & ctest --test-dir $buildDirectory --output-on-failure

    if ($LASTEXITCODE -ne 0) {
        throw "CTest failed."
    }

    Write-Host "[Whitebase Windows GNU Native] Backend status $Configuration"
    & $smokeExecutable

    if ($LASTEXITCODE -ne 0) {
        throw "Native smoke test failed."
    }
}

switch ($Command) {
    "build" {
        Confirm-Environment
        Configure-And-Build "Debug"
    }
    "check" {
        Confirm-Environment
        Configure-And-Build "Debug"
        Run-Tests "Debug"
    }
    "release" {
        Confirm-Environment
        Configure-And-Build "Release"
        Run-Tests "Release"
    }
    "clean" {
        $buildDirectory = Join-Path $SourceDirectory "build"
        Remove-Item -Recurse -Force $buildDirectory -ErrorAction SilentlyContinue
        Write-Host "[Whitebase Windows GNU Native] Removed build outputs."
    }
    "help" {
        Show-Usage
    }
}
