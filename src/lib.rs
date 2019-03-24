#![feature(reverse_bits)]
#![feature(ptr_offset_from)]

mod charutils;
mod numberparse;
mod parsedjson;
mod portability;
mod stage1;
mod stage2;
mod stringparse;

//TODO: Do compile hints like this exist in rust?
/*
macro_rules! likely {
    ($e:expr) => {
        $e
    };
}
*/
#[macro_export]
macro_rules! unlikely {
    ($e:expr) => {
        $e
    };
}

#[macro_export]
macro_rules! static_cast_u32 {
    ($v:expr) => {
        mem::transmute::<_, u32>($v)
    };
}

#[macro_export]
macro_rules! static_cast_i64 {
    ($v:expr) => {
        mem::transmute::<_, i64>($v)
    };
}

#[macro_export]
macro_rules! static_cast_u64 {
    ($v:expr) => {
        mem::transmute::<_, u64>($v)
    };
}

pub use crate::parsedjson::ParsedJson;
use crate::stage1::find_structural_bits;
use crate::stage2::unified_machine;
pub use crate::stage2::MachineError as Error;

pub fn parse(data: &[u8]) -> Result<ParsedJson, Error> {
    let mut pj = ParsedJson::default();
    unsafe {
        find_structural_bits(data, data.len() as u32, &mut pj);
        unified_machine(data, data.len(), &mut pj)?;
    };
    Ok(pj)
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
