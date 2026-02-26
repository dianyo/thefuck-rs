use clap::{Parser, Subcommand};
use colored::Colorize;
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
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    if cli.debug {
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
        None => {
            // Main fix command flow
            if let Some(force_cmd) = cli.force_command {
                fix_command(&force_cmd, cli.debug);
            } else if !cli.args.is_empty() {
                let cmd = cli.args.join(" ");
                fix_command(&cmd, cli.debug);
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

fn fix_command(command: &str, debug: bool) {
    if debug {
        eprintln!("{}: {}", "Fixing command".blue(), command);
    }

    // TODO: Implement actual command fixing
    // For now, just echo back the command
    println!(
        "{}",
        "Command fixing not yet implemented. Phase 4+ required.".yellow()
    );
    println!("Input: {}", command);
}
