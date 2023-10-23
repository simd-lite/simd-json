#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut data1 = data.to_vec();
    let res = simd_json::to_borrowed_value(&mut data1);
    if let Ok(ref jo) = res {
        jo.to_string();
    }
});
