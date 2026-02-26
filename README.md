# 🦀 thefuck-rs

> **Too slow for the fuck? Try fuck in Rust!** ⚡

A **blazingly fast** 🔥 Rust implementation of [thefuck](https://github.com/nvbn/thefuck) - the magnificent app that corrects your previous console command.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ⚡ Why thefuck-rs?

| | Rust 🦀 | Python 🐍 | Winner |
|---|---------|-----------|--------|
| **Startup** | 2.7 ms | 151 ms | **56x faster** 🚀 |
| **Correction** | 112 ms | 233 ms | **2x faster** ⚡ |
| **Memory** | 1.8 MB | 36 MB | **20x less** 💾 |

**Your typos deserve instant fuck, not fuck with a coffee sip.** ☕→⚡

## ✨ Features

- 🚀 **Instant startup** - Ready before you blink
- 🧠 **19 built-in rules** - Covers common mistakes
- 🐚 **Shell support** - Bash, Zsh, Fish

## 📦 Installation

### From Source

```bash
git clone https://github.com/your-username/thefuck-rs
cd thefuck-rs
cargo install --path .
```

## 🚀 Quick Start

**1. Initialize:**
```bash
thefuck init
```

**2. Add to your shell** (~/.zshrc or ~/.bashrc):
```bash
eval "$(thefuck alias)"
```

**3. Restart shell and make mistakes:**
```bash
$ git psuh origin main
git: 'psuh' is not a git command...

$ fuck
git push origin main [enter/↑/↓/ctrl+c]
```

## 🎮 Usage

```bash
fuck              # Fix the last command (interactive)
fuck -y           # Fix without confirmation (YOLO mode 🎲)
thefuck -f "cmd"  # Fix a specific command
thefuck --help    # Show all options
```

## 🏎️ Benchmarks

See [BENCHMARKS.md](BENCHMARKS.md) for detailed performance comparison.

```
Startup:    Rust ██ 2.7ms   vs   Python ████████████████████████ 151ms
Memory:     Rust ██ 1.8MB   vs   Python ████████████████████████ 36MB
```

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
cargo test      # Run tests
cargo bench     # Run benchmarks
cargo clippy    # Lint
```

## 📄 License

MIT License - see [LICENSE](LICENSE)

## 🙏 Credits

- [nvbn/thefuck](https://github.com/nvbn/thefuck) - The original masterpiece 🐍
- All contributors to both projects ❤️

---

<p align="center">
  <b>Stop waiting. Start fucking (your typos). 🦀⚡</b>
</p>
