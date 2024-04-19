use std::hint::black_box;

use bitvec::vec::BitVec;
use bonsai_trie::{
    databases::HashMapDb, id::{BasicId, BasicIdBuilder}, BatchedUpdateItem, BonsaiStorage, BonsaiStorageConfig
};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{prelude::*, thread_rng};
use starknet_types_core::{
    felt::Felt,
    hash::{Pedersen, StarkHash},
};
use rayon::prelude::*;

mod flamegraph;

fn storage_with_insert(c: &mut Criterion) {
    c.bench_function("storage commit with insert", move |b| {
        let mut rng = thread_rng();
        b.iter_with_large_drop(
            || {
                let mut bonsai_storage: BonsaiStorage<BasicId, _, Pedersen> = BonsaiStorage::new(
                    HashMapDb::<BasicId>::default(),
                    BonsaiStorageConfig::default(),
                )
                .unwrap();

                let felt = Felt::from_hex("0x66342762FDD54D033c195fec3ce2568b62052e").unwrap();
                for _ in 0..4000 {
                    let bitvec = BitVec::from_vec(vec![
                        rng.gen(),
                        rng.gen(),
                        rng.gen(),
                        rng.gen(),
                        rng.gen(),
                        rng.gen(),
                    ]);
                    bonsai_storage.insert(&[], &bitvec, &felt).unwrap();
                }

                let mut id_builder = BasicIdBuilder::new();
                bonsai_storage.commit(id_builder.new_id()).unwrap();

                bonsai_storage
            },
        );
    });
}

fn batched_update(c: &mut Criterion) {
    c.bench_function("storage commit with batched insert", move |b| {
        b.iter_with_large_drop(
            || {
                let mut bonsai_storage: BonsaiStorage<BasicId, _, Pedersen> = BonsaiStorage::new(
                    HashMapDb::<BasicId>::default(),
                    BonsaiStorageConfig::default(),
                )
                .unwrap();

                let felt = Felt::from_hex("0x66342762FDD54D033c195fec3ce2568b62052e").unwrap();
                let mut id_builder = BasicIdBuilder::new();

                // TODO(merge): we need to commit an empty commit first because otherwise we don't have a reference id
                //              ask aurelien why the api is that way
                let id1 = id_builder.new_id();
                bonsai_storage.commit(id1).unwrap();

                bonsai_storage.batched_update(id1, (0..40000).into_par_iter().map(|_| {
                    let mut rng = thread_rng();
                    let bitvec = BitVec::from_vec(vec![
                        rng.gen(),
                        rng.gen(),
                        rng.gen(),
                        rng.gen(),
                        rng.gen(),
                        rng.gen(),
                    ]);
                    BatchedUpdateItem::SetValue { identifier: vec![], key: bitvec, value: felt }
                })).unwrap();
                let id2 = id_builder.new_id();
                bonsai_storage.commit(id2).unwrap();

                bonsai_storage
            },
        );
    });
}

fn storage(c: &mut Criterion) {
    c.bench_function("storage commit", move |b| {
        let mut bonsai_storage: BonsaiStorage<BasicId, _, Pedersen> = BonsaiStorage::new(
            HashMapDb::<BasicId>::default(),
            BonsaiStorageConfig::default(),
        )
        .unwrap();
        let mut rng = thread_rng();

        let felt = Felt::from_hex("0x66342762FDD54D033c195fec3ce2568b62052e").unwrap();
        for _ in 0..1000 {
            let bitvec = BitVec::from_vec(vec![
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
            ]);
            bonsai_storage.insert(&[], &bitvec, &felt).unwrap();
        }

        let mut id_builder = BasicIdBuilder::new();
        b.iter_batched(
            || bonsai_storage.clone(),
            |mut bonsai_storage| {
                bonsai_storage.commit(id_builder.new_id()).unwrap();
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

fn one_update(c: &mut Criterion) {
    c.bench_function("one update", move |b| {
        let mut bonsai_storage: BonsaiStorage<BasicId, _, Pedersen> = BonsaiStorage::new(
            HashMapDb::<BasicId>::default(),
            BonsaiStorageConfig::default(),
        )
        .unwrap();
        let mut rng = thread_rng();

        let felt = Felt::from_hex("0x66342762FDD54D033c195fec3ce2568b62052e").unwrap();
        for _ in 0..1000 {
            let bitvec = BitVec::from_vec(vec![
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
            ]);
            bonsai_storage.insert(&[], &bitvec, &felt).unwrap();
        }

        let mut id_builder = BasicIdBuilder::new();
        bonsai_storage.commit(id_builder.new_id()).unwrap();

        b.iter_batched(
            || bonsai_storage.clone(),
            |mut bonsai_storage| {
                let bitvec = BitVec::from_vec(vec![0, 1, 2, 3, 4, 5]);
                bonsai_storage.insert(&[], &bitvec, &felt).unwrap();
                bonsai_storage.commit(id_builder.new_id()).unwrap();
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

fn five_updates(c: &mut Criterion) {
    c.bench_function("five updates", move |b| {
        let mut bonsai_storage: BonsaiStorage<BasicId, _, Pedersen> = BonsaiStorage::new(
            HashMapDb::<BasicId>::default(),
            BonsaiStorageConfig::default(),
        )
        .unwrap();
        let mut rng = thread_rng();

        let felt = Felt::from_hex("0x66342762FDD54D033c195fec3ce2568b62052e").unwrap();
        for _ in 0..1000 {
            let bitvec = BitVec::from_vec(vec![
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
            ]);
            bonsai_storage.insert(&[], &bitvec, &felt).unwrap();
        }

        let mut id_builder = BasicIdBuilder::new();
        bonsai_storage.commit(id_builder.new_id()).unwrap();

        b.iter_batched(
            || bonsai_storage.clone(),
            |mut bonsai_storage| {
                bonsai_storage
                    .insert(&[], &BitVec::from_vec(vec![0, 1, 2, 3, 4, 5]), &felt)
                    .unwrap();
                bonsai_storage
                    .insert(&[], &BitVec::from_vec(vec![0, 2, 2, 5, 4, 5]), &felt)
                    .unwrap();
                bonsai_storage
                    .insert(&[], &BitVec::from_vec(vec![0, 1, 2, 3, 3, 5]), &felt)
                    .unwrap();
                bonsai_storage
                    .insert(&[], &BitVec::from_vec(vec![0, 1, 1, 3, 99, 3]), &felt)
                    .unwrap();
                bonsai_storage
                    .insert(&[], &BitVec::from_vec(vec![0, 1, 2, 3, 4, 6]), &felt)
                    .unwrap();
                bonsai_storage.commit(id_builder.new_id()).unwrap();
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

fn hash(c: &mut Criterion) {
    c.bench_function("pedersen hash", move |b| {
        let felt0 =
            Felt::from_hex("0x100bd6fbfced88ded1b34bd1a55b747ce3a9fde9a914bca75571e4496b56443")
                .unwrap();
        let felt1 =
            Felt::from_hex("0x00a038cda302fedbc4f6117648c6d3faca3cda924cb9c517b46232c6316b152f")
                .unwrap();
        b.iter(|| {
            black_box(Pedersen::hash(&felt0, &felt1));
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default(); // .with_profiler(flamegraph::FlamegraphProfiler::new(100));
    targets = storage, one_update, five_updates, hash, storage_with_insert, batched_update
}
criterion_main!(benches);
