#[cfg(feature = "alloc")]
use alloc_counter::{count_alloc, AllocCounterSystem};

#[cfg(feature = "alloc")]
#[global_allocator]
static A: AllocCounterSystem = AllocCounterSystem;

macro_rules! test {
    ($file:ident, $alloc:expr, $realloc:expr) => {
        #[cfg(feature = "alloc")]
        #[test]
        fn $file() {
            use simd_json::{to_tape_with_buffers, Buffers};
            use std::fs::File;
            use std::io::Read;
            let mut v1 = Vec::new();
            let f = String::from(concat!("data/", stringify!($file), ".json"));
            let mut buffers = simd_json::Buffers::default();
            File::open(&f).unwrap().read_to_end(&mut v1).unwrap();
            let _ = to_tape_with_buffers(&mut v1, &mut buffers);
            // we only care about the second run as at this point buffer armortized and we no longer depend
            // on guessing
            File::open(f).unwrap().read_to_end(&mut v1).unwrap();
            let (count, _v) = count_alloc(|| to_tape_with_buffers(&mut v1, &mut buffers));
            dbg!(count);
            assert_eq!(count.0, $alloc);
            assert_eq!(count.1, $realloc);
        }
    };
}

test!(canada, 3, 2);
test!(citm_catalog, 3, 2);
test!(log, 1, 2);
test!(marine_ik, 3, 2);
test!(twitter, 1, 2);
test!(twitterescaped, 1, 2);
test!(numbers, 3, 2);
