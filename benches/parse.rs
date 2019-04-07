extern crate core_affinity;
#[macro_use]
extern crate criterion;

#[cfg(feature = "jemallocator")]
extern crate jemallocator;
#[cfg(feature = "jemallocator")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use criterion::{BatchSize, Criterion, ParameterizedBenchmark, Throughput};
#[cfg(feature = "bench-serder")]
use serde_json;
use simdjson;
#[cfg(feature = "simdjson-rust")]
use simdjson_rust::build_parsed_json;
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
            );
            let b = b.with_function("simdjson-tape", move |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut bytes| {
                        simdjson::to_value_fsm(&mut bytes).unwrap();
                    },
                    BatchSize::SmallInput,
                )
            });
            #[cfg(feature = "simdjson-rust")]
            let b = b.with_function("simdjson_cpp", move |b, data| {
                b.iter_batched(
                    || String::from_utf8(data.to_vec()).unwrap(),
                    |bytes| {
                        let _ = build_parsed_json(&bytes, true);
                    },
                    BatchSize::SmallInput,
                )
            });
            #[cfg(feature = "bench-serde")]
            let b = b.with_function("serde_json", move |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut bytes| {
                        let _: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                    },
                    BatchSize::SmallInput,
                )
            });
            c.bench(
                stringify!($name),
                b.throughput(|data| Throughput::Bytes(data.len() as u32)),
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
