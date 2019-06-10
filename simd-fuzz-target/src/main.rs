#[macro_use]
extern crate afl;
extern crate simd_json;

fn main() {
    fuzz!(|data: &[u8]| {
        unsafe {
            let mut data1 = data.clone().to_vec();
            #[allow(mutable_transmutes)]
            let data: &mut [u8] = std::mem::transmute(data);
            if let (Ok(ref jb), Ok(ref jo)) = (
                simd_json::to_borrowed_value(data),
                simd_json::to_owned_value(&mut data1),
            ) {
                let joi: simd_json::OwnedValue = jb.clone().into();
                assert_eq!(jo, joi);
                assert_eq!(jo.to_string(), jb.to_string());
            }
        }
    });
}
