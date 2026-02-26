use crate::error::{Result, TheFuckError};
use std::env;

/// Supported shell types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Tcsh,
    PowerShell,
    Cmd,
    Unknown,
}

impl ShellType {
    /// Returns the shell type from a shell name string.
    pub fn from_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        let basename = std::path::Path::new(&name_lower)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&name_lower);

        match basename {
            "bash" | "bash.exe" => ShellType::Bash,
            "zsh" | "zsh.exe" => ShellType::Zsh,
            "fish" | "fish.exe" => ShellType::Fish,
            "tcsh" | "csh" => ShellType::Tcsh,
            "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe" => ShellType::PowerShell,
            "cmd" | "cmd.exe" => ShellType::Cmd,
            _ => ShellType::Unknown,
        }
    }

    /// Returns the display name of the shell.
    pub fn name(&self) -> &'static str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Fish => "fish",
            ShellType::Tcsh => "tcsh",
            ShellType::PowerShell => "powershell",
            ShellType::Cmd => "cmd",
            ShellType::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Represents the detected shell with its metadata.
#[derive(Debug, Clone)]
pub struct Shell {
    pub shell_type: ShellType,
    pub path: Option<String>,
}

impl Shell {
    pub fn new(shell_type: ShellType) -> Self {
        Self {
            shell_type,
            path: None,
        }
    }

    pub fn with_path(shell_type: ShellType, path: String) -> Self {
        Self {
            shell_type,
            path: Some(path),
        }
    }
}

/// Detects the current shell.
///
/// Detection priority:
/// 1. TF_SHELL environment variable (set by our alias)
/// 2. SHELL environment variable
/// 3. Process tree walking (TODO: implement with sysinfo crate)
pub fn detect_shell() -> Result<Shell> {
    // 1. Check TF_SHELL (set by our shell alias)
    if let Ok(tf_shell) = env::var("TF_SHELL") {
        let shell_type = ShellType::from_name(&tf_shell);
        if shell_type != ShellType::Unknown {
            return Ok(Shell::with_path(shell_type, tf_shell));
        }
    }

    // 2. Check SHELL environment variable (Unix)
    if let Ok(shell_path) = env::var("SHELL") {
        let shell_type = ShellType::from_name(&shell_path);
        if shell_type != ShellType::Unknown {
            return Ok(Shell::with_path(shell_type, shell_path));
        }
    }

    // 3. Check COMSPEC for Windows cmd
    if let Ok(comspec) = env::var("COMSPEC") {
        if comspec.to_lowercase().contains("cmd") {
            return Ok(Shell::with_path(ShellType::Cmd, comspec));
        }
    }

    // 4. Check PSModulePath for PowerShell
    if env::var("PSModulePath").is_ok() {
        return Ok(Shell::new(ShellType::PowerShell));
    }

    // TODO: Walk process tree to find parent shell

    Err(TheFuckError::ShellDetectionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type_from_name() {
        assert_eq!(ShellType::from_name("bash"), ShellType::Bash);
        assert_eq!(ShellType::from_name("/bin/bash"), ShellType::Bash);
        assert_eq!(ShellType::from_name("/usr/local/bin/zsh"), ShellType::Zsh);
        assert_eq!(ShellType::from_name("fish"), ShellType::Fish);
        assert_eq!(ShellType::from_name("BASH"), ShellType::Bash);
        assert_eq!(ShellType::from_name("unknown_shell"), ShellType::Unknown);
    }

    #[test]
    fn test_shell_type_display() {
        assert_eq!(format!("{}", ShellType::Bash), "bash");
        assert_eq!(format!("{}", ShellType::Zsh), "zsh");
    }
}
