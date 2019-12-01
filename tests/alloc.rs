#[cfg(feature = "alloc")]
use alloc_counter::{count_alloc, AllocCounterSystem};

#[cfg(feature = "alloc")]
#[global_allocator]
static A: AllocCounterSystem = AllocCounterSystem;

macro_rules! test {
    ($file:ident, $alloc:expr, $realloc:expr, $drop:expr) => {
        #[cfg(feature = "alloc")]
        #[test]
        fn $file() {
            use simd_json::to_tape;
            use std::fs::File;
            use std::io::Read;
            let mut v1 = Vec::new();
            let f = String::from(concat!("data/", stringify!($file), ".json"));
            File::open(f).unwrap().read_to_end(&mut v1).unwrap();
            let (count, _v) = count_alloc(|| to_tape(&mut v1));
            assert_eq!(count, ($alloc, $realloc, $drop));
        }
    };
}

test!(canada, 4, 0, 3);
test!(citm_catalog, 4, 0, 3);
test!(log, 4, 0, 3);
test!(marine_ik, 4, 1, 3);
test!(twitter, 4, 0, 3);
test!(twitterescaped, 4, 0, 3);

// Allocations for this are different on archetecture
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(target_feature = "avx2")
))]
test!(numbers, 4, 0, 3);
#[cfg(target_feature = "avx2")]
test!(numbers, 4, 0, 3);
