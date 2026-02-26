# Contributing to thefuck-rs

Thank you for your interest in contributing to thefuck-rs!

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/thefuck-rs
   cd thefuck-rs
   ```
3. Create a branch for your changes:
   ```bash
   git checkout -b feature/my-feature
   ```

## Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run tests
cargo test

# Run with debug output
cargo run -- -d -f "your test command"
```

## Adding a New Rule

1. Create a new file in `src/rules/`:
   ```rust
   // src/rules/my_rule.rs
   use crate::types::{Command, Rule};

   pub struct MyRule;

   impl MyRule {
       pub fn new() -> Self {
           Self
       }
   }

   impl Rule for MyRule {
       fn name(&self) -> &str {
           "my_rule"
       }

       fn matches(&self, command: &Command) -> bool {
           // Your matching logic
           false
       }

       fn get_new_command(&self, command: &Command) -> Vec<String> {
           // Your correction logic
           vec![]
       }
   }
   ```

2. Add tests for your rule:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_my_rule_matches() {
           let rule = MyRule::new();
           let cmd = Command::new("test command", Some("error output".to_string()));
           assert!(rule.matches(&cmd));
       }
   }
   ```

3. Register the rule in `src/rules/mod.rs`:
   ```rust
   pub mod my_rule;
   pub use my_rule::MyRule;

   // In get_builtin_rules():
   Box::new(MyRule::new()),
   ```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix any warnings
- Add tests for new features
- Keep functions small and focused

## Pull Request Process

1. Update the README.md if needed
2. Ensure all tests pass: `cargo test`
3. Ensure no clippy warnings: `cargo clippy`
4. Update the rule count in documentation if adding rules
5. Write a clear PR description

## Reporting Issues

When reporting bugs, please include:

- Your operating system and shell
- The command that failed
- The error output
- Debug output: `thefuck -d -f "your command"`

## Questions?

Feel free to open an issue for questions or discussions.
