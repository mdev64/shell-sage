# Contributing to Shell Sage

Thanks for your interest in contributing! Shell Sage is an offline AI CLI
assistant for Apple Silicon Macs. This guide covers how to set up the project,
run the checks, and submit changes.

## Prerequisites

- An **Apple Silicon Mac** (M1/M2/M3/M4 or later)
- [Rust](https://www.rust-lang.org/tools/install) (edition 2024, Rust 1.85+)
- Xcode Command Line Tools (`xcode-select --install`)

## Getting started

```sh
git clone https://github.com/mdev64/shell-sage.git
cd shell-sage
cargo build
```

## Project structure

```
src/
  main.rs       CLI parsing, mode dispatch, output rendering
  lib.rs        Crate root (public modules)
  config.rs     Model config, app settings, cache paths
  downloader.rs Model download, verification, and cleanup
  engine.rs     Llama model loading and inference
  output.rs     Parse + sanitize the model's raw output
  prompt.rs     Prompt templates for each mode
  setup.rs      The `sage --config` interactive wizard
tests/
  *.rs          Integration tests (one per source module)
```

## Development workflow

- Build: `cargo build`
- Run: `cargo run -- "your query"`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets`
- Format: `cargo fmt`

Before submitting changes, please make sure the following all pass:

```sh
cargo fmt --check
cargo clippy --all-targets
cargo test
```

## Pull request process

1. Open an issue describing the bug or feature first.
2. Create a branch from `main`.
3. Make focused, minimal changes.
4. Add or update tests for any changed behavior.
5. Run the checks listed above.
6. Open a pull request with a clear description of what changed and why.

## Reporting bugs

When reporting a bug, please include:

- Your macOS version and Apple Silicon chip (e.g. macOS 15, M3)
- The exact command you ran
- The full terminal output
- Your config (`~/.cache/sage/config.json`), with any personal info removed

## License

By contributing, you agree that your contributions will be licensed under the
[MIT License](LICENSE).
