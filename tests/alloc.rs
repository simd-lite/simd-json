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
            use simd_json::{fill_tape, Buffers, Tape};
            use std::fs::File;
            use std::io::Read;
            let mut v1 = Vec::new();
            let f = String::from(concat!("data/", stringify!($file), ".json"));
            let mut buffers = Buffers::default();
            let mut tape = Tape::null();
            File::open(&f).unwrap().read_to_end(&mut v1).unwrap();
            let _ = fill_tape(&mut v1, &mut buffers, &mut tape);
            // we only care about the second run as at this point buffer armortized and we no longer depend
            // on guessing
            let mut v2 = Vec::new();
            File::open(f).unwrap().read_to_end(&mut v2).unwrap();
            let (count, _v) = count_alloc(|| fill_tape(&mut v2, &mut buffers, &mut tape));
            dbg!(count);
            assert_eq!(count.0, $alloc);
            assert_eq!(count.1, $realloc);
        }
    };
}

// We are testing the "best case", using `fill_tape` and running it twice to ensure all allocations
// are armortizedm this way we should see no additional allocations

test!(canada, 0, 0);
test!(citm_catalog, 0, 0);
test!(log, 0, 0);
test!(marine_ik, 0, 0);
test!(twitter, 0, 0);
test!(twitterescaped, 0, 0);
test!(numbers, 0, 0);
