#[macro_use]
extern crate criterion;

#[cfg(feature = "jemallocator")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(feature = "bench-serde")]
use serde_json;

use criterion::{criterion_group, Criterion};

#[cfg(feature = "bench-serde")]
fn serde_from_slice(data: &[u8]) {
    let _: serde_json::Value = serde_json::from_slice(data).unwrap();
}

#[allow(unused_macros)]
macro_rules! bench_file {
    ($name:ident) => {
        fn $name(c: &mut Criterion) {
            use core::time::Duration;
            use criterion::{BatchSize, Throughput};
            use simd_json::Buffers;
            use std::fs::File;
            use std::io::Read;
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

            // group.bench_with_input("simd_json::to_tape", &vec, |b, data| {
            //     b.iter_batched(
            //         || data.clone(),
            //         |mut bytes| {
            //             simd_json::to_tape(&mut bytes).unwrap();
            //         },
            //         BatchSize::SmallInput,
            //     )
            // });
            group.bench_with_input("simd_json::to_tape_with_buffers", &vec, |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut bytes| {
                        simd_json::to_tape_with_buffers(&mut bytes, &mut buffers).unwrap();
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    };
}

#[allow(unused_macros)]
macro_rules! bench_file_skip {
    ($name:ident) => {
        fn $name(_c: &mut Criterion) {}
    };
}

#[cfg(feature = "bench-apache_builds")]
bench_file!(apache_builds);
#[cfg(not(feature = "bench-apache_builds"))]
bench_file_skip!(apache_builds);

#[cfg(feature = "bench-event_stacktrace_10kb")]
bench_file!(event_stacktrace_10kb);
#[cfg(not(feature = "bench-event_stacktrace_10kb"))]
bench_file_skip!(event_stacktrace_10kb);

#[cfg(feature = "bench-github_events")]
bench_file!(github_events);
#[cfg(not(feature = "bench-github_events"))]
bench_file_skip!(github_events);

#[cfg(feature = "bench-canada")]
bench_file!(canada);
#[cfg(not(feature = "bench-canada"))]
bench_file_skip!(canada);

#[cfg(feature = "bench-citm_catalog")]
bench_file!(citm_catalog);
#[cfg(not(feature = "bench-citm_catalog"))]
bench_file_skip!(citm_catalog);

#[cfg(feature = "bench-log")]
bench_file!(log);
#[cfg(not(feature = "bench-log"))]
bench_file_skip!(log);

#[cfg(feature = "bench-twitter")]
bench_file!(twitter);
#[cfg(not(feature = "bench-twitter"))]
bench_file_skip!(twitter);

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
