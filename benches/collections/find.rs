use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use dashmap::DashMap;
use indexmap::IndexMap;
use rand::{Rng, SeedableRng};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::collections::VecDeque;
use std::collections::{BTreeMap, HashMap};

fn vec_find(data: &[(usize, Vec<u8>)], id: usize) {
    data.iter().find(|(i, _)| i == &id);
}

fn hashmap_find(data: &HashMap<usize, Vec<u8>>, id: usize) {
    data.get(&id);
}

fn btree_find(data: &BTreeMap<usize, Vec<u8>>, id: usize) {
    data.get(&id);
}

fn dashmap_find(data: &DashMap<usize, Vec<u8>>, id: usize) {
    data.get(&id);
}

fn indexmap_find(data: &IndexMap<usize, Vec<u8>>, id: usize) {
    data.get(&id);
}

fn smallvec_find_1(data: &SmallVec<[(usize, Vec<u8>); 1]>, id: usize) {
    data.iter().find(|(i, _)| i == &id);
}

fn smallvec_find_4(data: &SmallVec<[(usize, Vec<u8>); 4]>, id: usize) {
    data.iter().find(|(i, _)| i == &id);
}

fn smallvec_find_8(data: &SmallVec<[(usize, Vec<u8>); 8]>, id: usize) {
    data.iter().find(|(i, _)| i == &id);
}

fn smallvec_find_16(data: &SmallVec<[(usize, Vec<u8>); 16]>, id: usize) {
    data.iter().find(|(i, _)| i == &id);
}

fn smallvec_find_32(data: &SmallVec<[(usize, Vec<u8>); 32]>, id: usize) {
    data.iter().find(|(i, _)| i == &id);
}

fn vecdeque_find(data: &VecDeque<(usize, Vec<u8>)>, id: usize) {
    data.iter().find(|(i, _)| i == &id);
}

fn fxhashmap_find(data: &FxHashMap<usize, Vec<u8>>, id: usize) {
    data.get(&id);
}

fn vec_vs_hashmap(c: &mut Criterion) {
    let data_sizes = [
        8 * 1024,         // 8 KB
        512 * 1024,       // 512 KB
        16 * 1024 * 1024, // 16 MB
    ];
    let data_lengths = [10, 100, 1000];

    let mut group = c.benchmark_group("collections-find");
    let mut rng = rand::rngs::StdRng::seed_from_u64(64);

    for &data_size in &data_sizes {
        let data = vec![1u8; data_size];
        for data_len in data_lengths {
            let index_to_find = (rng.random::<u64>() % data_len as u64) as usize;

            //  we create a vector of data
            let vec_data = (0..data_len).map(|i| (i, data.clone())).collect::<Vec<_>>();

            //  we create a hashmap of data
            let hashmap_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<HashMap<_, _>>();

            //  we create a btree of data
            let btree_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<BTreeMap<_, _>>();

            //  we create a dashmap of data
            let dashmap_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<DashMap<_, _>>();

            //  we create a indexmap of data
            let indexmap_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<IndexMap<_, _>>();

            //  we create a smallvec of data
            let smallvec_1_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<SmallVec<[(usize, Vec<u8>); 1]>>();
            let smallvec_4_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<SmallVec<[(usize, Vec<u8>); 4]>>();
            let smallvec_8_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<SmallVec<[(usize, Vec<u8>); 8]>>();
            let smallvec_16_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<SmallVec<[(usize, Vec<u8>); 16]>>();
            let smallvec_32_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<SmallVec<[(usize, Vec<u8>); 32]>>();

            //  we create a vecdeque of data
            let vecdeque_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<VecDeque<_>>();

            //  we create a fxhashmap of data
            let fxhashmap_data = (0..data_len)
                .map(|i| (i, data.clone()))
                .collect::<FxHashMap<_, _>>();

            //  we create a benchmark for the vector
            group.bench_with_input(
                BenchmarkId::new("Vec Find", format!("{}KBx{data_len}", data_size / 1024)),
                &(vec_data, index_to_find),
                |b, (vec_data, index_to_find)| {
                    b.iter(|| vec_find(black_box(vec_data), black_box(*index_to_find)));
                },
            );

            //  we create a benchmark for the hashmap
            group.bench_with_input(
                BenchmarkId::new("HashMap Find", format!("{}KBx{data_len}", data_size / 1024)),
                &(hashmap_data, index_to_find),
                |b, (hashmap_data, index_to_find)| {
                    b.iter(|| hashmap_find(black_box(hashmap_data), black_box(*index_to_find)));
                },
            );

            //  we create a benchmark for the btree
            group.bench_with_input(
                BenchmarkId::new(
                    "BTreeMap Find",
                    format!("{}KBx{data_len}", data_size / 1024),
                ),
                &(btree_data, index_to_find),
                |b, (btree_data, index_to_find)| {
                    b.iter(|| btree_find(black_box(btree_data), black_box(*index_to_find)));
                },
            );

            //  we create a benchmark for the dashmap
            group.bench_with_input(
                BenchmarkId::new("DashMap Find", format!("{}KBx{data_len}", data_size / 1024)),
                &(dashmap_data, index_to_find),
                |b, (dashmap_data, index_to_find)| {
                    b.iter(|| dashmap_find(black_box(dashmap_data), black_box(*index_to_find)));
                },
            );
            //  we create a benchmark for the indexmap
            group.bench_with_input(
                BenchmarkId::new(
                    "IndexMap Find",
                    format!("{}KBx{data_len}", data_size / 1024),
                ),
                &(indexmap_data, index_to_find),
                |b, (indexmap_data, index_to_find)| {
                    b.iter(|| indexmap_find(black_box(indexmap_data), black_box(*index_to_find)));
                },
            );

            //  we create a benchmark for the smallvec
            group.bench_with_input(
                BenchmarkId::new(
                    "SmallVec<1> Find",
                    format!("{}KBx{data_len}", data_size / 1024),
                ),
                &(smallvec_1_data, index_to_find),
                |b, (smallvec_1_data, index_to_find)| {
                    b.iter(|| {
                        smallvec_find_1(black_box(smallvec_1_data), black_box(*index_to_find))
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new(
                    "SmallVec<4> Find",
                    format!("{}KBx{data_len}", data_size / 1024),
                ),
                &(smallvec_4_data, index_to_find),
                |b, (smallvec_4_data, index_to_find)| {
                    b.iter(|| {
                        smallvec_find_4(black_box(smallvec_4_data), black_box(*index_to_find))
                    });
                },
            );
            group.bench_with_input(
                BenchmarkId::new(
                    "SmallVec<8> Find",
                    format!("{}KBx{data_len}", data_size / 1024),
                ),
                &(smallvec_8_data, index_to_find),
                |b, (smallvec_8_data, index_to_find)| {
                    b.iter(|| {
                        smallvec_find_8(black_box(smallvec_8_data), black_box(*index_to_find))
                    });
                },
            );
            group.bench_with_input(
                BenchmarkId::new(
                    "SmallVec<16> Find",
                    format!("{}KBx{data_len}", data_size / 1024),
                ),
                &(smallvec_16_data, index_to_find),
                |b, (smallvec_16_data, index_to_find)| {
                    b.iter(|| {
                        smallvec_find_16(black_box(smallvec_16_data), black_box(*index_to_find))
                    });
                },
            );
            group.bench_with_input(
                BenchmarkId::new(
                    "SmallVec<32> Find",
                    format!("{}KBx{data_len}", data_size / 1024),
                ),
                &(smallvec_32_data, index_to_find),
                |b, (smallvec_32_data, index_to_find)| {
                    b.iter(|| {
                        smallvec_find_32(black_box(smallvec_32_data), black_box(*index_to_find))
                    });
                },
            );
            //  we create a benchmark for the vecdeque
            group.bench_with_input(
                BenchmarkId::new(
                    "VecDeque Find",
                    format!("{}KBx{data_len}", data_size / 1024),
                ),
                &(vecdeque_data, index_to_find),
                |b, (vecdeque_data, index_to_find)| {
                    b.iter(|| vecdeque_find(black_box(vecdeque_data), black_box(*index_to_find)));
                },
            );
            //  we create a benchmark for the fxhashmap
            group.bench_with_input(
                BenchmarkId::new(
                    "FxHashMap Find",
                    format!("{}KBx{data_len}", data_size / 1024),
                ),
                &(fxhashmap_data, index_to_find),
                |b, (fxhashmap_data, index_to_find)| {
                    b.iter(|| fxhashmap_find(black_box(fxhashmap_data), black_box(*index_to_find)));
                },
            );
        }
    }

    group.finish();
}

criterion_group!(collections_find, vec_vs_hashmap);
criterion_main!(collections_find);
