# Contributing to Whitebase

日本語版: [CONTRIBUTING.ja.md](./CONTRIBUTING.ja.md)

Thank you for your interest in contributing to Whitebase.

## Development Model

Whitebase uses `dev` as the integration branch and `main` as the stable branch.

```text
working branch
    ↓ Pull Request
dev
    ↓ Pull Request
main
```

Create feature, fix, documentation, CI, and other working branches from the latest `dev` branch.

Do not develop directly on `main` or `dev`.

## Starting Work

Update `dev` before creating a working branch.

```shell
git switch dev
git pull --ff-only origin dev
git switch -c feature/example
```

Use a branch name that reflects the purpose of the change, for example:

- `feature/...` for new functionality
- `fix/...` for bug fixes
- `docs/...` for documentation
- `ci/...` for CI or workflow changes

## Development Environment

The Rust toolchain is managed by `rust-toolchain.toml`.

### Windows

Run the common Windows setup and integrated checks from the repository root:

```powershell
.\scripts\ops.bat setup
.\scripts\ops.bat check
```

Windows GNU Native development additionally requires an MSYS2 UCRT64 environment with MinGW-w64 GCC, CMake, Ninja, and NASM.

### Linux x86_64

Check the Linux GCC/NASM native backends with:

```bash
./scripts/linux-native.sh check
```

For the full command reference, see [Whitebase Operations](./docs/tools/Whitebase%20Operations.md).

## Rust Workspace Checks

Before opening a pull request, run the checks relevant to your change. The standard Rust checks are:

```shell
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

OS-specific changes should also be validated on the affected operating system whenever possible.

## Control Center

The Whitebase Control Center provides integrated check, build, and release operations for the current platform.

```shell
cargo run -p whitebase-control-center
```

The main aggregate operations are:

- `Check All` — run the checks supported on the current platform.
- `Build All` — build the development artifacts supported on the current platform.
- `Release All` — build the release artifacts supported on the current platform.

## Before Committing

Review the working tree before creating a commit:

```shell
git status
git diff --check
git diff --stat
```

Do not commit private keys, access tokens, personal information, temporary patches, archives, or unnecessary generated files.

## Pull Requests

Working branches should normally target `dev`.

Describe the change and the validation performed using the pull request template. For OS-specific changes, clearly state which operating systems were tested.

Required CI, dependency review, code scanning, and review conversations must be resolved before merging when enforced by repository rules.

Squash merge is preferred for working branches.

## Promoting `dev` to `main`

After changes have been integrated and validated on `dev`, open a pull request from `dev` to `main`.

`main` represents the stable branch. Release packaging and GitHub Pages workflows run from `main` according to the repository CI configuration.

## Security

Do not report security vulnerabilities in a public issue.

Follow the private reporting process described in [SECURITY.md](./SECURITY.md).
