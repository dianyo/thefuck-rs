mod bash;
mod detect;
mod fish;
mod output;
mod zsh;

pub use bash::Bash;
pub use detect::{detect_shell, Shell, ShellType};
pub use fish::Fish;
pub use output::{get_output, get_raw_command_from_history};
pub use zsh::Zsh;

use crate::config::Settings;
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

/// Creates a shell implementation for the given shell type.
pub fn create_shell(shell_type: ShellType, settings: Settings) -> Box<dyn ShellOperations> {
    match shell_type {
        ShellType::Bash => Box::new(Bash::new(settings)),
        ShellType::Zsh => Box::new(Zsh::new(settings)),
        ShellType::Fish => Box::new(Fish::new(settings)),
        // For unsupported shells, default to Bash-like behavior
        _ => Box::new(Bash::new(settings)),
    }
}

/// Gets the current shell based on detection and creates the appropriate implementation.
pub fn get_current_shell(settings: Settings) -> Result<Box<dyn ShellOperations>> {
    let detected = detect_shell()?;
    Ok(create_shell(detected.shell_type, settings))
}

/// Shell builtin commands that are available in most shells.
pub const BUILTIN_COMMANDS: &[&str] = &[
    "alias", "bg", "bind", "break", "builtin", "case", "cd", "command", "compgen", "complete",
    "continue", "declare", "dirs", "disown", "echo", "enable", "eval", "exec", "exit", "export",
    "fc", "fg", "getopts", "hash", "help", "history", "if", "jobs", "kill", "let", "local",
    "logout", "popd", "printf", "pushd", "pwd", "read", "readonly", "return", "set", "shift",
    "shopt", "source", "suspend", "test", "times", "trap", "type", "typeset", "ulimit", "umask",
    "unalias", "unset", "until", "wait", "while",
];

/// Checks if a command is a shell builtin.
pub fn is_builtin(command: &str) -> bool {
    BUILTIN_COMMANDS.contains(&command)
}
