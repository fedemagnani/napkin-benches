use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use dashmap::DashMap;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use std::sync::Arc;
use tokio::{runtime::Builder, sync::Mutex};
use tokio_stream::{
    StreamExt,
    wrappers::{BroadcastStream, ReceiverStream, UnboundedReceiverStream},
};

pub type Data = Vec<u8>;

async fn arc_mutex(
    num_tasks: usize,
    mut handles: Vec<tokio::task::JoinHandle<()>>,
    values: Vec<(usize, Data)>,
    mut indices_per_task: Vec<Vec<usize>>,
) {
    let counter = Arc::new(Mutex::new(values));

    for _ in 0..num_tasks {
        let counter = Arc::clone(&counter);
        let indices = indices_per_task.pop().unwrap();
        handles.push(tokio::spawn(async move {
            // we search each index and we set the data zero
            for i in indices {
                let mut guard = counter.lock().await;
                let (_, data) = guard.iter_mut().find(|(index, _)| *index == i).unwrap();
                let len = data.len();
                *data = vec![0; len];
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

async fn unbounded(
    num_tasks: usize,
    mut values: Vec<(usize, Data)>,
    mut indices_per_task: Vec<Vec<usize>>,
) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    for _ in 0..num_tasks {
        let tx = tx.clone();
        let indices = indices_per_task.pop().unwrap();
        tokio::spawn(async move {
            for i in indices {
                let (one_tx, one_rx) = tokio::sync::oneshot::channel();
                tx.send((i, one_tx)).unwrap();
                one_rx.await.unwrap();
            }
        });
    }

    drop(tx); // kill last sender

    while let Some((i, one_tx)) = rx.next().await {
        let (_, data) = values.iter_mut().find(|(index, _)| *index == i).unwrap();
        let len = data.len();
        *data = vec![0; len];
        one_tx.send(()).unwrap();
    }
}

async fn mpsc(
    num_tasks: usize,
    mut values: Vec<(usize, Data)>,
    mut indices_per_task: Vec<Vec<usize>>,
    capacity: usize,
) {
    let (tx, rx) = tokio::sync::mpsc::channel(capacity);
    let mut rx = ReceiverStream::new(rx);

    for _ in 0..num_tasks {
        let tx = tx.clone();
        let indices = indices_per_task.pop().unwrap();
        tokio::spawn(async move {
            for i in indices {
                let (one_tx, one_rx) = tokio::sync::oneshot::channel();
                tx.send((i, one_tx)).await.ok();
                one_rx.await.unwrap();
            }
        });
    }

    drop(tx); // kill last sender

    while let Some((i, one_tx)) = rx.next().await {
        let (_, data) = values.iter_mut().find(|(index, _)| *index == i).unwrap();
        let len = data.len();
        *data = vec![0; len];
        one_tx.send(()).unwrap();
    }
}

async fn broadcast(
    num_tasks: usize,
    mut values: Vec<(usize, Data)>,
    mut indices_per_task: Vec<Vec<usize>>,
) {
    let (tx, rx) = tokio::sync::broadcast::channel(10_000_000);
    let mut rx = BroadcastStream::new(rx);

    for _ in 0..num_tasks {
        let tx = tx.clone();
        let indices = indices_per_task.pop().unwrap();
        tokio::spawn(async move {
            for i in indices {
                let (one_tx, mut one_rx) = tokio::sync::mpsc::channel(1);
                tx.send((i, one_tx)).ok();
                one_rx.recv().await;
            }
        });
    }

    drop(tx); // kill last sender

    while let Some(Ok((i, one_tx))) = rx.next().await {
        let (_, data) = values.iter_mut().find(|(index, _)| *index == i).unwrap();
        let len = data.len();
        *data = vec![0; len];
        one_tx.send(()).await.unwrap();
    }
}

async fn dash_map(
    num_tasks: usize,
    mut handles: Vec<tokio::task::JoinHandle<()>>,
    values: Vec<(usize, Data)>,
    mut indices_per_task: Vec<Vec<usize>>,
) {
    let dash = Arc::new(
        values
            .iter()
            .map(|(i, v)| (*i, v.clone()))
            .collect::<DashMap<_, _>>(),
    );

    for _ in 0..num_tasks {
        let indices = indices_per_task.pop().unwrap();
        let dash = dash.clone();
        handles.push(tokio::spawn(async move {
            // we search each index and we set the data zero
            for i in indices {
                let mut data = dash.get_mut(&i).unwrap();
                let len = data.len();
                *data = vec![0; len];
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

// Benchmark using AtomicUsize
fn find(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(64);

    let num_tasks = [10, 100, 1_000];
    let indices_per_task = [100, 1_000, 10_000];
    let channels_capacities = [1, 100];

    let mut group = c.benchmark_group("tokio-find");

    for n in num_tasks.into_iter() {
        for ind in indices_per_task.into_iter() {
            let param = format!("{n}t{ind}");

            let random_ids = (0..ind)
                .map(|_| rng.random_range(0..1_000_000) as usize)
                .collect::<Vec<_>>();

            let mut values = random_ids.clone();
            values.shuffle(&mut rng);
            let values = values
                .iter()
                .map(|i| (*i, vec![0; 100]))
                .collect::<Vec<_>>();

            let indices_per_task = (0..n)
                .map(|_| {
                    let mut indices = random_ids.clone();
                    indices.shuffle(&mut rng);
                    indices
                })
                .collect::<Vec<_>>();

            group.bench_with_input(
                BenchmarkId::new("arc_mutex", param.clone()),
                &(n, values.clone(), indices_per_task.clone()),
                |b, (n, v, i)| {
                    b.to_async(&rt)
                        .iter(|| arc_mutex(*n, Vec::with_capacity(*n), v.clone(), i.clone()));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("dashmap", param.clone()),
                &(n, values.clone(), indices_per_task.clone()),
                |b, (n, v, i)| {
                    b.to_async(&rt)
                        .iter(|| dash_map(*n, Vec::with_capacity(*n), v.clone(), i.clone()));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("unbounded_ch", param.clone()),
                &(n, values.clone(), indices_per_task.clone()),
                |b, (n, v, i)| {
                    b.to_async(&rt).iter(|| unbounded(*n, v.clone(), i.clone()));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("broadcast_ch", param.clone()),
                &(n, values.clone(), indices_per_task.clone()),
                |b, (n, v, i)| {
                    b.to_async(&rt).iter(|| broadcast(*n, v.clone(), i.clone()));
                },
            );

            for cap in channels_capacities.into_iter() {
                let param = format!("{n}t{ind}c{cap}");

                group.bench_with_input(
                    BenchmarkId::new("mpsc_ch", param.clone()),
                    &(n, values.clone(), indices_per_task.clone()),
                    |b, (n, v, i)| {
                        b.to_async(&rt).iter(|| mpsc(*n, v.clone(), i.clone(), cap));
                    },
                );
            }
        }
    }
}

criterion_group!(benches, find);
criterion_main!(benches);
