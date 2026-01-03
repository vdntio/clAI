//! Performance benchmarks for clAI startup and critical paths
//! 
//! Targets:
//! - Cold startup: <50ms median
//! - History reading: <100ms for large files
//! 
//! Run with: `cargo bench --features bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use clai::cli::Cli;
use clai::config::{get_file_config, Config};
use clai::context::gatherer::gather_context;
use clai::signals::setup_signal_handlers;
use std::time::Instant;

/// Benchmark cold startup: parsing args, loading config, and gathering context
/// 
/// This measures the critical path from program start to first context ready.
/// Target: <50ms median
fn benchmark_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    
    // Set sample size and measurement time for startup benchmarks
    group.sample_size(100);
    group.measurement_time(std::time::Duration::from_secs(10));
    
    // Benchmark: CLI parsing
    group.bench_function("parse_args", |b| {
        b.iter(|| {
            // Simulate parsing CLI args - create Cli directly (faster than parsing)
            let _cli = Cli {
                instruction: "list files".to_string(),
                model: None,
                provider: None,
                quiet: false,
                verbose: 0,
                no_color: false,
                color: clai::cli::ColorChoice::Auto,
                interactive: false,
                force: false,
                dry_run: false,
                context: None,
                offline: false,
                num_options: 3,
            };
        });
    });
    
    // Benchmark: Config loading (lazy, first access)
    group.bench_function("load_config_cold", |b| {
        let cli = Cli {
            instruction: "test instruction".to_string(),
            model: None,
            provider: None,
            quiet: false,
            verbose: 0,
            no_color: false,
            color: clai::cli::ColorChoice::Auto,
            interactive: false,
            force: false,
            dry_run: false,
            context: None,
            offline: false,
            num_options: 3,
        };
        
        b.iter(|| {
            // Reset cache for each iteration to measure cold load
            clai::config::cache::reset_config_cache();
            let _config = get_file_config(black_box(&cli));
        });
    });
    
    // Benchmark: Config loading (warm - cached)
    group.bench_function("load_config_warm", |b| {
        let cli = Cli {
            instruction: "test instruction".to_string(),
            model: None,
            provider: None,
            quiet: false,
            verbose: 0,
            no_color: false,
            color: clai::cli::ColorChoice::Auto,
            interactive: false,
            force: false,
            dry_run: false,
            context: None,
            offline: false,
            num_options: 3,
        };
        
        // Pre-warm cache
        let _ = get_file_config(&cli);
        
        b.iter(|| {
            let _config = get_file_config(black_box(&cli));
        });
    });
    
    // Benchmark: Config creation from CLI
    group.bench_function("create_config_from_cli", |b| {
        b.iter(|| {
            let cli = Cli {
                instruction: "test instruction".to_string(),
                model: None,
                provider: None,
                quiet: false,
                verbose: 0,
                no_color: false,
                color: clai::cli::ColorChoice::Auto,
                interactive: false,
                force: false,
                dry_run: false,
                context: None,
                offline: false,
                num_options: 3,
            };
            
            let _config = Config::from_cli(black_box(cli));
        });
    });
    
    // Benchmark: Signal handler setup
    group.bench_function("setup_signal_handlers", |b| {
        b.iter(|| {
            let _flag = setup_signal_handlers();
        });
    });
    
    // Benchmark: Context gathering (cold start)
    group.bench_function("gather_context", |b| {
        b.iter(|| {
            // Reset system info cache for cold start measurement
            // Note: System info cache is internal, so we measure with cache
            let config = Config {
                instruction: "test".to_string(),
                model: None,
                provider: None,
                quiet: false,
                verbose: 0,
                no_color: false,
                color: clai::cli::ColorChoice::Auto,
                interactive: false,
                force: false,
                dry_run: false,
                context: None,
                offline: false,
                num_options: 3,
            };
            
            let _context = gather_context(black_box(&config));
        });
    });
    
    // Benchmark: Full startup path (cold)
    group.bench_function("full_startup_cold", |b| {
        b.iter(|| {
            // Reset caches for true cold start
            clai::config::cache::reset_config_cache();
            
            let start = Instant::now();
            
            // 1. Parse args (simulated)
            let cli = Cli {
                instruction: "test instruction".to_string(),
                model: None,
                provider: None,
                quiet: false,
                verbose: 0,
                no_color: false,
                color: clai::cli::ColorChoice::Auto,
                interactive: false,
                force: false,
                dry_run: false,
                context: None,
                offline: false,
                num_options: 3,
            };
            
            // 2. Setup signal handlers
            let _interrupt_flag = setup_signal_handlers();
            
            // 3. Load config (lazy, first access)
            let _file_config = get_file_config(&cli);
            
            // 4. Create runtime config
            let config = Config::from_cli(cli);
            
            // 5. Gather context (critical path)
            let _context = gather_context(&config);
            
            let elapsed = start.elapsed();
            
            // Assert startup is <50ms (target)
            // Note: This is informational - criterion will report actual times
            black_box(elapsed);
        });
    });
    
    // Benchmark: Full startup path (warm - with caches)
    group.bench_function("full_startup_warm", |b| {
        // Pre-warm caches
        let cli = Cli {
            instruction: "warmup".to_string(),
            model: None,
            provider: None,
            quiet: false,
            verbose: 0,
            no_color: false,
            color: clai::cli::ColorChoice::Auto,
            interactive: false,
            force: false,
            dry_run: false,
            context: None,
            offline: false,
            num_options: 3,
        };
        let _ = get_file_config(&cli);
        let config = Config::from_cli(cli.clone());
        let _ = gather_context(&config);
        
        b.iter(|| {
            let start = Instant::now();
            
            // 1. Parse args (simulated)
            let cli = Cli {
                instruction: "test instruction".to_string(),
                model: None,
                provider: None,
                quiet: false,
                verbose: 0,
                no_color: false,
                color: clai::cli::ColorChoice::Auto,
                interactive: false,
                force: false,
                dry_run: false,
                context: None,
                offline: false,
                num_options: 3,
            };
            
            // 2. Setup signal handlers
            let _interrupt_flag = setup_signal_handlers();
            
            // 3. Load config (cached)
            let _file_config = get_file_config(&cli);
            
            // 4. Create runtime config
            let config = Config::from_cli(cli);
            
            // 5. Gather context (cached system info)
            let _context = gather_context(&config);
            
            let elapsed = start.elapsed();
            black_box(elapsed);
        });
    });
    
    group.finish();
}

/// Benchmark history reading performance
/// 
/// Measures tail read performance for large history files.
/// Target: <100ms for large files
fn benchmark_history_reading(c: &mut Criterion) {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    
    let mut group = c.benchmark_group("history");
    group.sample_size(50);
    
    // Create a large history file (1000+ lines)
    let mut temp_file = NamedTempFile::new().unwrap();
    for i in 1..=1000 {
        writeln!(temp_file, "command_{}", i).unwrap();
    }
    temp_file.flush().unwrap();
    let history_path = PathBuf::from(temp_file.path());
    
    group.bench_function("read_history_tail_1000_lines", |b| {
        b.iter(|| {
            let _history = clai::context::history::read_history_tail(
                black_box(&history_path),
                black_box(100),
            );
        });
    });
    
    // Cleanup
    drop(temp_file);
}

criterion_group!(benches, benchmark_startup, benchmark_history_reading);
criterion_main!(benches);

