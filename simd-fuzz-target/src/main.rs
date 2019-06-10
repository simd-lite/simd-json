#[macro_use]
extern crate afl;
extern crate simd_json;

fn main() {
    fuzz!(|data: &[u8]| {
        unsafe {
            let mut data1 = data.clone().to_vec();
            if let Ok(ref jo) = simd_json::to_owned_value(&mut data1) {
                jo.to_string();
            }
        }
    });
}
