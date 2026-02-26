//! Built-in rules for command correction.
//!
//! Rules are organized by category:
//! - `sudo` - Permission-related fixes
//! - `git` - Git command corrections
//! - `cd` - Directory navigation fixes
//! - `cargo` - Rust cargo command fixes
//! - etc.
//!
//! Each rule implements the `Rule` trait and is registered
//! with the rule registry.

use crate::types::Rule;

// Rule modules
pub mod cargo_no_command;
pub mod cat_dir;
pub mod cd_mkdir;
pub mod cd_parent;
pub mod chmod_x;
pub mod cp_omitting_directory;
pub mod git_add;
pub mod git_not_command;
pub mod git_push;
pub mod git_stash;
pub mod ls_la;
pub mod man_no_space;
pub mod mkdir_p;
pub mod no_command;
pub mod open;
pub mod python_command;
pub mod rm_dir;
pub mod sudo;
pub mod touch;

// Re-export rules
pub use cargo_no_command::CargoNoCommandRule;
pub use cat_dir::CatDirRule;
pub use cd_mkdir::CdMkdirRule;
pub use cd_parent::CdParentRule;
pub use chmod_x::ChmodXRule;
pub use cp_omitting_directory::CpOmittingDirectoryRule;
pub use git_add::GitAddRule;
pub use git_not_command::GitNotCommandRule;
pub use git_push::GitPushRule;
pub use git_stash::GitStashRule;
pub use ls_la::LsLaRule;
pub use man_no_space::ManNoSpaceRule;
pub use mkdir_p::MkdirPRule;
pub use no_command::NoCommandRule;
pub use open::OpenRule;
pub use python_command::PythonCommandRule;
pub use rm_dir::RmDirRule;
pub use sudo::SudoRule;
pub use touch::TouchRule;

/// Returns all built-in rules.
///
/// This function returns a vector of boxed rule trait objects.
/// Rules are returned in no particular order - sorting by priority
/// is done by the Corrector.
pub fn get_builtin_rules() -> Vec<Box<dyn Rule>> {
    let rules: Vec<Box<dyn Rule>> = vec![
        // Permission rules
        Box::new(SudoRule::new()),
        Box::new(ChmodXRule::new()),
        // Directory rules
        Box::new(CdMkdirRule::new()),
        Box::new(CdParentRule::new()),
        Box::new(MkdirPRule::new()),
        Box::new(TouchRule::new()),
        // File operation rules
        Box::new(RmDirRule::new()),
        Box::new(CpOmittingDirectoryRule::new()),
        Box::new(CatDirRule::new()),
        // Git rules
        Box::new(GitPushRule::new()),
        Box::new(GitNotCommandRule::new()),
        Box::new(GitAddRule::new()),
        Box::new(GitStashRule::new()),
        // Cargo (Rust) rules
        Box::new(CargoNoCommandRule::new()),
        // Python rules
        Box::new(PythonCommandRule::new()),
        // Command rules
        Box::new(NoCommandRule::new()),
        Box::new(ManNoSpaceRule::new()),
        Box::new(OpenRule::new()),
        // Misc rules
        Box::new(LsLaRule::new()),
    ];

    tracing::debug!("Loaded {} built-in rules", rules.len());
    rules
}

/// Macro to create a simple rule struct.
///
/// This macro generates a rule struct with the basic boilerplate.
/// For more complex rules, implement the Rule trait manually.
#[macro_export]
macro_rules! define_rule {
    (
        name: $name:ident,
        rule_name: $rule_name:expr,
        priority: $priority:expr,
        enabled_by_default: $enabled:expr,
        requires_output: $requires_output:expr,
        match_fn: $match_fn:expr,
        get_new_command_fn: $get_cmd_fn:expr
    ) => {
        pub struct $name;

        impl $name {
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::types::Rule for $name {
            fn name(&self) -> &str {
                $rule_name
            }

            fn matches(&self, command: &$crate::types::Command) -> bool {
                ($match_fn)(command)
            }

            fn get_new_command(&self, command: &$crate::types::Command) -> Vec<String> {
                ($get_cmd_fn)(command)
            }

            fn priority(&self) -> i32 {
                $priority
            }

            fn enabled_by_default(&self) -> bool {
                $enabled
            }

            fn requires_output(&self) -> bool {
                $requires_output
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_rules() {
        let rules = get_builtin_rules();
        assert_eq!(rules.len(), 19);
    }

    #[test]
    fn test_builtin_rules_have_names() {
        let rules = get_builtin_rules();
        let names: Vec<&str> = rules.iter().map(|r| r.name()).collect();

        // Original 10 rules
        assert!(names.contains(&"sudo"));
        assert!(names.contains(&"cd_mkdir"));
        assert!(names.contains(&"cd_parent"));
        assert!(names.contains(&"mkdir_p"));
        assert!(names.contains(&"rm_dir"));
        assert!(names.contains(&"cp_omitting_directory"));
        assert!(names.contains(&"git_push"));
        assert!(names.contains(&"git_not_command"));
        assert!(names.contains(&"no_command"));
        assert!(names.contains(&"ls_la"));

        // New rules from Phase 8
        assert!(names.contains(&"git_add"));
        assert!(names.contains(&"git_stash"));
        assert!(names.contains(&"cargo_no_command"));
        assert!(names.contains(&"cat_dir"));
        assert!(names.contains(&"chmod_x"));
        assert!(names.contains(&"python_command"));
        assert!(names.contains(&"man_no_space"));
        assert!(names.contains(&"touch"));
        assert!(names.contains(&"open"));
    }
}
