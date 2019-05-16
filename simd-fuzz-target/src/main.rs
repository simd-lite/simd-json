#[macro_use]
extern crate afl;
extern crate simd_json;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut v = data.to_vec();
        let _ = simd_json::to_owned_value(&mut v);
    });
}
