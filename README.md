# thefuck-rs

A blazingly fast Rust implementation of [thefuck](https://github.com/nvbn/thefuck) - the magnificent app that corrects your previous console command.

## Features

- **100x faster** than the Python version (~3ms vs ~300ms startup)
- **19 built-in rules** covering common mistakes
- **User-defined rules** via TOML configuration
- **Interactive selection** with arrow keys or vim-style navigation
- **Shell integration** for Bash, Zsh, and Fish

## Installation

### From Source

```bash
git clone https://github.com/your-username/thefuck-rs
cd thefuck-rs
cargo install --path .
```

### Pre-built Binaries

Download from the [releases page](https://github.com/your-username/thefuck-rs/releases).

## Quick Start

1. Initialize configuration:
   ```bash
   thefuck init
   ```

2. Add to your shell config:

   **Bash** (~/.bashrc):
   ```bash
   eval "$(thefuck alias)"
   ```

   **Zsh** (~/.zshrc):
   ```bash
   eval "$(thefuck alias)"
   ```

   **Fish** (~/.config/fish/config.fish):
   ```fish
   thefuck alias | source
   ```

3. Restart your shell or source the config:
   ```bash
   source ~/.bashrc  # or ~/.zshrc
   ```

4. Make a mistake and type `fuck`:
   ```bash
   $ git psuh origin main
   git: 'psuh' is not a git command...
   
   $ fuck
   git push origin main [enter/↑/↓/ctrl+c]
   ```

## Usage

```bash
# Fix the last command interactively
fuck

# Fix without confirmation
fuck -y

# Fix a specific command
thefuck -f "git psuh"

# Show available commands
thefuck --help
```

## Configuration

Configuration is stored in `~/.config/thefuck/settings.toml` (or `~/Library/Application Support/thefuck/settings.toml` on macOS).

```toml
# Wait time for slow commands (seconds)
wait_command = 3

# Whether to require confirmation before running
require_confirmation = true

# Rules to exclude
exclude_rules = []

# Custom priorities
[priority]
sudo = 500
```

### Environment Variables

- `TF_ALIAS` - Custom alias name (default: fuck)
- `TF_HISTORY` - Command history (set by shell alias)
- `TF_SHELL` - Override detected shell
- `THEFUCK_DEBUG` - Enable debug output
- `THEFUCK_NO_COLORS` - Disable colored output

## Built-in Rules

| Rule | Description |
|------|-------------|
| `sudo` | Prepend `sudo` when permission denied |
| `git_push` | Fix `git push` upstream errors |
| `git_not_command` | Fix git command typos |
| `git_add` | Add missing files before commit |
| `git_stash` | Help with stash operations |
| `no_command` | Suggest similar commands |
| `cd_parent` | Fix `cd..` → `cd ..` |
| `cd_mkdir` | Create directory and cd into it |
| `mkdir_p` | Add `-p` flag for nested directories |
| `rm_dir` | Add `-r` flag for directories |
| `cp_omitting_directory` | Add `-r` flag for directories |
| `cat_dir` | Replace `cat` with `ls` for directories |
| `chmod_x` | Add execute permission for scripts |
| `touch` | Create parent directories |
| `cargo_no_command` | Fix cargo subcommand typos |
| `python_command` | Add `python` prefix for .py files |
| `man_no_space` | Fix `mangit` → `man git` |
| `ls_la` | Fix `ls` typos |
| `open` | Replace `open` with `xdg-open` on Linux |

## User-Defined Rules

Create custom rules in `~/.config/thefuck/rules/`:

```toml
# my_rule.toml
name = "my_custom_rule"
enabled = true
priority = 1000

# Regex pattern to match command
match_script = "^myapp (.+)$"

# Optional: Also match output
match_output = "error: invalid option"

# Fixed replacement
new_command = "myapp --fixed"

# Or pattern-based (with capture groups)
new_command_pattern = "myapp --correct $1"
```

## Performance

| Metric | Rust | Python | Improvement |
|--------|------|--------|-------------|
| Startup | ~3ms | ~300ms | 100x |
| Correction | ~340µs | ~50-200ms | 150-600x |
| Memory | ~2MB | ~30-50MB | 15-25x |

Run benchmarks:
```bash
cargo bench
./benchmark_comparison.sh
```

## Development

```bash
# Run tests
cargo test

# Run with debug output
cargo run -- -d -f "git psuh"

# Build release binary
cargo build --release
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [nvbn/thefuck](https://github.com/nvbn/thefuck) - The original Python implementation
- All contributors to the original project
