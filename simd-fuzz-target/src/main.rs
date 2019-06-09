#[macro_use]
extern crate afl;
extern crate simd_json;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut v1 = data.to_vec();
        if let Ok(ref jo) = simd_json::to_owned_value(&mut v1) {
            jo.to_string();
        }
    });
}
