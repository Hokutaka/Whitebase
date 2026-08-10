[![CI](https://github.com/Hokutaka/Whitebase/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/Hokutaka/Whitebase/actions/workflows/main.yml)
[![CodeQL](https://github.com/Hokutaka/Whitebase/actions/workflows/github-code-scanning/codeql/badge.svg?branch=main)](https://github.com/Hokutaka/Whitebase/actions/workflows/github-code-scanning/codeql)

**English** | [日本語](README.ja.md)

A repository for learning and experimentation.

# Whitebase
Whitebase is a place for building and experimenting with various foundational implementations.

It is organized as a monorepo centered around Core.
Consumer-facing functionality is limited to displaying execution results, charts, and other visualizations.
Concrete applications and use-case-specific implementations are kept in separate projects.

Currently, it focuses on array operations for f32 and f64 using Rust, C++, and Assembly.
Use it at your own risk.

For more details, see the sections below.

## Documentation

- [Detailed documentation](/docs//Overview.md)
- [Project structure](docs/project-tree.md)
- [Layer Overview(Ja)](docs/Layer/Layer-Overview.md)

### Operation Tools

| Name | Implementation |
| --- | --- |
| [Whitebase Operations](docs/tools/Whitebase%20Operations.md) | Windows Batch / Linux Native Shell |
| [Whitebase Control Center](docs/tools/Whitebase%20Control%20Center.md) | Rust + egui / Windows and Linux |
| [Whitebase Control Panel](docs/tools/Whitebase%20Control%20Panel.md) | C# + WPF / Windows |

## License

MIT License