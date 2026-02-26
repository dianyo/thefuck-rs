# Performance Benchmarks

Real-world performance comparison between thefuck-rs (Rust) and thefuck (Python 3.32).

## Summary

| Metric | Rust (thefuck-rs) | Python (thefuck) | Improvement |
|--------|-------------------|------------------|-------------|
| Startup time | **2.3 ms** | 250 ms | **109x faster** |
| Correction time | **109 ms** | 340 ms | **3.1x faster** |
| Memory usage | **1.9 MB** | 36 MB | **19x less** |

*Tested on macOS with Apple Silicon, Python 3.11.6, ZSH 5.9*

## Detailed Results

### Startup Time (--version)

```
Rust (10 runs):
  8.2ms (cold)
  2.3ms, 3.0ms, 2.3ms, 2.3ms, 2.1ms, 2.1ms, 2.3ms, 2.4ms, 2.2ms
  Average: 2.3ms

Python (10 runs):
  221.8ms, 223.1ms, 206.6ms, 212.2ms, 300.6ms
  247.5ms, 307.3ms, 354.7ms, 231.2ms, 214.3ms
  Average: 252ms
```

**Rust is 109x faster for startup**

### Correction Time (with command re-execution)

```
Rust (-y -f "git psuh", 10 runs):
  121.3ms, 105.7ms, 108.9ms, 110.3ms, 110.0ms
  108.5ms, 106.3ms, 109.1ms, 110.6ms, 108.7ms
  Average: 109ms

Python (-y "git psuh", 5 runs):
  495.7ms, 292.1ms, 285.8ms, 270.9ms, 361.3ms
  Average: 341ms
```

**Rust is 3.1x faster for corrections**

Note: Both include the time to re-run the failed command to capture output.

### Memory Usage (peak RSS)

```
Rust:   2,031,616 bytes = 1.9 MB
Python: 37,928,960 bytes = 36.2 MB
```

**Rust uses 19x less memory**

## Micro-benchmarks (Criterion)

Internal operation benchmarks (Rust only):

| Operation | Time |
|-----------|------|
| Command::new | 57.75 ns |
| Command::script_parts (simple) | 233 ns |
| Command::script_parts (with quotes) | 366 ns |
| get_builtin_rules | 68 ns |
| Settings::default | 630 ns |

### Rule Matching

| Rule | Match Time |
|------|-----------|
| cd_parent (no output) | 3.3 ns |
| sudo | 286 ns |
| git_not_command | 329 ns |

### Full Correction Flow (without I/O)

| Test Case | Time |
|-----------|------|
| Permission denied (sudo) | 2.9 µs |
| cd_parent | 1.4 µs |
| No match | 2.0 µs |
| Git typo (with PATH scan) | 345 µs |
| Complete flow | 338 µs |

## Why Rust is Faster

1. **No interpreter overhead**: Native compiled binary vs Python startup
2. **Static rule registration**: Rules compiled-in vs dynamic module loading
3. **Zero-cost abstractions**: Trait-based dispatch with monomorphization
4. **Efficient string handling**: Zero-copy parsing where possible
5. **Optimized regex**: Rust regex crate with SIMD acceleration
6. **No GIL**: True parallelism potential (not yet utilized)

## Running Benchmarks

```bash
# Run Criterion micro-benchmarks
cargo bench

# Quick comparison (requires Python thefuck installed)
echo "Rust:" && time ./target/release/thefuck --version
echo "Python:" && time thefuck --version
```

## Test Environment

- **OS**: macOS 14.x (Sonoma)
- **CPU**: Apple Silicon (M-series)
- **Rust**: 1.75+ (release build with LTO)
- **Python**: 3.11.6
- **thefuck**: 3.32
- **Shell**: ZSH 5.9
