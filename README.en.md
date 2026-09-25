[![CI](https://github.com/Hokutaka/Whitebase/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/Hokutaka/Whitebase/actions/workflows/main.yml)
[![CodeQL](https://github.com/Hokutaka/Whitebase/actions/workflows/github-code-scanning/codeql/badge.svg?branch=main)](https://github.com/Hokutaka/Whitebase/actions/workflows/github-code-scanning/codeql)

[日本語](README.md) | **English**

A repository for learning and experimentation.

# Whitebase
Whitebase is a place for building and experimenting with foundational implementations.

It is organized as a monorepo centered around Whitebase Core.

The repository keeps consumer-facing functionality minimal, focusing on execution results, charts, and other visualizations.
Concrete applications and use-case-specific implementations are developed in separate projects.

Whitebase currently focuses on small `f32` and `f64` compute kernels implemented through Rust, C++, Assembly, and Cerune, including scalar, SIMD, and VM-based execution paths.

This is an experimental project. Use it at your own risk.

For more details, see the sections below.

## Documentation

- [Detailed documentation](/docs//Overview.en.md)
- [Project structure](docs/project-tree.md)
- [Layer Overview(Ja)](docs/Layer/Layer-Overview.en.md)

### Operation Tools

| Name | Implementation |
| --- | --- |
| [Whitebase Operations](docs/tools/Whitebase%20Operations.md) | Windows Batch / Linux Native Shell |
| [Whitebase Control Center](docs/tools/Whitebase%20Control%20Center.md) | Rust + egui / Windows and Linux |
| [Whitebase Control Panel](docs/tools/Whitebase%20Control%20Panel.md) | C# + WPF / Windows |

## License

MIT License