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
            dbg!(&count);
            assert!(count.0 <= $alloc);
            assert!(count.1 <= $realloc);
            assert!(count.2 <= $drop);
        }
    };
}

test!(canada, 5, 0, 4);
test!(citm_catalog, 5, 0, 4);
test!(log, 5, 0, 4);
test!(marine_ik, 5, 1, 4);
test!(twitter, 5, 0, 4);
test!(twitterescaped, 5, 0, 4);
test!(numbers, 5, 0, 4);
