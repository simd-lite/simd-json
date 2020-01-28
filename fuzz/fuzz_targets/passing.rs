#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut data1 = data.clone().to_vec();
    if let Ok(ref jo) = simd_json::to_owned_value(&mut data1) {
        jo.to_string();
    }
});
