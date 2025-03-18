#[macro_use]
extern crate criterion;

use core::time::Duration;

#[cfg(feature = "jemallocator")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(feature = "bench-serde")]
use serde_json;

use criterion::{BatchSize, Criterion, Throughput, criterion_group};
use simd_json::Buffers;

use std::fs::File;
use std::io::Read;

fn to_borrowed_value(data: &mut [u8]) {
    simd_json::to_borrowed_value(data).unwrap();
}

fn to_borrowed_value_with_buffers(data: &mut [u8], buffers: &mut Buffers) {
    simd_json::to_borrowed_value_with_buffers(data, buffers).unwrap();
}

fn to_owned_value(data: &mut [u8]) {
    simd_json::to_owned_value(data).unwrap();
}

#[cfg(feature = "bench-serde")]
fn serde_from_slice(data: &[u8]) {
    let _: serde_json::Value = serde_json::from_slice(data).unwrap();
}

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

            let mut group = c.benchmark_group(stringify!($name));
            group.throughput(Throughput::Bytes(vec.len() as u64));
            group
                .warm_up_time(Duration::from_secs(1))
                .measurement_time(Duration::from_secs(20));

            let mut buffers = Buffers::default();

            group.bench_with_input("simd_json::to_borrowed_value", &vec, |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut bytes| to_borrowed_value(&mut bytes),
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(
                "simd_json::to_borrowed_value_with_buffers",
                &vec,
                |b, data| {
                    b.iter_batched(
                        || data.clone(),
                        |mut bytes| to_borrowed_value_with_buffers(&mut bytes, &mut buffers),
                        BatchSize::SmallInput,
                    )
                },
            );

            group.bench_with_input("simd_json::to_owned_value", &vec, |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut bytes| to_owned_value(&mut bytes),
                    BatchSize::SmallInput,
                )
            });

            #[cfg(feature = "bench-serde")]
            group.bench_with_input("serde_json::from_slice", &vec, |b, data| {
                b.iter_batched(
                    || data,
                    |bytes| serde_from_slice(&bytes),
                    BatchSize::SmallInput,
                )
            });
        }
    };
}

bench_file!(apache_builds);
bench_file!(event_stacktrace_10kb);
bench_file!(github_events);
bench_file!(canada);
bench_file!(citm_catalog);
bench_file!(log);
bench_file!(twitter);

criterion_group!(
    benches,
    apache_builds,
    event_stacktrace_10kb,
    github_events,
    canada,
    citm_catalog,
    log,
    twitter
);
criterion_main!(benches);
