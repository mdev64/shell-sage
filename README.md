# Shell Sage

```text
███████╗██╗  ██╗███████╗██╗     ██╗         ███████╗ █████╗  ██████╗ ███████╗
██╔════╝██║  ██║██╔════╝██║     ██║         ██╔════╝██╔══██╗██╔════╝ ██╔════╝
███████╗███████║█████╗  ██║     ██║         ███████╗███████║██║  ███╗█████╗
╚════██║██╔══██║██╔══╝  ██║     ██║         ╚════██║██╔══██║██║   ██║██╔══╝
███████║██║  ██║███████╗███████╗███████╗    ███████║██║  ██║╚██████╔╝███████╗
╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝    ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝
```

> An offline AI CLI assistant for **Apple Silicon Macs** that turns natural language into shell commands — running a Gemma 4 model entirely on-device.

## What is Shell Sage?

Shell Sage is a command-line assistant that runs a Gemma 4 model **locally on
your Mac** using the Metal GPU. Describe what you want in plain English and it
prints the exact shell command to run. It can also explain terminal commands
and answer general-knowledge questions.

Shell Sage **never executes anything itself** — it only suggests. You review
the command, paste it, and press Enter.

## Features

- **100% offline** — the model runs locally via Metal; no API keys, no cloud.
- **Generate commands** — `sage "find all mp4 files"` produces a ready-to-run command.
- **Explain commands** — `sage -e "ls -la"` breaks down what a command does.
- **Answer questions** — `sage -a "why is the sky blue?"`.
- **Clipboard support** — optionally copies the command so you just paste and run.
- **Code hints** — explains each flag and option after the generated command.
- **RAM-aware setup** — recommends the right model size for your machine.

## Requirements

- An **Apple Silicon Mac** (M1/M2/M3/M4 or later). Intel Macs are **not** supported.
- macOS with the **Xcode Command Line Tools** installed (needed for the Metal backend).
- [Rust](https://www.rust-lang.org/tools/install) (edition 2024, Rust 1.85+) — only when building from source.

## Installation

### Homebrew

```sh
brew install mdev64/tap/shell-sage
```

### Build from source

```sh
# 1. Install Xcode Command Line Tools (if not already installed)
xcode-select --install

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Clone and build
git clone https://github.com/mdev64/shell-sage.git
cd shell-sage
cargo build --release

# 4. Put the binary on your PATH
cp target/release/sage /usr/local/bin/sage
```

## Quick start

Run the setup wizard on first use. It detects your RAM, recommends a model,
confirms your shell, and downloads the model weights to `~/.cache/sage/`.

```sh
sage --config
```

Then start generating commands:

```sh
sage "find all mp4 files"
```

## Usage

| Command                            | What it does                        |
| ---------------------------------- | ----------------------------------- |
| `sage "list files modified today"` | Generate a shell command            |
| `sage -e "ls -la"`                 | Explain a terminal command          |
| `sage -a "why is the sky blue?"`   | Answer a general-knowledge question |
| `sage --config`                    | Run the configuration wizard        |
| `sage --help`                      | Show help                           |

### Examples

Generate a command:

```sh
$ sage "compress all png files in this directory"

$ find . -name "*.png" -exec pngquant --ext .png {} \;
# find: searches the filesystem for files
# -name "*.png": matches files ending in .png
# -exec ...: runs pngquant on each match
✓ Command copied to clipboard. Press '⌘ + V', then press Enter to run.
```

Explain a command:

```sh
$ sage -e "tar -xzvf archive.tar.gz"
```

Ask a question:

```sh
$ sage -a "why is the sky blue?"
```

## How it works

1. Shell Sage tokenizes your query and feeds it to a local Gemma 4 model (via
   `llama-cpp-2` with the Metal backend).
2. The model returns a raw shell command plus `# `-prefixed hints.
3. Shell Sage prints the command and the hints, and (optionally) copies the
   command to your clipboard.
4. You paste and press Enter — Shell Sage never runs anything itself.

## Configuration

Settings live in `~/.cache/sage/config.json` and can be changed anytime with
`sage --config`:

- **Model** — Gemma 4 **E2B** (faster, lower RAM) or **E4B** (more capable, higher RAM).
- **Shell** — the shell used in generated commands (e.g. `/bin/zsh`).
- **Code hints** — toggle the `# `-prefixed explanations.
- **Clipboard** — toggle auto-copying the command to the clipboard.

## License

MIT — see [LICENSE](LICENSE).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
