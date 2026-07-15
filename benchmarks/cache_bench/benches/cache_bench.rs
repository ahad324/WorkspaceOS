use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use lru::LruCache;
use moka::sync::Cache as MokaCache;
use std::num::NonZeroUsize;
use std::time::Instant;

// Helper to generate Zipfian-like skewed distribution access sequence
fn generate_zipf_keys(num_requests: usize, num_keys: usize, s: f64) -> Vec<usize> {
    let mut keys = Vec::with_capacity(num_requests);
    let mut rng = 123456789usize; // Simple LCG for determinism
    
    // Precompute cumulative probabilities
    let mut c = 0.0;
    for i in 1..=num_keys {
        c += 1.0 / (i as f64).powf(s);
    }
    
    for _ in 0..num_requests {
        // LCG RNG
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let rand_val = ((rng & 0x7FFFFFFF) as f64) / 2147483647.0;
        
        let mut sum = 0.0;
        let mut key = 0;
        for i in 1..=num_keys {
            sum += (1.0 / (i as f64).powf(s)) / c;
            if rand_val <= sum {
                key = i;
                break;
            }
        }
        if key == 0 {
            key = num_keys;
        }
        keys.push(key);
    }
    keys
}

// Helper to generate a sequential scan sequence (cache pollution scenario)
fn generate_scan_keys(num_requests: usize) -> Vec<usize> {
    (0..num_requests).collect()
}

fn bench_caches(c: &mut Criterion) {
    let request_count = 5000;
    let key_space = 500;
    let cache_capacity = 100;

    // Generate test workloads
    let zipf_keys = generate_zipf_keys(request_count, key_space, 1.0);
    let scan_keys = generate_scan_keys(request_count);

    let mut group = c.benchmark_group("Cache Performance");

    // 1. Skewed Zipfian reads
    group.bench_function(BenchmarkId::new("LRU - Zipfian Reads", cache_capacity), |b| {
        b.iter_custom(|iters| {
            let mut elapsed = std::time::Duration::ZERO;
            for _ in 0..iters {
                let mut cache = LruCache::new(NonZeroUsize::new(cache_capacity).unwrap());
                // Warm up cache
                for &k in &zipf_keys {
                    cache.put(k, k);
                }
                
                let start = Instant::now();
                for &k in &zipf_keys {
                    let _ = cache.get(&k);
                }
                elapsed += start.elapsed();
            }
            elapsed
        });
    });

    group.bench_function(BenchmarkId::new("Moka W-TinyLFU - Zipfian Reads", cache_capacity), |b| {
        b.iter_custom(|iters| {
            let mut elapsed = std::time::Duration::ZERO;
            for _ in 0..iters {
                let cache = MokaCache::new(cache_capacity as u64);
                // Warm up cache
                for &k in &zipf_keys {
                    cache.insert(k, k);
                }
                // Moka performs updates asynchronously, so we wait briefly or let it run
                
                let start = Instant::now();
                for &k in &zipf_keys {
                    let _ = cache.get(&k);
                }
                elapsed += start.elapsed();
            }
            elapsed
        });
    });

    // 2. Scan Pollution reads (Cache pollution scenario)
    group.bench_function(BenchmarkId::new("LRU - Scan Pollution", cache_capacity), |b| {
        b.iter_custom(|iters| {
            let mut elapsed = std::time::Duration::ZERO;
            for _ in 0..iters {
                let mut cache = LruCache::new(NonZeroUsize::new(cache_capacity).unwrap());
                // Put initial hot keys
                for k in 0..cache_capacity {
                    cache.put(k, k);
                }

                let start = Instant::now();
                // Perform a scanning sweep that exceeds cache size
                for &k in &scan_keys {
                    cache.put(k, k);
                }
                elapsed += start.elapsed();
            }
            elapsed
        });
    });

    group.bench_function(BenchmarkId::new("Moka W-TinyLFU - Scan Pollution", cache_capacity), |b| {
        b.iter_custom(|iters| {
            let mut elapsed = std::time::Duration::ZERO;
            for _ in 0..iters {
                let cache = MokaCache::new(cache_capacity as u64);
                // Put initial hot keys
                for k in 0..cache_capacity {
                    cache.insert(k, k);
                }

                let start = Instant::now();
                // Perform a scanning sweep that exceeds cache size
                for &k in &scan_keys {
                    cache.insert(k, k);
                }
                elapsed += start.elapsed();
            }
            elapsed
        });
    });

    group.finish();
}

criterion_group!(benches, bench_caches);
criterion_main!(benches);
