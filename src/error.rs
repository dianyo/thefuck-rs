use thiserror::Error;

/// Errors that can occur in thefuck-rs
#[derive(Error, Debug)]
pub enum TheFuckError {
    #[error("Empty command provided")]
    EmptyCommand,

    #[error("Failed to parse command: {0}")]
    CommandParseError(String),

    #[error("Shell not supported: {0}")]
    UnsupportedShell(String),

    #[error("Failed to detect shell")]
    ShellDetectionFailed,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Failed to load config file: {0}")]
    ConfigLoadError(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    ConfigParseError(#[from] toml::de::Error),

    #[error("Rule error in '{rule}': {message}")]
    RuleError { rule: String, message: String },

    #[error("Command execution failed: {0}")]
    ExecutionError(String),

    #[error("Timeout waiting for command output")]
    CommandTimeout,
}

pub type Result<T> = std::result::Result<T, TheFuckError>;
