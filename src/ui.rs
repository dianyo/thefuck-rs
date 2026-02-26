use crate::config::Settings;
use crate::types::CorrectedCommand;
use colored::{control::set_override, Colorize};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};

/// Actions that can be performed in the command selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Select the current command
    Select,
    /// Abort without selecting
    Abort,
    /// Move to the previous command
    Previous,
    /// Move to the next command
    Next,
}

/// Command selector for interactive selection.
pub struct CommandSelector {
    commands: Vec<CorrectedCommand>,
    index: usize,
}

impl CommandSelector {
    /// Creates a new command selector with the given commands.
    pub fn new(commands: Vec<CorrectedCommand>) -> Option<Self> {
        if commands.is_empty() {
            None
        } else {
            Some(Self { commands, index: 0 })
        }
    }

    /// Moves to the next command (wrapping around).
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.commands.len();
    }

    /// Moves to the previous command (wrapping around).
    pub fn previous(&mut self) {
        if self.index == 0 {
            self.index = self.commands.len() - 1;
        } else {
            self.index -= 1;
        }
    }

    /// Returns the currently selected command.
    pub fn current(&self) -> &CorrectedCommand {
        &self.commands[self.index]
    }

    /// Returns all commands.
    pub fn commands(&self) -> &[CorrectedCommand] {
        &self.commands
    }

    /// Returns the current index.
    pub fn current_index(&self) -> usize {
        self.index
    }

    /// Returns the total number of commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns true if there are no commands.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// Reads a key press and returns the corresponding action.
fn read_action() -> io::Result<Action> {
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if let Some(action) = key_to_action(key_event) {
                    return Ok(action);
                }
            }
        }
    }
}

/// Converts a key event to an action.
fn key_to_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        // Arrow keys
        KeyCode::Up => Some(Action::Previous),
        KeyCode::Down => Some(Action::Next),

        // Vim-style navigation (j/k)
        KeyCode::Char('k') => Some(Action::Previous),
        KeyCode::Char('j') => Some(Action::Next),

        // Ctrl+N / Ctrl+P (Emacs-style)
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Next),
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::Previous)
        }

        // Abort
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Abort),
        KeyCode::Char('q') => Some(Action::Abort),
        KeyCode::Esc => Some(Action::Abort),

        // Select
        KeyCode::Enter => Some(Action::Select),
        KeyCode::Char(' ') => Some(Action::Select),

        _ => None,
    }
}

/// Displays the confirmation prompt for a command.
fn show_confirmation(selector: &CommandSelector, no_colors: bool) {
    let cmd = selector.current();
    let index = selector.current_index();
    let total = selector.len();

    // Clear the current line
    let mut stderr = io::stderr();
    execute!(stderr, cursor::MoveToColumn(0), terminal::Clear(ClearType::CurrentLine)).ok();

    // Build the prompt
    if no_colors {
        eprint!(
            "{} [{}/{}] [enter/↑/↓/ctrl+c]",
            cmd.script, index + 1, total
        );
    } else {
        eprint!(
            "{} {} {}",
            cmd.script.green().bold(),
            format!("[{}/{}]", index + 1, total).dimmed(),
            "[enter/↑/↓/ctrl+c]".dimmed()
        );
    }
    stderr.flush().ok();
}

/// Shows a message when no corrections are found.
fn show_no_match(alias: &str, no_colors: bool) {
    let message = if alias == "fuck" {
        "No fucks given"
    } else {
        "Nothing found"
    };

    if no_colors {
        eprintln!("{}", message);
    } else {
        eprintln!("{}", message.red());
    }
}

/// Shows the selected command (for non-interactive mode).
fn show_corrected_command(cmd: &CorrectedCommand, no_colors: bool) {
    if no_colors {
        eprintln!("{} [{}]", cmd.script, cmd.rule_name);
    } else {
        eprintln!(
            "{} {}",
            cmd.script.green().bold(),
            format!("[{}]", cmd.rule_name).dimmed()
        );
    }
}

/// Shows the aborted message.
fn show_aborted(no_colors: bool) {
    if no_colors {
        eprintln!("\nAborted");
    } else {
        eprintln!("\n{}", "Aborted".red());
    }
}

/// Selects a command from the given corrections.
///
/// Returns:
/// - The first command when confirmation is disabled
/// - None when Ctrl+C is pressed or no corrections are available
/// - The selected command otherwise
pub fn select_command(
    corrections: Vec<CorrectedCommand>,
    settings: &Settings,
) -> Option<CorrectedCommand> {
    let selector = match CommandSelector::new(corrections) {
        Some(s) => s,
        None => {
            show_no_match("fuck", settings.no_colors);
            return None;
        }
    };

    // If confirmation is disabled, return the first command immediately
    if !settings.require_confirmation {
        show_corrected_command(selector.current(), settings.no_colors);
        return Some(selector.commands.into_iter().next().unwrap());
    }

    // Interactive selection
    select_interactive(selector, settings)
}

/// Interactive command selection with arrow keys.
fn select_interactive(
    mut selector: CommandSelector,
    settings: &Settings,
) -> Option<CorrectedCommand> {
    // Force colors on before entering raw mode (raw mode can break TTY detection)
    if !settings.no_colors {
        set_override(true);
    }

    // Enable raw mode for key reading
    if terminal::enable_raw_mode().is_err() {
        // Fallback to non-interactive mode if raw mode fails
        show_corrected_command(selector.current(), settings.no_colors);
        return Some(selector.commands.into_iter().next().unwrap());
    }

    // Show initial prompt
    show_confirmation(&selector, settings.no_colors);

    let result = loop {
        match read_action() {
            Ok(Action::Select) => {
                eprintln!(); // New line after selection
                break Some(selector.commands.swap_remove(selector.current_index()));
            }
            Ok(Action::Abort) => {
                show_aborted(settings.no_colors);
                break None;
            }
            Ok(Action::Previous) => {
                selector.previous();
                show_confirmation(&selector, settings.no_colors);
            }
            Ok(Action::Next) => {
                selector.next();
                show_confirmation(&selector, settings.no_colors);
            }
            Err(_) => {
                // Error reading key, abort
                show_aborted(settings.no_colors);
                break None;
            }
        }
    };

    // Restore terminal mode
    terminal::disable_raw_mode().ok();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_selector_new_empty() {
        let selector = CommandSelector::new(vec![]);
        assert!(selector.is_none());
    }

    #[test]
    fn test_command_selector_new() {
        let commands = vec![
            CorrectedCommand::new("fix1", "rule1", 100),
            CorrectedCommand::new("fix2", "rule2", 200),
        ];
        let selector = CommandSelector::new(commands).unwrap();
        assert_eq!(selector.current().script, "fix1");
        assert_eq!(selector.len(), 2);
    }

    #[test]
    fn test_command_selector_next() {
        let commands = vec![
            CorrectedCommand::new("fix1", "rule1", 100),
            CorrectedCommand::new("fix2", "rule2", 200),
        ];
        let mut selector = CommandSelector::new(commands).unwrap();

        assert_eq!(selector.current().script, "fix1");

        selector.next();
        assert_eq!(selector.current().script, "fix2");

        selector.next();
        assert_eq!(selector.current().script, "fix1"); // Wrapped around
    }

    #[test]
    fn test_command_selector_previous() {
        let commands = vec![
            CorrectedCommand::new("fix1", "rule1", 100),
            CorrectedCommand::new("fix2", "rule2", 200),
        ];
        let mut selector = CommandSelector::new(commands).unwrap();

        assert_eq!(selector.current().script, "fix1");

        selector.previous();
        assert_eq!(selector.current().script, "fix2"); // Wrapped to end

        selector.previous();
        assert_eq!(selector.current().script, "fix1");
    }

    #[test]
    fn test_key_to_action() {
        // Arrow keys
        assert_eq!(
            key_to_action(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Some(Action::Previous)
        );
        assert_eq!(
            key_to_action(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Some(Action::Next)
        );

        // Vim keys
        assert_eq!(
            key_to_action(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
            Some(Action::Previous)
        );
        assert_eq!(
            key_to_action(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
            Some(Action::Next)
        );

        // Ctrl+C
        assert_eq!(
            key_to_action(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Action::Abort)
        );

        // Enter
        assert_eq!(
            key_to_action(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some(Action::Select)
        );

        // Unknown key
        assert_eq!(
            key_to_action(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
            None
        );
    }
}
