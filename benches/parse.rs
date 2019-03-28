extern crate core_affinity;
extern crate jemallocator;
#[macro_use]
extern crate criterion;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use criterion::{BatchSize, Benchmark, Criterion, ParameterizedBenchmark, Throughput};
use serde_json;
use simdjson;
use std::fs::File;
use std::io::{self, Read, Write};

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

            let len = vec.len();

            c.bench(
                stringify!($name),
                ParameterizedBenchmark::new(
                    "simdjson",
                    |b, data| {
                        b.iter_batched(
                            || data.clone(),
                            |mut bytes| {
                                simdjson::to_value(&mut bytes).unwrap();
                            },
                            BatchSize::SmallInput,
                        )
                    },
                    vec![vec],
                )
                .with_function("serde_json", move |b, data| {
                    b.iter_batched(
                        || data.clone(),
                        |mut bytes| {
                            let _: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                        },
                        BatchSize::SmallInput,
                    )
                })
                .throughput(|data| Throughput::Bytes(data.len() as u32)),
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
