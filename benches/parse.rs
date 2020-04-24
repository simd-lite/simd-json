#[macro_use]
extern crate criterion;

use core::time::Duration;

#[cfg(feature = "jemallocator")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(feature = "bench-serde")]
use serde_json;

use criterion::{BatchSize, Criterion, ParameterizedBenchmark, Throughput};
use simd_json;
use std::fs::File;
use std::io::Read;

macro_rules! bench_file {
    ($name:ident) => {
        fn $name(c: &mut Criterion) {
            let core_ids = core_affinity::get_core_ids().unwrap();
            core_affinity::set_for_current(core_ids[0]);

            let mut vec = Vec::new();
            File::open(concat!("data/", stringify!($name), ".json"))
                .unwrap()
                .read_to_end(&mut vec)
                .unwrap();

            let b = ParameterizedBenchmark::new(
                "simd_json::to_tape",
                |b, data| {
                    b.iter_batched(
                        || data.clone(),
                        |mut bytes| {
                            simd_json::to_tape(&mut bytes).unwrap();
                        },
                        BatchSize::SmallInput,
                    )
                },
                vec![vec],
            )
            .warm_up_time(Duration::from_secs(1))
            .measurement_time(Duration::from_secs(20));

            let b = b.with_function("simd_json::to_borrowed_value", |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut bytes| {
                        simd_json::to_borrowed_value(&mut bytes).unwrap();
                    },
                    BatchSize::SmallInput,
                )
            });

            let b = b.with_function("simd_json::to_owned_value", |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut bytes| {
                        simd_json::to_owned_value(&mut bytes).unwrap();
                    },
                    BatchSize::SmallInput,
                )
            });

            #[cfg(feature = "bench-serde")]
            let b = b.with_function("serde_json::from_slice", |b, data| {
                b.iter_batched(
                    || data,
                    |bytes| {
                        let _: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                    },
                    BatchSize::SmallInput,
                )
            });

            c.bench(
                stringify!($name),
                b.throughput(|data| Throughput::Bytes(data.len() as u64)),
            );
        }
    };
}

bench_file!(apache_builds);
bench_file!(canada);
bench_file!(citm_catalog);
bench_file!(log);
bench_file!(twitter);

criterion_group!(benches, apache_builds, canada, citm_catalog, log, twitter);
criterion_main!(benches);
