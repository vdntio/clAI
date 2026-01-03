# Performance Benchmarks

This document describes the performance benchmarks for clAI and how to run them.

## Overview

The benchmarks measure critical startup and performance metrics:
- **Startup time**: Target <50ms median for cold startup
- **History reading**: Target <100ms for large history files (1000+ lines)

## Running Benchmarks

### Prerequisites

Install Criterion (automatically included as dev dependency):
```bash
cargo build --release --benches --features bench
```

### Run All Benchmarks

```bash
cargo bench --features bench
```

### Run Specific Benchmark Group

```bash
# Startup benchmarks only
cargo bench --bench startup --features bench

# History benchmarks only  
cargo bench --bench startup --features bench -- history
```

### Quick Test (verify benchmarks compile)

```bash
cargo bench --bench startup --features bench -- --test
```

## Benchmark Results

After running benchmarks, results are available in:
- **HTML Reports**: `target/criterion/startup/*/report/index.html`
- **Console Output**: Summary statistics printed to terminal

### Key Metrics

- **Median**: Target <50ms for full startup
- **Mean**: Average execution time
- **P95**: 95th percentile (worst-case performance)
- **Throughput**: Operations per second

## Benchmark Details

### Startup Benchmarks

1. **parse_args**: CLI argument parsing
2. **load_config_cold**: Config loading (first access, cold cache)
3. **load_config_warm**: Config loading (cached, warm)
4. **create_config_from_cli**: Runtime config creation
5. **setup_signal_handlers**: Signal handler initialization
6. **gather_context**: Context gathering (system, directory, history, stdin)
7. **full_startup_cold**: Complete startup path (cold, all caches reset)
8. **full_startup_warm**: Complete startup path (warm, caches populated)

### History Benchmarks

1. **read_history_tail_1000_lines**: Efficient tail read from large history file

## Performance Targets

- **Cold Startup**: <50ms median (from program start to context ready)
- **Warm Startup**: <10ms median (with all caches populated)
- **History Read**: <100ms for 1000+ line files

## Optimization Notes

The following optimizations are already implemented:

- ✅ **Lazy Config Loading**: Config loaded only on first access
- ✅ **System Info Caching**: System information cached per run
- ✅ **Pre-compiled Regexes**: All regex patterns compiled once at startup
- ✅ **Efficient History Read**: Tail-based reading (last 4096 bytes)

## Release Build Configuration

The release profile is optimized for performance:

```toml
[profile.release]
codegen-units = 1    # Better optimization
lto = true          # Link-time optimization
panic = "abort"     # Smaller binary
opt-level = 3       # Maximum optimization
strip = true        # Remove debug symbols
```

## Continuous Benchmarking

For CI/CD integration, use:

```bash
# Run benchmarks and save results
cargo bench --features bench -- --save-baseline main

# Compare against baseline
cargo bench --features bench -- --baseline main
```

## Troubleshooting

### Gnuplot Not Found

If you see "Gnuplot not found", Criterion will use the plotters backend instead. This is fine - all functionality works without Gnuplot.

### Benchmark Takes Too Long

Adjust sample size in `benches/startup.rs`:
```rust
group.sample_size(50);  // Reduce from 100 for faster runs
```

