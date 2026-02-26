//! Rule to add execute permission for scripts.
//!
//! When you try to run `./script` and get permission denied,
//! this suggests `chmod +x` first.

use crate::types::{Command, Rule};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct ChmodXRule;

impl ChmodXRule {
    pub fn new() -> Self {
        Self
    }

    fn get_script_path(command: &Command) -> Option<String> {
        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.is_empty() {
            return None;
        }

        let script = &parts[0];
        if script.starts_with("./") {
            Some(script.clone())
        } else {
            None
        }
    }

    fn file_exists_without_execute(path: &str) -> bool {
        let path = Path::new(path);

        if !path.exists() {
            return false;
        }

        if let Ok(metadata) = fs::metadata(path) {
            let mode = metadata.permissions().mode();
            // Check if file lacks execute permission for owner
            return mode & 0o100 == 0;
        }

        false
    }
}

impl Default for ChmodXRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ChmodXRule {
    fn name(&self) -> &str {
        "chmod_x"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(o) => o.to_lowercase(),
            None => return false,
        };

        if !command.script.starts_with("./") {
            return false;
        }

        if !output.contains("permission denied") {
            return false;
        }

        if let Some(script_path) = Self::get_script_path(command) {
            return Self::file_exists_without_execute(&script_path);
        }

        false
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        if let Some(script_path) = Self::get_script_path(command) {
            // Remove the "./" prefix for chmod
            let path_for_chmod = script_path.strip_prefix("./").unwrap_or(&script_path);
            return vec![format!("chmod +x {} && {}", path_for_chmod, command.script)];
        }

        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_chmod_x_no_match_regular_command() {
        let rule = ChmodXRule::new();

        let cmd = Command::new(
            "ls -la",
            Some("Permission denied".to_string()),
        );

        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_chmod_x_no_match_without_permission_error() {
        let rule = ChmodXRule::new();

        let cmd = Command::new(
            "./script.sh",
            Some("Hello, world!".to_string()),
        );

        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_chmod_x_get_new_command() {
        let rule = ChmodXRule::new();

        let cmd = Command::new(
            "./test_script.sh arg1 arg2",
            Some("bash: ./test_script.sh: Permission denied".to_string()),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["chmod +x test_script.sh && ./test_script.sh arg1 arg2"]);
    }

    #[test]
    fn test_chmod_x_with_actual_file() {
        let rule = ChmodXRule::new();

        // Create a temporary file without execute permission
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_script.sh");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "#!/bin/bash\necho 'test'").unwrap();

        // Set permissions to read/write only (no execute)
        let mut perms = file.metadata().unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms).unwrap();

        // Verify our helper function works
        assert!(ChmodXRule::file_exists_without_execute(file_path.to_str().unwrap()));
    }
}
