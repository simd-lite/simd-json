#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg_attr(not(feature = "no-inline"), inline)]
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub fn add_overflow(value1: u64, value2: u64, result: &mut u64) -> bool {
    return unsafe { _addcarry_u64(0, value1, value2, result) } != 0;
}

//TODO: static?

#[cfg_attr(not(feature = "no-inline"), inline)]
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub fn hamming(input_num: u64) -> u32 {
    return unsafe { _popcnt64(input_num as i64) as u32 };
}

#[cfg_attr(not(feature = "no-inline"), inline)]
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub fn trailingzeroes(input_num: u64) -> u32 {
    return unsafe { _tzcnt_u64(input_num) as u32 };
}
