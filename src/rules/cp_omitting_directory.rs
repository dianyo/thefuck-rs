use crate::types::{Command, Rule};

/// Rule that adds -r flag to cp when trying to copy a directory.
pub struct CpOmittingDirectoryRule;

impl CpOmittingDirectoryRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CpOmittingDirectoryRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for CpOmittingDirectoryRule {
    fn name(&self) -> &str {
        "cp_omitting_directory"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(out) => out.to_lowercase(),
            None => return false,
        };

        command.script.starts_with("cp ")
            && (output.contains("omitting directory")
                || output.contains("is a directory")
                || output.contains("not a regular file"))
            && !command.script.contains("-r")
            && !command.script.contains("-R")
            && !command.script.contains("-a")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Insert -r after cp
        let script = command.script.replacen("cp ", "cp -r ", 1);
        vec![script]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_omitting_directory_matches() {
        let rule = CpOmittingDirectoryRule::new();
        let cmd = Command::new(
            "cp mydir /dest",
            Some("cp: omitting directory 'mydir'".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_cp_omitting_directory_matches_is_directory() {
        let rule = CpOmittingDirectoryRule::new();
        let cmd = Command::new(
            "cp mydir /dest",
            Some("cp: mydir is a directory (not copied)".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_cp_omitting_directory_no_match_with_r_flag() {
        let rule = CpOmittingDirectoryRule::new();
        let cmd = Command::new(
            "cp -r mydir /dest",
            Some("cp: omitting directory 'mydir'".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_cp_omitting_directory_no_match_with_a_flag() {
        let rule = CpOmittingDirectoryRule::new();
        let cmd = Command::new(
            "cp -a mydir /dest",
            Some("cp: omitting directory 'mydir'".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_cp_omitting_directory_get_new_command() {
        let rule = CpOmittingDirectoryRule::new();
        let cmd = Command::new("cp mydir /dest", Some("omitting directory".to_string()));
        assert_eq!(rule.get_new_command(&cmd), vec!["cp -r mydir /dest"]);
    }
}
