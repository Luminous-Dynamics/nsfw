/// Performance benchmarks for NSFW
///
/// Run with: cargo bench
use std::time::{Instant, Duration};
use nsfw::cache::SearchCache;
use nsfw::nix_ops::types::SearchResult;

/// Benchmark result
#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    iterations: usize,
    total_time: Duration,
    avg_time: Duration,
    ops_per_sec: f64,
}

impl BenchmarkResult {
    fn new(name: &str, iterations: usize, total_time: Duration) -> Self {
        let avg_time = total_time / iterations as u32;
        let ops_per_sec = iterations as f64 / total_time.as_secs_f64();

        Self {
            name: name.to_string(),
            iterations,
            total_time,
            avg_time,
            ops_per_sec,
        }
    }

    fn print(&self) {
        println!("Benchmark: {}", self.name);
        println!("  Iterations: {}", self.iterations);
        println!("  Total time: {:?}", self.total_time);
        println!("  Average time: {:?}", self.avg_time);
        println!("  Ops/sec: {:.2}", self.ops_per_sec);
        println!();
    }
}

/// Benchmark cache performance
fn benchmark_cache() {
    println!("\n=== Cache Benchmarks ===\n");

    // Clear cache
    SearchCache::clear();

    // Prepare test data
    let results: Vec<SearchResult> = (0..50)
        .map(|i| SearchResult {
            pname: format!("package-{}", i),
            version: "1.0.0".to_string(),
            description: format!("Test package {}", i),
        })
        .collect();

    // Benchmark: Cache write
    let iterations = 10000;
    let start = Instant::now();
    for i in 0..iterations {
        let query = format!("test-{}", i % 100);
        SearchCache::put(&query, 10, results.clone());
    }
    let elapsed = start.elapsed();
    BenchmarkResult::new("Cache Write", iterations, elapsed).print();

    // Benchmark: Cache read (hit)
    let iterations = 100000;
    let start = Instant::now();
    for i in 0..iterations {
        let query = format!("test-{}", i % 100);
        let _ = SearchCache::get(&query, 10);
    }
    let elapsed = start.elapsed();
    BenchmarkResult::new("Cache Read (Hit)", iterations, elapsed).print();

    // Benchmark: Cache read (miss)
    SearchCache::clear();
    let iterations = 100000;
    let start = Instant::now();
    for i in 0..iterations {
        let query = format!("nonexistent-{}", i);
        let _ = SearchCache::get(&query, 10);
    }
    let elapsed = start.elapsed();
    BenchmarkResult::new("Cache Read (Miss)", iterations, elapsed).print();
}

/// Benchmark path operations
fn benchmark_paths() {
    println!("\n=== Path Translation Benchmarks ===\n");

    use nsfw::path_translation::PathTranslator;

    let translator = PathTranslator::new();

    // Benchmark: Windows → Linux
    let iterations = 100000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = translator.to_linux("C:\\Users\\Test\\file.txt");
    }
    let elapsed = start.elapsed();
    BenchmarkResult::new("Windows → Linux", iterations, elapsed).print();

    // Benchmark: Linux → Windows
    let iterations = 100000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = translator.to_windows("/mnt/c/Users/Test/file.txt");
    }
    let elapsed = start.elapsed();
    BenchmarkResult::new("Linux → Windows", iterations, elapsed).print();

    // Benchmark: Detect path type
    let iterations = 100000;
    let start = Instant::now();
    for i in 0..iterations {
        let path = if i % 2 == 0 {
            "C:\\Users\\Test\\file.txt"
        } else {
            "/mnt/c/Users/Test/file.txt"
        };
        let _ = translator.detect_path_type(path);
    }
    let elapsed = start.elapsed();
    BenchmarkResult::new("Path Type Detection", iterations, elapsed).print();
}

/// Main benchmark runner
fn main() {
    println!("╔════════════════════════════════════════════════╗");
    println!("║         NSFW Performance Benchmarks            ║");
    println!("╚════════════════════════════════════════════════╝");

    benchmark_cache();
    benchmark_paths();

    println!("\n✅ All benchmarks complete!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmarks_run() {
        // Just verify benchmarks can run without panicking
        benchmark_cache();
        benchmark_paths();
    }
}
