# Performance Benchmarks

Performance comparison between thefuck-rs (Rust) and thefuck (Python).

## Summary

| Metric | Rust (thefuck-rs) | Python (thefuck) | Improvement |
|--------|-------------------|------------------|-------------|
| Startup time | ~3ms | ~300ms* | ~100x faster |
| Rule matching (simple) | 3-300 ns | N/A | - |
| Full correction flow | ~340 µs | ~50-200ms* | 150-600x faster |
| Memory usage | ~2 MB | ~30-50 MB* | 15-25x less |

*Python estimates based on typical measurements. Run `./benchmark_comparison.sh` with Python thefuck installed for exact comparison.

## Detailed Criterion Benchmarks

### Command Operations

| Operation | Time |
|-----------|------|
| Command::new | 57.75 ns |
| Command::script_parts (simple) | 233 ns |
| Command::script_parts (with quotes) | 366 ns |

### Rule System

| Operation | Time |
|-----------|------|
| get_builtin_rules | 68 ns |
| Settings::default | 630 ns |

### Individual Rule Matching

| Rule | Time |
|------|------|
| cd_parent (no output) | 3.3 ns |
| sudo | 286 ns |
| git_not_command | 329 ns |

### Full Correction Flow

| Test Case | Time |
|-----------|------|
| Permission denied (sudo) | 2.9 µs |
| cd_parent | 1.4 µs |
| No match | 2.0 µs |
| Git typo (with PATH scan) | 345 µs |
| Complete flow | 338 µs |

## Why Rust is Faster

1. **No interpreter overhead**: Native compiled binary vs Python interpreter startup
2. **Static rule registration**: Rules compiled-in vs dynamic module loading
3. **Zero-cost abstractions**: Trait-based dispatch with static analysis
4. **Efficient string handling**: Zero-copy parsing where possible
5. **Optimized regex**: Rust regex crate with SIMD acceleration

## Running Benchmarks

```bash
# Run Criterion micro-benchmarks
cargo bench

# Run comparison with Python (requires thefuck installed)
./benchmark_comparison.sh [iterations]

# Generate charts (requires matplotlib)
python3 scripts/generate_charts.py
```

## Benchmark Environment

- Hardware: Results will vary based on CPU
- Rust: Release build with LTO
- Criterion: 100 samples per benchmark
