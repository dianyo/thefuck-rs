use crate::config::Settings;
use crate::types::{Command, CorrectedCommand, Rule};
use std::collections::HashSet;

/// The corrector is responsible for matching rules against commands
/// and generating corrected commands.
pub struct Corrector<'a> {
    rules: Vec<&'a dyn Rule>,
    settings: &'a Settings,
}

impl<'a> Corrector<'a> {
    /// Creates a new Corrector with the given rules and settings.
    pub fn new(rules: Vec<&'a dyn Rule>, settings: &'a Settings) -> Self {
        Self { rules, settings }
    }

    /// Returns all enabled rules sorted by priority.
    fn get_enabled_rules(&self) -> Vec<&dyn Rule> {
        let mut enabled: Vec<&dyn Rule> = self
            .rules
            .iter()
            .copied()
            .filter(|rule| {
                self.settings
                    .is_rule_enabled(rule.name(), rule.enabled_by_default())
            })
            .collect();

        // Sort by priority (lower priority = checked first)
        enabled.sort_by_key(|rule| {
            self.settings
                .get_rule_priority(rule.name(), rule.priority())
        });

        enabled
    }

    /// Checks if a rule matches the command.
    fn is_match(&self, rule: &dyn Rule, command: &Command) -> bool {
        // If the rule requires output but we don't have any, skip
        if rule.requires_output() && command.output.is_none() {
            tracing::debug!("Skipping rule '{}': requires output but none available", rule.name());
            return false;
        }

        tracing::debug!("Trying rule '{}'...", rule.name());

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rule.matches(command)
        })) {
            Ok(result) => {
                if result {
                    tracing::debug!("Rule '{}' matched!", rule.name());
                }
                result
            }
            Err(e) => {
                tracing::error!(
                    "Rule '{}' panicked during match: {:?}",
                    rule.name(),
                    e
                );
                false
            }
        }
    }

    /// Gets corrected commands from a matching rule.
    fn get_corrected_from_rule(
        &self,
        rule: &dyn Rule,
        command: &Command,
    ) -> Vec<CorrectedCommand> {
        let base_priority = self
            .settings
            .get_rule_priority(rule.name(), rule.priority());

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rule.get_new_command(command)
        })) {
            Ok(new_commands) => {
                new_commands
                    .into_iter()
                    .enumerate()
                    .map(|(i, script)| {
                        // Priority increases for each additional suggestion from the same rule
                        let priority = (i as i32 + 1) * base_priority;
                        CorrectedCommand::new(script, rule.name(), priority)
                    })
                    .collect()
            }
            Err(e) => {
                tracing::error!(
                    "Rule '{}' panicked during get_new_command: {:?}",
                    rule.name(),
                    e
                );
                vec![]
            }
        }
    }

    /// Gets all corrected commands for the given command.
    ///
    /// Returns an iterator of corrected commands, sorted by priority
    /// and deduplicated.
    pub fn get_corrected_commands(&self, command: &Command) -> Vec<CorrectedCommand> {
        let rules = self.get_enabled_rules();

        tracing::debug!(
            "Checking {} enabled rules against command: {}",
            rules.len(),
            command.script
        );

        // Collect all corrected commands from matching rules
        let mut all_corrections: Vec<CorrectedCommand> = vec![];

        for rule in rules {
            if self.is_match(rule, command) {
                let corrections = self.get_corrected_from_rule(rule, command);
                all_corrections.extend(corrections);
            }
        }

        // Organize: sort by priority and remove duplicates
        organize_commands(all_corrections)
    }
}

/// Organizes corrected commands: sorts by priority and removes duplicates.
///
/// Duplicates are determined by the script and rule name (ignoring priority).
fn organize_commands(mut commands: Vec<CorrectedCommand>) -> Vec<CorrectedCommand> {
    if commands.is_empty() {
        return commands;
    }

    // Sort by priority
    commands.sort();

    // Remove duplicates while preserving order
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut result: Vec<CorrectedCommand> = vec![];

    for cmd in commands {
        let key = (cmd.script.clone(), cmd.rule_name.clone());
        if !seen.contains(&key) {
            seen.insert(key);
            result.push(cmd);
        }
    }

    if !result.is_empty() {
        tracing::debug!(
            "Corrected commands: {}",
            result
                .iter()
                .map(|c| format!("'{}' ({})", c.script, c.rule_name))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test rule that always matches
    struct AlwaysMatchRule;

    impl Rule for AlwaysMatchRule {
        fn name(&self) -> &str {
            "always_match"
        }

        fn matches(&self, _command: &Command) -> bool {
            true
        }

        fn get_new_command(&self, command: &Command) -> Vec<String> {
            vec![format!("fixed: {}", command.script)]
        }

        fn requires_output(&self) -> bool {
            false
        }
    }

    // Test rule that never matches
    struct NeverMatchRule;

    impl Rule for NeverMatchRule {
        fn name(&self) -> &str {
            "never_match"
        }

        fn matches(&self, _command: &Command) -> bool {
            false
        }

        fn get_new_command(&self, _command: &Command) -> Vec<String> {
            vec!["should never be called".to_string()]
        }
    }

    // Test rule with high priority
    struct HighPriorityRule;

    impl Rule for HighPriorityRule {
        fn name(&self) -> &str {
            "high_priority"
        }

        fn matches(&self, _command: &Command) -> bool {
            true
        }

        fn get_new_command(&self, _command: &Command) -> Vec<String> {
            vec!["high priority fix".to_string()]
        }

        fn priority(&self) -> i32 {
            100
        }

        fn requires_output(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_corrector_with_matching_rule() {
        let settings = Settings::default();
        let always_match = AlwaysMatchRule;
        let rules: Vec<&dyn Rule> = vec![&always_match];
        let corrector = Corrector::new(rules, &settings);

        let command = Command::new("test command", None);
        let corrections = corrector.get_corrected_commands(&command);

        assert_eq!(corrections.len(), 1);
        assert_eq!(corrections[0].script, "fixed: test command");
        assert_eq!(corrections[0].rule_name, "always_match");
    }

    #[test]
    fn test_corrector_with_non_matching_rule() {
        let settings = Settings::default();
        let never_match = NeverMatchRule;
        let rules: Vec<&dyn Rule> = vec![&never_match];
        let corrector = Corrector::new(rules, &settings);

        let command = Command::new("test command", Some("error output".to_string()));
        let corrections = corrector.get_corrected_commands(&command);

        assert!(corrections.is_empty());
    }

    #[test]
    fn test_corrector_priority_ordering() {
        let settings = Settings::default();
        let always_match = AlwaysMatchRule;
        let high_priority = HighPriorityRule;
        let rules: Vec<&dyn Rule> = vec![&always_match, &high_priority];
        let corrector = Corrector::new(rules, &settings);

        let command = Command::new("test", None);
        let corrections = corrector.get_corrected_commands(&command);

        assert_eq!(corrections.len(), 2);
        // High priority (100) should come before default priority (1000)
        assert_eq!(corrections[0].rule_name, "high_priority");
        assert_eq!(corrections[1].rule_name, "always_match");
    }

    #[test]
    fn test_corrector_requires_output() {
        let settings = Settings::default();
        let never_match = NeverMatchRule; // requires_output = true by default
        let rules: Vec<&dyn Rule> = vec![&never_match];
        let corrector = Corrector::new(rules, &settings);

        // Command without output - rule should be skipped
        let command = Command::new("test", None);
        let corrections = corrector.get_corrected_commands(&command);

        assert!(corrections.is_empty());
    }

    #[test]
    fn test_organize_commands_removes_duplicates() {
        let commands = vec![
            CorrectedCommand::new("fix1", "rule1", 100),
            CorrectedCommand::new("fix1", "rule1", 200), // duplicate
            CorrectedCommand::new("fix2", "rule2", 150),
        ];

        let result = organize_commands(commands);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].script, "fix1");
        assert_eq!(result[1].script, "fix2");
    }
}
