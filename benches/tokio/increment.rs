use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::{runtime::Builder, sync::Mutex};
use tokio_stream::{
    StreamExt,
    wrappers::{BroadcastStream, ReceiverStream, UnboundedReceiverStream},
};

async fn atomic_usize(
    num_tasks: usize,
    increments_per_task: usize,
    mut handles: Vec<tokio::task::JoinHandle<()>>,
) {
    let counter = Arc::new(AtomicUsize::new(0));

    for _ in 0..num_tasks {
        let counter = counter.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..increments_per_task {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(
        counter.load(Ordering::Relaxed),
        num_tasks * increments_per_task
    );
}

async fn arc_mutex(
    num_tasks: usize,
    increments_per_task: usize,
    mut handles: Vec<tokio::task::JoinHandle<()>>,
) {
    let counter = Arc::new(Mutex::new(0usize));

    for _ in 0..num_tasks {
        let counter = Arc::clone(&counter);
        handles.push(tokio::spawn(async move {
            for _ in 0..increments_per_task {
                let mut lock = counter.lock().await;
                *lock += 1;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(*counter.lock().await, num_tasks * increments_per_task);
}

async fn unbounded(num_tasks: usize, increments_per_task: usize) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);
    let mut counter = 0;

    for _ in 0..num_tasks {
        let tx = tx.clone();
        tokio::spawn(async move {
            for _ in 0..increments_per_task {
                tx.send(()).unwrap();
            }
        });
    }
    while (rx.next().await).is_some() {
        counter += 1;
        if counter == num_tasks * increments_per_task {
            break;
        }
    }

    assert_eq!(counter, num_tasks * increments_per_task);
}

async fn mpsc(num_tasks: usize, increments_per_task: usize, capacity: usize) {
    let (tx, rx) = tokio::sync::mpsc::channel(capacity);
    let mut rx = ReceiverStream::new(rx);
    let mut counter = 0;

    for _ in 0..num_tasks {
        let tx = tx.clone();
        tokio::spawn(async move {
            for _ in 0..increments_per_task {
                loop {
                    if tx.send(()).await.is_ok() {
                        break;
                    }
                }
            }
        });
    }

    while (rx.next().await).is_some() {
        counter += 1;
        if counter == num_tasks * increments_per_task {
            break;
        }
    }

    assert_eq!(counter, num_tasks * increments_per_task);
}

async fn broadcast(num_tasks: usize, increments_per_task: usize) {
    let (tx, rx) = tokio::sync::broadcast::channel(10_000_000);
    let mut rx = BroadcastStream::new(rx);
    let mut counter = 0;

    for _ in 0..num_tasks {
        let tx = tx.clone();
        tokio::spawn(async move {
            for _ in 0..increments_per_task {
                loop {
                    if tx.send(()).is_ok() {
                        break;
                    }
                }
            }
        });
    }

    while (rx.next().await).is_some() {
        counter += 1;
        if counter == num_tasks * increments_per_task {
            break;
        }
    }

    assert_eq!(counter, num_tasks * increments_per_task);
}

// Benchmark using AtomicUsize
fn increment(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    let num_tasks = [4, 8, 16];
    let increments_per_task = [10_000, 100_000, 1_000_000];

    let channels_capacities = [1, 100];

    let mut group = c.benchmark_group("tokio-increment");

    for n in num_tasks.into_iter() {
        for inc in increments_per_task.into_iter() {
            let param = format!("{n}t{inc}");

            group.bench_with_input(
                BenchmarkId::new("atomic_usize", param.clone()),
                &(n, inc),
                |b, (n, inc)| {
                    b.to_async(&rt)
                        .iter(|| atomic_usize(*n, *inc, Vec::with_capacity(*n)));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("arc_mutex", param.clone()),
                &(n, inc),
                |b, (n, inc)| {
                    b.to_async(&rt)
                        .iter(|| arc_mutex(*n, *inc, Vec::with_capacity(*n)));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("unbounded_ch", param.clone()),
                &(n, inc),
                |b, (n, inc)| {
                    b.to_async(&rt).iter(|| unbounded(*n, *inc));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("broadcast_ch", param),
                &(n, inc),
                |b, (n, inc)| {
                    b.to_async(&rt).iter(|| broadcast(*n, *inc));
                },
            );

            for cap in channels_capacities.into_iter() {
                let param = format!("{n}t{inc}c{cap}");

                group.bench_with_input(
                    BenchmarkId::new("mpsc_ch", param.clone()),
                    &(n, inc, cap),
                    |b, (n, inc, cap)| {
                        b.to_async(&rt).iter(|| mpsc(*n, *inc, *cap));
                    },
                );
            }
        }
    }
}

criterion_group!(benches, increment);
criterion_main!(benches);
