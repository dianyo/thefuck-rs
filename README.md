# thefuck-rs

A Rust port of [thefuck](https://github.com/nvbn/thefuck) - the magnificent app which corrects your previous console command.

## Why Rust?

This is a learning project to:
1. Understand the architecture of the original thefuck project
2. Achieve faster startup and execution times
3. Learn Rust while building something practical

## Performance Goals

| Metric | Python Original | Rust Target |
|--------|-----------------|-------------|
| Startup time | ~200-500ms | ~10-50ms |
| Rule loading | ~100-200ms | ~1-5ms |
| Memory usage | ~30-50MB | ~5-10MB |

## Installation

### From source

```bash
cargo install --path .
```

### Setup alias

Add to your shell configuration:

**Bash (~/.bashrc)**:
```bash
eval "$(thefuck alias)"
```

**Zsh (~/.zshrc)**:
```zsh
eval "$(thefuck alias)"
```

**Fish (~/.config/fish/config.fish)**:
```fish
thefuck alias | source
```

## Usage

```bash
# After a failed command
$ git psuh origin main
git: 'psuh' is not a git command. See 'git --help'.

$ fuck
git push origin main  # Corrected!
```

## Development

### Building

```bash
cargo build
cargo build --release
```

### Running tests

```bash
cargo test
```

### Running benchmarks

```bash
cargo bench
```

### Project Structure

```
thefuck-rs/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── main.rs         # CLI binary
│   ├── types.rs        # Core types (Command, Rule, CorrectedCommand)
│   ├── error.rs        # Error types
│   └── shell/          # Shell detection and integration
│       ├── mod.rs
│       └── detect.rs
├── benches/
│   └── benchmark.rs    # Performance benchmarks
└── Cargo.toml
```

## Conversion Progress

- [x] Phase 1: Project setup and core types
- [ ] Phase 2: Configuration system
- [ ] Phase 3: Shell integration
- [ ] Phase 4: Rule system infrastructure
- [ ] Phase 5: Core rules (sudo, no_command, git_push, etc.)
- [ ] Phase 6: UI and user interaction
- [ ] Phase 7: Performance benchmarking
- [ ] Phase 8: Extended rules
- [ ] Phase 9: Advanced features
- [ ] Phase 10: Documentation and release

## License

MIT (same as original thefuck)

## Credits

Original project by [@nvbn](https://github.com/nvbn/thefuck)
