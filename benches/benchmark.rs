use criterion::{black_box, criterion_group, criterion_main, Criterion};
use thefuck::types::Command;

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

fn benchmark_command_script_parts(c: &mut Criterion) {
    c.bench_function("Command::script_parts", |b| {
        let mut cmd = Command::new("git push origin main --force", None);
        b.iter(|| {
            // Reset cache to measure parsing time
            let mut cmd_clone = cmd.clone();
            black_box(cmd_clone.script_parts())
        })
    });
}

fn benchmark_command_script_parts_cached(c: &mut Criterion) {
    c.bench_function("Command::script_parts (cached)", |b| {
        let mut cmd = Command::new("git push origin main --force", None);
        // Pre-compute to cache
        let _ = cmd.script_parts();
        b.iter(|| black_box(cmd.script_parts()))
    });
}

fn benchmark_command_script_parts_complex(c: &mut Criterion) {
    c.bench_function("Command::script_parts (complex)", |b| {
        b.iter(|| {
            let mut cmd = Command::new(
                black_box(r#"git commit -m "fix: resolve issue with \"quotes\" in message""#),
                None,
            );
            black_box(cmd.script_parts())
        })
    });
}

criterion_group!(
    benches,
    benchmark_command_creation,
    benchmark_command_script_parts,
    benchmark_command_script_parts_cached,
    benchmark_command_script_parts_complex,
);

criterion_main!(benches);
