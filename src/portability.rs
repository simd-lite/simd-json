#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[inline]
pub unsafe fn add_overflow(value1: u64, value2: u64, result: &mut u64) -> bool {
    return _addcarry_u64(0, value1, value2, result) != 0;
}

//TODO: static?

#[inline]
pub unsafe fn hamming(input_num: u64) -> u32 {
    #[cfg(target_arch = "x86_64")]
    return _popcnt64(input_num as i64) as u32;
    #[cfg(target_arch = "x86")]
    return __popcnt(input_num as u32) + __popcnt((input_num >> 32) as u32) as u32;
}

#[inline]
pub unsafe fn trailingzeroes(input_num: u64) -> u32 {
    return _tzcnt_u64(input_num) as u32;
}
