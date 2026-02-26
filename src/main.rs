use clap::{Parser, Subcommand};
use colored::Colorize;
use thefuck::config::Settings;
use thefuck::shell::{create_shell, detect_shell, get_raw_command_from_history};

/// thefuck-rs - Magnificent app which corrects your previous console command
#[derive(Parser, Debug)]
#[command(name = "thefuck")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The command to fix (usually passed by the shell alias)
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,

    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,

    /// Force a specific command to fix
    #[arg(long = "force-command", short = 'f')]
    force_command: Option<String>,

    /// Repeat mode - run thefuck again if the fix fails
    #[arg(long, short = 'r')]
    repeat: bool,

    /// Run without confirmation
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate shell alias
    Alias {
        /// Name for the alias (default: fuck)
        #[arg(default_value = "fuck")]
        name: String,
    },

    /// Show version information
    Version,

    /// Show detected shell
    Shell,

    /// Initialize configuration directory
    Init,

    /// Show current settings
    Config,
}

fn main() {
    let cli = Cli::parse();

    // Load settings
    let mut settings = Settings::load().unwrap_or_else(|e| {
        eprintln!("{}: {}", "Warning: Failed to load settings".yellow(), e);
        Settings::default()
    });

    // Merge CLI args
    settings.merge_from_args(cli.debug, cli.repeat, cli.yes);

    // Initialize logging
    if settings.debug {
        tracing_subscriber::fmt()
            .with_env_filter("thefuck=debug")
            .init();
    }

    match cli.command {
        Some(Commands::Alias { name }) => {
            print_alias(&name, &settings);
        }
        Some(Commands::Version) => {
            println!("thefuck-rs {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::Shell) => {
            show_shell();
        }
        Some(Commands::Init) => {
            init_config();
        }
        Some(Commands::Config) => {
            show_config(&settings);
        }
        None => {
            // Main fix command flow
            if let Some(force_cmd) = cli.force_command {
                fix_command(&force_cmd, &settings);
            } else if !cli.args.is_empty() {
                let cmd = cli.args.join(" ");
                fix_command(&cmd, &settings);
            } else {
                // No command provided - show help
                println!(
                    "{}",
                    "No command to fix. Use 'thefuck --help' for usage.".yellow()
                );
            }
        }
    }
}

fn print_alias(name: &str, settings: &Settings) {
    match detect_shell() {
        Ok(detected) => {
            let shell = create_shell(detected.shell_type, settings.clone());
            let alias = shell.app_alias(name);
            println!("{}", alias);
        }
        Err(e) => {
            eprintln!("{}: {}", "Error detecting shell".red(), e);
            std::process::exit(1);
        }
    }
}

fn show_shell() {
    match detect_shell() {
        Ok(shell) => {
            println!("Detected shell: {}", shell.shell_type.to_string().green());
            if let Some(path) = shell.path {
                println!("Shell path: {}", path);
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Failed to detect shell".red(), e);
            std::process::exit(1);
        }
    }
}

fn init_config() {
    match Settings::init_config_dir() {
        Ok(path) => {
            println!("{}", "Configuration initialized!".green());
            println!("Config directory: {}", path.display());
            if let Some(config_file) = Settings::config_file_path() {
                println!("Settings file: {}", config_file.display());
            }
            if let Some(rules_dir) = Settings::user_rules_dir() {
                println!("User rules directory: {}", rules_dir.display());
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Failed to initialize config".red(), e);
            std::process::exit(1);
        }
    }
}

fn show_config(settings: &Settings) {
    println!("{}", "Current Settings:".green().bold());
    println!();

    // Show paths
    println!("{}", "Paths:".blue());
    if let Some(config_dir) = Settings::config_dir() {
        println!("  Config directory: {}", config_dir.display());
    }
    if let Some(config_file) = Settings::config_file_path() {
        let exists = if config_file.exists() { "(exists)" } else { "(not found)" };
        println!("  Settings file: {} {}", config_file.display(), exists.dimmed());
    }
    if let Some(rules_dir) = Settings::user_rules_dir() {
        let exists = if rules_dir.exists() { "(exists)" } else { "(not found)" };
        println!("  User rules: {} {}", rules_dir.display(), exists.dimmed());
    }
    println!();

    // Show settings
    println!("{}", "Settings:".blue());
    println!("  rules: {:?}", settings.rules);
    println!("  exclude_rules: {:?}", settings.exclude_rules);
    println!("  wait_command: {}s", settings.wait_command);
    println!("  wait_slow_command: {}s", settings.wait_slow_command);
    println!("  require_confirmation: {}", settings.require_confirmation);
    println!("  no_colors: {}", settings.no_colors);
    println!("  debug: {}", settings.debug);
    println!("  alter_history: {}", settings.alter_history);
    println!("  repeat: {}", settings.repeat);
    println!("  instant_mode: {}", settings.instant_mode);
    println!("  num_close_matches: {}", settings.num_close_matches);
    println!("  history_limit: {:?}", settings.history_limit);
    println!("  slow_commands: {:?}", settings.slow_commands);

    if !settings.priority.is_empty() {
        println!("  priority overrides: {:?}", settings.priority);
    }
    if !settings.excluded_search_path_prefixes.is_empty() {
        println!("  excluded_search_path_prefixes: {:?}", settings.excluded_search_path_prefixes);
    }
}

fn fix_command(history_or_command: &str, settings: &Settings) {
    // Extract the actual command from history if needed
    let command = get_raw_command_from_history(history_or_command)
        .unwrap_or_else(|| history_or_command.to_string());

    if settings.debug {
        eprintln!("{}: {}", "Fixing command".blue(), command);
        eprintln!("  Timeout: {}s", settings.get_timeout(&command));
        eprintln!("  Require confirmation: {}", settings.require_confirmation);
    }

    // Get command output by re-running
    if settings.debug {
        eprintln!("{}", "Re-running command to get output...".dimmed());
    }

    let output = match thefuck::shell::get_output(&command, &command, settings) {
        Ok(Some(out)) => {
            if settings.debug {
                eprintln!("{}", "Got command output:".dimmed());
                for line in out.lines().take(5) {
                    eprintln!("  {}", line.dimmed());
                }
                if out.lines().count() > 5 {
                    eprintln!("  {}", "...".dimmed());
                }
            }
            Some(out)
        }
        Ok(None) => {
            if settings.debug {
                eprintln!("{}", "Command timed out".yellow());
            }
            None
        }
        Err(e) => {
            if settings.debug {
                eprintln!("{}: {}", "Failed to get output".red(), e);
            }
            None
        }
    };

    // Create a Command object
    let cmd = thefuck::Command::new(&command, output);

    if settings.debug {
        eprintln!("{}: {}", "Command object".blue(), cmd);
    }

    // Get built-in rules
    let builtin_rules = thefuck::get_builtin_rules();
    let rules: Vec<&dyn thefuck::Rule> = builtin_rules.iter().map(|r| r.as_ref()).collect();

    if settings.debug {
        eprintln!("{}: {} rules loaded", "Corrector".blue(), rules.len());
    }

    // Create corrector and get corrections
    let corrector = thefuck::Corrector::new(rules, settings);
    let corrections = corrector.get_corrected_commands(&cmd);

    if corrections.is_empty() {
        println!(
            "{}",
            "No correction found. Add more rules in Phase 5+.".yellow()
        );
        println!("Command: {}", command);
    } else {
        // For now, just print the first correction
        // UI selection will be added in Phase 6
        println!("{}", "Corrections found:".green());
        for (i, correction) in corrections.iter().enumerate() {
            if i == 0 {
                println!(
                    "  {} {} {}",
                    "→".green(),
                    correction.script.green().bold(),
                    format!("[{}]", correction.rule_name).dimmed()
                );
            } else {
                println!(
                    "    {} {}",
                    correction.script,
                    format!("[{}]", correction.rule_name).dimmed()
                );
            }
        }

        // Output the first correction for the shell to eval
        // (In the real flow, this would be selected by the user)
        if !settings.require_confirmation {
            print!("{}", corrections[0].script);
        }
    }
}
