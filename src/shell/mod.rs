mod detect;

pub use detect::{detect_shell, Shell, ShellType};

use crate::error::Result;

/// Trait for shell-specific operations.
///
/// Different shells have different ways to:
/// - Generate aliases
/// - Get command history
/// - Quote commands
/// - Expand aliases
pub trait ShellOperations {
    /// Returns the shell type.
    fn shell_type(&self) -> ShellType;

    /// Generates the shell alias/function for invoking thefuck.
    fn app_alias(&self, alias_name: &str) -> String;

    /// Quotes a command for safe execution in this shell.
    fn quote(&self, command: &str) -> String;

    /// Gets the last N commands from shell history.
    fn get_history(&self, limit: usize) -> Result<Vec<String>>;

    /// Expands aliases in the given command.
    fn expand_aliases(&self, command: &str) -> String {
        // Default implementation: no expansion
        command.to_string()
    }

    /// Adds a command to shell history.
    fn put_to_history(&self, command: &str) -> Result<()>;

    /// Combines two commands with OR (||) for this shell.
    fn or_commands(&self, first: &str, second: &str) -> String {
        format!("{} || {}", first, second)
    }

    /// Combines two commands with AND (&&) for this shell.
    fn and_commands(&self, first: &str, second: &str) -> String {
        format!("{} && {}", first, second)
    }
}
