//! Built-in rules for command correction.
//!
//! Rules are organized by category:
//! - `sudo` - Permission-related fixes
//! - `git` - Git command corrections
//! - `cd` - Directory navigation fixes
//! - etc.
//!
//! Each rule implements the `Rule` trait and is registered
//! with the rule registry.

use crate::types::Rule;

// Rule modules will be added here as we implement them
// pub mod sudo;
// pub mod git_push;
// pub mod no_command;
// etc.

/// Returns all built-in rules.
///
/// This function returns a vector of boxed rule trait objects.
/// Rules are returned in no particular order - sorting by priority
/// is done by the Corrector.
pub fn get_builtin_rules() -> Vec<Box<dyn Rule>> {
    let rules: Vec<Box<dyn Rule>> = vec![
        // Rules will be added here as we implement them
        // Box::new(sudo::SudoRule::new()),
        // Box::new(git_push::GitPushRule::new()),
        // etc.
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
        // For now, we have no rules, so this should be empty
        assert!(rules.is_empty());
    }
}
