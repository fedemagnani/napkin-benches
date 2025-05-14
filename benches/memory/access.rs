use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rand::{prelude::*, rng};

fn sequential_access(data: &mut [u8]) {
    for d in data {
        *d = d.wrapping_add(1);
    }
}

fn random_access(data: &mut [u8], indices: &[usize]) {
    for &i in indices {
        data[i] = data[i].wrapping_add(1);
    }
}

fn stride_access(data: &mut [u8], stride: usize) {
    let len = data.len();
    for i in (0..len).step_by(stride) {
        data[i] = data[i].wrapping_add(1);
    }
}

fn memory_access_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory-access");

    // Define data sizes (in bytes)
    let sizes = [
        8 * 1024,         // 8 KB
        64 * 1024,        // 64 KB
        512 * 1024,       // 512 KB
        4 * 1024 * 1024,  // 4 MB
        16 * 1024 * 1024, // 16 MB
    ];

    for &size in &sizes {
        // Prepare data
        let mut data = vec![0u8; size];

        // Sequential Access
        group.bench_with_input(BenchmarkId::new("Sequential", size), &size, |b, &_size| {
            b.iter(|| sequential_access(black_box(&mut data)));
        });

        // Random Access
        let mut rng = rng();
        let mut indices: Vec<usize> = (0..data.len()).collect();
        indices.shuffle(&mut rng);

        group.bench_with_input(BenchmarkId::new("Random", size), &size, |b, &_size| {
            b.iter(|| random_access(black_box(&mut data), black_box(&indices)));
        });

        // Stride Access with stride of 64 bytes
        let stride = 64;
        group.bench_with_input(BenchmarkId::new("Stride", size), &size, |b, &_size| {
            b.iter(|| stride_access(black_box(&mut data), black_box(stride)));
        });
    }

    group.finish();
}

criterion_group!(memory, memory_access_benchmark);
criterion_main!(memory);
