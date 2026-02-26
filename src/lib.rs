//! # thefuck-rs
//!
//! A Rust port of [thefuck](https://github.com/nvbn/thefuck) - the magnificent app
//! which corrects your previous console command.
//!
//! ## Overview
//!
//! This library provides the core functionality for detecting and correcting
//! mistyped console commands. It includes:
//!
//! - Command parsing and representation
//! - Shell detection and integration
//! - A rule-based correction system
//! - Configuration management
//!
//! ## Example
//!
//! ```rust
//! use thefuck::types::Command;
//!
//! let mut cmd = Command::new(
//!     "git psuh origin main",
//!     Some("git: 'psuh' is not a git command.".to_string())
//! );
//!
//! // Get the command parts
//! let parts = cmd.script_parts();
//! assert_eq!(parts[0], "git");
//! ```

pub mod config;
pub mod corrector;
pub mod error;
pub mod executor;
pub mod rules;
pub mod shell;
pub mod types;
pub mod ui;
pub mod user_rules;

// Re-export commonly used types
pub use config::Settings;
pub use corrector::Corrector;
pub use error::{Result, TheFuckError};
pub use executor::{command_exists, execute_command, execute_command_capture};
pub use rules::get_builtin_rules;
pub use types::{Command, CorrectedCommand, Rule, RuleInfo, DEFAULT_PRIORITY};
pub use ui::select_command;
pub use user_rules::load_user_rules;
