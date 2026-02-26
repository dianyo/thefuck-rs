use criterion::{black_box, criterion_group, criterion_main, Criterion};
use thefuck::config::Settings;
use thefuck::corrector::Corrector;
use thefuck::rules::{get_builtin_rules, CdParentRule, GitNotCommandRule, SudoRule};
use thefuck::types::{Command, Rule};

/// Benchmark Command creation
fn benchmark_command_creation(c: &mut Criterion) {
    c.bench_function("Command::new", |b| {
        b.iter(|| {
            Command::new(
                black_box("git push origin main"),
                black_box(Some("error: failed to push".to_string())),
            )
        })
    });
}

/// Benchmark Command script_parts parsing
fn benchmark_command_script_parts(c: &mut Criterion) {
    let mut group = c.benchmark_group("Command::script_parts");

    // Simple command
    group.bench_function("simple", |b| {
        b.iter(|| {
            let mut cmd = Command::new("git push origin main", None);
            let parts = cmd.script_parts();
            black_box(parts.len())
        })
    });

    // Complex command with quotes
    group.bench_function("with_quotes", |b| {
        b.iter(|| {
            let mut cmd = Command::new(
                r#"git commit -m "fix: resolve issue with \"quotes\" in message""#,
                None,
            );
            let parts = cmd.script_parts();
            black_box(parts.len())
        })
    });

    group.finish();
}

/// Benchmark rule loading
fn benchmark_rule_loading(c: &mut Criterion) {
    c.bench_function("get_builtin_rules", |b| {
        b.iter(|| {
            black_box(get_builtin_rules())
        })
    });
}

/// Benchmark corrector with different commands
fn benchmark_corrector(c: &mut Criterion) {
    let settings = Settings::default();
    let rules = get_builtin_rules();
    let rule_refs: Vec<&dyn thefuck::Rule> = rules.iter().map(|r| r.as_ref()).collect();

    let mut group = c.benchmark_group("Corrector");

    // Command that matches git_not_command
    group.bench_function("git_typo", |b| {
        let corrector = Corrector::new(rule_refs.clone(), &settings);
        let cmd = Command::new(
            "git psuh origin main",
            Some("git: 'psuh' is not a git command. See 'git --help'.\n\nThe most similar command is\n\tpush\n".to_string()),
        );
        b.iter(|| {
            black_box(corrector.get_corrected_commands(&cmd))
        })
    });

    // Command that matches sudo
    group.bench_function("permission_denied", |b| {
        let corrector = Corrector::new(rule_refs.clone(), &settings);
        let cmd = Command::new(
            "cat /etc/shadow",
            Some("cat: /etc/shadow: Permission denied".to_string()),
        );
        b.iter(|| {
            black_box(corrector.get_corrected_commands(&cmd))
        })
    });

    // Command that matches cd_parent (no output required)
    group.bench_function("cd_parent", |b| {
        let corrector = Corrector::new(rule_refs.clone(), &settings);
        let cmd = Command::new("cd..", None);
        b.iter(|| {
            black_box(corrector.get_corrected_commands(&cmd))
        })
    });

    // Command that matches nothing
    group.bench_function("no_match", |b| {
        let corrector = Corrector::new(rule_refs.clone(), &settings);
        let cmd = Command::new(
            "echo hello",
            Some("hello".to_string()),
        );
        b.iter(|| {
            black_box(corrector.get_corrected_commands(&cmd))
        })
    });

    group.finish();
}

/// Benchmark settings loading
fn benchmark_settings(c: &mut Criterion) {
    c.bench_function("Settings::default", |b| {
        b.iter(|| {
            black_box(Settings::default())
        })
    });
}

/// Benchmark full flow (without command re-running)
fn benchmark_full_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_flow");

    // Simulating the complete correction flow
    group.bench_function("complete_correction", |b| {
        b.iter(|| {
            // 1. Load settings
            let settings = Settings::default();

            // 2. Load rules
            let rules = get_builtin_rules();
            let rule_refs: Vec<&dyn thefuck::Rule> = rules.iter().map(|r| r.as_ref()).collect();

            // 3. Create command (pre-computed output)
            let cmd = Command::new(
                "git psuh origin main",
                Some("git: 'psuh' is not a git command.".to_string()),
            );

            // 4. Create corrector and get corrections
            let corrector = Corrector::new(rule_refs, &settings);
            let corrections = corrector.get_corrected_commands(&cmd);

            black_box(corrections)
        })
    });

    group.finish();
}

/// Benchmark individual rule matching
fn benchmark_rule_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_matching");

    // Sudo rule
    let sudo_rule = SudoRule::new();
    let sudo_cmd = Command::new(
        "cat /etc/shadow",
        Some("Permission denied".to_string()),
    );
    group.bench_function("sudo_match", |b| {
        b.iter(|| {
            black_box(sudo_rule.matches(&sudo_cmd))
        })
    });

    // Git not command rule
    let git_rule = GitNotCommandRule::new();
    let git_cmd = Command::new(
        "git psuh",
        Some("git: 'psuh' is not a git command.".to_string()),
    );
    group.bench_function("git_not_command_match", |b| {
        b.iter(|| {
            black_box(git_rule.matches(&git_cmd))
        })
    });

    // Cd parent rule (no output)
    let cd_rule = CdParentRule::new();
    let cd_cmd = Command::new("cd..", None);
    group.bench_function("cd_parent_match", |b| {
        b.iter(|| {
            black_box(cd_rule.matches(&cd_cmd))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_command_creation,
    benchmark_command_script_parts,
    benchmark_rule_loading,
    benchmark_corrector,
    benchmark_settings,
    benchmark_full_flow,
    benchmark_rule_matching,
);

criterion_main!(benches);
