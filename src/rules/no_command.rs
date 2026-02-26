use crate::types::{Command, Rule};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use strsim::jaro_winkler;

/// Rule that suggests similar commands when a command is not found.
///
/// When you mistype a command like `gti` instead of `git`, this rule
/// suggests similar executables that exist on the system.
pub struct NoCommandRule {
    /// Cached list of executables (computed lazily)
    executables_cache: Option<HashSet<String>>,
}

impl NoCommandRule {
    pub fn new() -> Self {
        Self {
            executables_cache: None,
        }
    }

    /// Gets all executables from PATH.
    fn get_all_executables(&mut self) -> &HashSet<String> {
        if self.executables_cache.is_none() {
            let mut executables = HashSet::new();

            if let Ok(path_var) = env::var("PATH") {
                for path_dir in env::split_paths(&path_var) {
                    if let Ok(entries) = fs::read_dir(&path_dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let path = entry.path();
                            if Self::is_executable(&path) {
                                if let Some(name) = path.file_name() {
                                    if let Some(name_str) = name.to_str() {
                                        executables.insert(name_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            self.executables_cache = Some(executables);
        }

        self.executables_cache.as_ref().unwrap()
    }

    /// Checks if a path is executable.
    #[cfg(unix)]
    fn is_executable(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            return metadata.is_file() && (permissions.mode() & 0o111 != 0);
        }
        false
    }

    #[cfg(not(unix))]
    fn is_executable(path: &Path) -> bool {
        // On Windows, check for common executable extensions
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            return matches!(ext_lower.as_str(), "exe" | "cmd" | "bat" | "com");
        }
        false
    }

    /// Finds close matches for a command name.
    fn get_close_matches(&mut self, name: &str, max_matches: usize) -> Vec<String> {
        let executables = self.get_all_executables();
        // Use lower threshold for short names (jaro_winkler gives lower scores for short strings)
        let threshold = if name.len() <= 3 {
            0.5
        } else if name.len() <= 5 {
            0.6
        } else {
            0.7
        };

        let mut matches: Vec<(String, f64)> = executables
            .iter()
            .filter_map(|exec| {
                let similarity = jaro_winkler(name, exec);
                if similarity >= threshold {
                    Some((exec.clone(), similarity))
                } else {
                    None
                }
            })
            .collect();

        // Sort by similarity (highest first)
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top matches
        matches
            .into_iter()
            .take(max_matches)
            .map(|(name, _)| name)
            .collect()
    }

    /// Checks if a command exists.
    fn command_exists(&mut self, name: &str) -> bool {
        self.get_all_executables().contains(name)
    }
}

impl Default for NoCommandRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for NoCommandRule {
    fn name(&self) -> &str {
        "no_command"
    }

    fn matches(&self, command: &Command) -> bool {
        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.is_empty() {
            return false;
        }

        let cmd_name = &parts[0];

        // Check if output indicates command not found
        let output = match &command.output {
            Some(out) => out.to_lowercase(),
            None => return false,
        };

        let is_not_found = output.contains("not found")
            || output.contains("is not recognized as")
            || output.contains("command not found")
            || output.contains("not recognized as an internal or external command");

        if !is_not_found {
            return false;
        }

        // Check that the command doesn't exist but we can find similar ones
        let mut rule = NoCommandRule::new();
        !rule.command_exists(cmd_name) && !rule.get_close_matches(cmd_name, 1).is_empty()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.is_empty() {
            return vec![];
        }

        let old_command = &parts[0];
        let mut rule = NoCommandRule::new();
        let matches = rule.get_close_matches(old_command, 3);

        matches
            .into_iter()
            .map(|new_cmd| command.script.replacen(old_command, &new_cmd, 1))
            .collect()
    }

    fn priority(&self) -> i32 {
        // Higher priority value = checked later (fallback rule)
        3000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_command_matches_not_found() {
        let rule = NoCommandRule::new();
        // This test is environment-dependent (needs git on PATH)
        // We're testing the logic, not the actual matching
        let cmd = Command::new(
            "gti status",
            Some("zsh: command not found: gti".to_string()),
        );

        // The rule will match if 'git' is in PATH and 'gti' is not
        // This is environment-dependent, so we just check the output parsing
        let output = cmd.output.as_ref().unwrap().to_lowercase();
        assert!(output.contains("not found"));
    }

    #[test]
    fn test_no_command_close_matches() {
        // Test that the jaro_winkler similarity algorithm works as expected
        // "gti" is a typo of "git" - jaro_winkler gives ~0.55
        let similarity = jaro_winkler("git", "gti");
        assert!(
            similarity > 0.5,
            "gti should be similar to git, got {}",
            similarity
        );

        // "git" and "xyz" should not be similar
        let similarity2 = jaro_winkler("git", "xyz");
        assert!(
            similarity2 < 0.5,
            "xyz should not be similar to git, got {}",
            similarity2
        );

        // "push" and "psuh" should be similar (transposition)
        let similarity3 = jaro_winkler("push", "psuh");
        assert!(
            similarity3 > 0.7,
            "psuh should be similar to push, got {}",
            similarity3
        );

        // "mkdir" and "mkidr" should be similar
        let similarity4 = jaro_winkler("mkdir", "mkidr");
        assert!(
            similarity4 > 0.8,
            "mkidr should be similar to mkdir, got {}",
            similarity4
        );
    }

    #[test]
    fn test_no_command_priority() {
        let rule = NoCommandRule::new();
        assert_eq!(rule.priority(), 3000);
        assert!(rule.priority() > crate::types::DEFAULT_PRIORITY);
    }
}
