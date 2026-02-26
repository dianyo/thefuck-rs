use clap::{Parser, Subcommand};
use colored::Colorize;
use thefuck::config::Settings;
use thefuck::shell::detect_shell;

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
            print_alias(&name);
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

fn print_alias(name: &str) {
    match detect_shell() {
        Ok(shell) => {
            let alias = generate_alias(name, &shell.shell_type);
            println!("{}", alias);
        }
        Err(e) => {
            eprintln!("{}: {}", "Error detecting shell".red(), e);
            std::process::exit(1);
        }
    }
}

fn generate_alias(name: &str, shell_type: &thefuck::shell::ShellType) -> String {
    use thefuck::shell::ShellType;

    match shell_type {
        ShellType::Bash | ShellType::Zsh => {
            format!(
                r#"{name} () {{
    TF_PYTHONIOENCODING=$PYTHONIOENCODING;
    export TF_SHELL={shell};
    export TF_ALIAS={name};
    export TF_SHELL_ALIASES=$(alias);
    export TF_HISTORY="$(fc -ln -10)";
    export PYTHONIOENCODING=utf-8;
    TF_CMD=$(
        thefuck --force-command "$TF_HISTORY"
    ) && eval "$TF_CMD";
    unset TF_HISTORY;
    export PYTHONIOENCODING=$TF_PYTHONIOENCODING;
    history -s "$TF_CMD";
}}"#,
                name = name,
                shell = shell_type.name()
            )
        }
        ShellType::Fish => {
            format!(
                r#"function {name} -d "Correct your previous console command"
    set -l TF_HISTORY (builtin history -n 10)
    set -lx TF_SHELL fish
    set -lx TF_ALIAS {name}
    set -l TF_CMD (thefuck --force-command "$TF_HISTORY")
    if test -n "$TF_CMD"
        eval "$TF_CMD"
        builtin history delete --exact --case-sensitive -- (builtin history -n 1)
        builtin history merge
    end
end"#,
                name = name
            )
        }
        ShellType::PowerShell => {
            format!(
                r#"function {name} {{
    $TF_HISTORY = (Get-History -Count 10 | ForEach-Object {{ $_.CommandLine }}) -join "`n"
    $env:TF_SHELL = "powershell"
    $env:TF_ALIAS = "{name}"
    $TF_CMD = (thefuck --force-command $TF_HISTORY)
    if ($TF_CMD) {{
        Invoke-Expression $TF_CMD
    }}
}}"#,
                name = name
            )
        }
        _ => {
            format!(
                "# Shell '{}' is not fully supported yet.\n# Please contribute!",
                shell_type.name()
            )
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

fn fix_command(command: &str, settings: &Settings) {
    if settings.debug {
        eprintln!("{}: {}", "Fixing command".blue(), command);
        eprintln!("  Timeout: {}s", settings.get_timeout(command));
        eprintln!("  Require confirmation: {}", settings.require_confirmation);
    }

    // TODO: Implement actual command fixing
    // For now, just echo back the command
    println!(
        "{}",
        "Command fixing not yet implemented. Phase 4+ required.".yellow()
    );
    println!("Input: {}", command);
}
