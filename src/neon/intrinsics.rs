#![allow(unused)]
#![allow(non_camel_case_types)]

//use std::arch::aarch64::*;

use std::mem;
use core;
use crate::neon::intrinsics2;
use crate::simd_lite::aarch64 as simd_lite;

#[allow(unused)]
macro_rules! types {
    ($(
        $(#[$doc:meta])*
        pub struct $name:ident($($fields:tt)*);
    )*) => ($(
        $(#[$doc])*
        #[derive(Copy, Clone, Debug)]
        #[allow(non_camel_case_types)]
        #[repr(simd)]
        #[allow(clippy::missing_inline_in_public_items)]
        pub struct $name($($fields)*);
    )*)
}

#[allow(non_camel_case_types)]
pub type poly64_t = i64;

/// Extract vector from pair of vectors
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
//#[rustc_args_required_const(2)]
pub unsafe fn vextq_u8(a: uint8x16_t, b: uint8x16_t, n: i32) -> uint8x16_t {
    // FIXME: const
    mem::transmute(intrinsics2::vextq_u8(mem::transmute(a), mem::transmute(b), n))
}

/// Extract vector from pair of vectors
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
pub unsafe fn vextq_s8(a: int8x16_t, b: int8x16_t, n: i32) -> int8x16_t {
    // FIXME: const
    mem::transmute(intrinsics2::vextq_s8(mem::transmute(a), mem::transmute(b), n))
}

#[inline]
pub unsafe fn vmull_p64(a: poly64_t, b: poly64_t) -> poly128_t {
    mem::transmute(simd_lite::vmull_p64(mem::transmute(a), mem::transmute(b)))
}

#[inline]
pub fn vpaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    unsafe { mem::transmute(simd_lite::vpaddq_u8(mem::transmute(a), mem::transmute(b))) }
}

#[inline]
pub unsafe fn vshrq_n_u8(a: uint8x16_t, n: i32) -> uint8x16_t {
    // FIXME: const
    mem::transmute(intrinsics2::vshrq_n_u8(mem::transmute(a), n))
}

types! {
    /// ARM-specific 64-bit wide vector of eight packed `i8`.
    pub struct int8x8_t(i8, i8, i8, i8, i8, i8, i8, i8);
    /// ARM-specific 64-bit wide vector of eight packed `u8`.
    pub struct uint8x8_t(u8, u8, u8, u8, u8, u8, u8, u8);
    /// ARM-specific 64-bit wide polynomial vector of eight packed `u8`.
    pub struct poly8x8_t(u8, u8, u8, u8, u8, u8, u8, u8);
    /// ARM-specific 64-bit wide vector of four packed `i16`.
    pub struct int16x4_t(i16, i16, i16, i16);
    /// ARM-specific 64-bit wide vector of four packed `u16`.
    pub struct uint16x4_t(u16, u16, u16, u16);
    /// ARM-specific 64-bit wide vector of four packed `u16`.
    pub struct poly16x4_t(u16, u16, u16, u16);
    /// ARM-specific 64-bit wide vector of two packed `i32`.
    pub struct int32x2_t(i32, i32);
    /// ARM-specific 64-bit wide vector of two packed `u32`.
    pub struct uint32x2_t(u32, u32);
    /// ARM-specific 64-bit wide vector of two packed `f32`.
    pub struct float32x2_t(f32, f32);
    /// ARM-specific 64-bit wide vector of one packed `i64`.
    pub struct int64x1_t(i64);
    /// ARM-specific 64-bit wide vector of one packed `u64`.
    pub struct uint64x1_t(u64);
    /// ARM-specific 128-bit wide vector of sixteen packed `i8`.
    pub struct int8x16_t(
        i8, i8 ,i8, i8, i8, i8 ,i8, i8,
        i8, i8 ,i8, i8, i8, i8 ,i8, i8,
    );
    /// ARM-specific 128-bit wide vector of sixteen packed `u8`.
    pub struct uint8x16_t(
        u8, u8 ,u8, u8, u8, u8 ,u8, u8,
        u8, u8 ,u8, u8, u8, u8 ,u8, u8,
    );
    /// ARM-specific 128-bit wide vector of sixteen packed `u8`.
    pub struct poly8x16_t(
        u8, u8, u8, u8, u8, u8, u8, u8,
        u8, u8, u8, u8, u8, u8, u8, u8
    );
    /// ARM-specific 128-bit wide vector of eight packed `i16`.
    pub struct int16x8_t(i16, i16, i16, i16, i16, i16, i16, i16);
    /// ARM-specific 128-bit wide vector of eight packed `u16`.
    pub struct uint16x8_t(u16, u16, u16, u16, u16, u16, u16, u16);
    /// ARM-specific 128-bit wide vector of eight packed `u16`.
    pub struct poly16x8_t(u16, u16, u16, u16, u16, u16, u16, u16);
    /// ARM-specific 128-bit wide vector of four packed `i32`.
    pub struct int32x4_t(i32, i32, i32, i32);
    /// ARM-specific 128-bit wide vector of four packed `u32`.
    pub struct uint32x4_t(u32, u32, u32, u32);
    /// ARM-specific 128-bit wide vector of four packed `f32`.
    pub struct float32x4_t(f32, f32, f32, f32);
    /// ARM-specific 128-bit wide vector of two packed `i64`.
    pub struct int64x2_t(i64, i64);
    /// ARM-specific 128-bit wide vector of two packed `u64`.
    pub struct uint64x2_t(u64, u64);
    /// ARM-specific 128-bit wide vector of one packed `i128`.
    pub struct poly128_t(i128); // FIXME: check this!
}

impl uint8x16_t {
    #[inline]
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8, i: u8, j: u8, k: u8, l: u8, m: u8, n: u8, o: u8, p: u8) -> uint8x16_t {
        unsafe { std::mem::transmute([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]) }
    }
}

impl int8x16_t {
    #[inline]
    pub fn new(a: i8, b: i8, c: i8, d: i8, e: i8, f: i8, g: i8, h: i8, i: i8, j: i8, k: i8, l: i8, m: i8, n: i8, o: i8, p: i8) -> int8x16_t {
        unsafe { std::mem::transmute([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]) }
    }
}

impl int32x4_t {
    #[inline]
    pub fn new(a: i32, b: i32, c: i32, d: i32) -> int32x4_t {
        unsafe { std::mem::transmute([a, b, c, d]) }
    }
}


#[inline]
pub unsafe fn vld1q_s8(addr: *const i8) -> int8x16_t {
    mem::transmute(simd_lite::vld1q_s8(addr))
}

#[inline]
pub unsafe fn vld1q_u8(addr: *const u8) -> uint8x16_t {
    mem::transmute(simd_lite::vld1q_u8(addr))
}

macro_rules! aarch64_simd_2 {
    ($name: ident, $type: ty, $simd_fn: ident, $intrarm: ident, $intraarch: ident) => {
        aarch64_simd_2!($name, $type, $type, $simd_fn, $intrarm, $intraarch);
    };
    ($name: ident, $type: ty, $res: ty, $simd_fn: ident, $intrarm: ident, $intraarch: ident) => {
        #[inline]
        pub fn $name(a: $type, b: $type) -> $res {
              unsafe { mem::transmute(simd_lite::$name(mem::transmute(a), mem::transmute(b))) }
        }
    }
}

macro_rules! aarch64_simd_ceq {
    ($name: ident, $type: ty, $res: ty) => {
        /// Compare bitwise Equal (vector)
        aarch64_simd_2!($name, $type, $res, simd_eq, cmeq, cmeq);
    };
}

aarch64_simd_ceq!(vceqq_s8, int8x16_t, uint8x16_t);
aarch64_simd_ceq!(vceqq_u8, uint8x16_t, uint8x16_t);

macro_rules! aarch64_simd_cgt {
    ($name:ident, $type:ty, $res:ty) => {
        /// Compare signed Greater than (vector)
        aarch64_simd_2!($name, $type, $res, simd_gt, cmgt, cmgt);
    };
}

aarch64_simd_cgt!(vcgtq_s8, int8x16_t, uint8x16_t);

macro_rules! aarch64_simd_cle {
    ($name:ident, $type:ty, $res:ty) => {
        /// Compare signed Lesser than equals (vector)
        aarch64_simd_2!($name, $type, $res, simd_le, cmge, cmge);
    };
}

aarch64_simd_cle!(vcleq_u8, uint8x16_t, uint8x16_t);

#[inline]
pub unsafe fn vdupq_n_s8(a: i8) -> int8x16_t {
    mem::transmute(simd_lite::vdupq_n_s8(a))
}

#[inline]
pub fn vdupq_n_u8(a: u8) -> uint8x16_t {
    unsafe { mem::transmute(simd_lite::vdupq_n_u8(a)) }
}

#[inline]
pub fn vmovq_n_u8(a: u8) -> uint8x16_t {
    unsafe { mem::transmute(simd_lite::vmovq_n_u8(a)) }
}

#[inline]
pub fn vmovq_n_s8(a: i8) -> int8x16_t {
    unsafe { mem::transmute(simd_lite::vmovq_n_s8(a)) }
}

#[inline]
pub unsafe fn vaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    // FIXME: private?
    mem::transmute(intrinsics2::vaddq_u8(mem::transmute(a), mem::transmute(b)))
}

#[inline]
pub unsafe fn vaddq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    // FIXME: private?
    mem::transmute(intrinsics2::vaddq_s8(mem::transmute(a), mem::transmute(b)))
}

#[inline]
pub unsafe fn vaddq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    // FIXME: private?
    mem::transmute(intrinsics2::vaddq_s32(mem::transmute(a), mem::transmute(b)))
}

#[inline]
pub fn vandq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    unsafe { mem::transmute(simd_lite::vandq_u8(mem::transmute(a), mem::transmute(b))) }
}
#[inline]
pub fn vandq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    unsafe { mem::transmute(simd_lite::vandq_s8(mem::transmute(a), mem::transmute(b))) }
}
#[inline]
pub fn vandq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    unsafe { mem::transmute(simd_lite::vandq_s16(mem::transmute(a), mem::transmute(b))) }
}
#[inline]
pub fn vorrq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    unsafe { mem::transmute(simd_lite::vorrq_u8(mem::transmute(a), mem::transmute(b))) }
}
#[inline]
pub fn vorrq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    unsafe { mem::transmute(simd_lite::vorrq_s8(mem::transmute(a), mem::transmute(b))) }
}
#[inline]
pub fn veorq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    unsafe { mem::transmute(simd_lite::veorq_u8(mem::transmute(a), mem::transmute(b))) }
}
#[inline]
pub fn veorq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    unsafe { mem::transmute(simd_lite::veorq_s8(mem::transmute(a), mem::transmute(b))) }
}

macro_rules! arm_reinterpret {
    ($name: ident, $from: ty, $to: ty) => {
        // Vector reinterpret cast operation
        #[inline]
        pub fn $name(a: $from) -> $to {
            unsafe { mem::transmute(simd_lite::$name(mem::transmute(a))) }
        }
    };
}

arm_reinterpret!(vreinterpret_u64_u32, uint32x2_t, uint64x1_t);
arm_reinterpret!(vreinterpretq_s8_u8, uint8x16_t, int8x16_t);
arm_reinterpret!(vreinterpretq_u16_u8, uint8x16_t, uint16x8_t);
arm_reinterpret!(vreinterpretq_u32_u8, uint8x16_t, uint32x4_t);
arm_reinterpret!(vreinterpretq_u64_u8, uint8x16_t, uint64x2_t);
arm_reinterpret!(vreinterpretq_u8_s8, int8x16_t, uint8x16_t);

macro_rules! arm_vget_lane {
    ($name: ident, $to: ty, $from: ty, $lanes: literal) => {
        #[inline]
        pub unsafe fn $name(v: $from, lane: i32) -> $ to {
            // FIXME! const
            mem::transmute(intrinsics2::$name(mem::transmute(v), lane))
        }
    };
}

arm_vget_lane!(vgetq_lane_u16, u16, uint16x8_t, 7);
arm_vget_lane!(vgetq_lane_u32, u32, uint32x4_t, 3);
arm_vget_lane!(vgetq_lane_u64, u64, uint64x2_t, 1);
arm_vget_lane!(vget_lane_u64, u64, uint64x1_t, 0);

#[inline]
pub unsafe fn vqmovn_u64(a: uint64x2_t) -> uint32x2_t {
    mem::transmute(simd_lite::vqmovn_u64(mem::transmute(a)))
}

#[inline]
pub unsafe fn vqtbl1q_s8(t: int8x16_t, idx: uint8x16_t) -> int8x16_t {
    mem::transmute(core::arch::aarch64::vqtbl1q_s8(mem::transmute(t), mem::transmute(idx)))
}

#[inline]
pub unsafe fn vqtbl1q_u8(t: uint8x16_t, idx: uint8x16_t) -> uint8x16_t {
    mem::transmute(core::arch::aarch64::vqtbl1q_s8(mem::transmute(t), mem::transmute(idx)))
}

#[inline]
pub unsafe fn vqsubq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    mem::transmute(simd_lite::vqsubq_u8(mem::transmute(a), mem::transmute(b)))
}

//#[inline]
//pub unsafe fn vqsubq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
//    // FIXME: we have a problem!
//    mem::transmute(intrinsics2::vqsubq_s8(mem::transmute(a), mem::transmute(b)))
//}

#[inline]
pub unsafe fn vtstq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    mem::transmute(simd_lite::vtstq_u8(mem::transmute(a), mem::transmute(b)))
}

//#[inline]
//pub unsafe fn vtstq_s8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
//    simd_lite::vtstq_s8(a, b)
//}

//#[inline]
//fn test_u8(a: u8, b: u8) -> u8 {
//    if a & b != 0 {
//        0xFF
//    } else {
//        0x00
//    }
//}
//
//#[inline]
//pub unsafe fn vtstq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
//    uint8x16_t(
//        test_u8(a.0, b.0),
//        test_u8(a.1, b.1),
//        test_u8(a.2, b.2),
//        test_u8(a.3, b.3),
//        test_u8(a.4, b.4),
//        test_u8(a.5, b.5),
//        test_u8(a.6, b.6),
//        test_u8(a.7, b.7),
//        test_u8(a.8, b.8),
//        test_u8(a.9, b.9),
//        test_u8(a.10, b.10),
//        test_u8(a.11, b.11),
//        test_u8(a.12, b.12),
//        test_u8(a.13, b.13),
//        test_u8(a.14, b.14),
//        test_u8(a.15, b.15),
//    )
//}
//
#[inline]
fn test_s8(a: i8, b: i8) -> i8 {
    if a & b != 0 {
        -1
    } else {
        0x00
    }
}

#[inline]
pub unsafe fn vtstq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    int8x16_t(
        test_s8(a.0, b.0),
        test_s8(a.1, b.1),
        test_s8(a.2, b.2),
        test_s8(a.3, b.3),
        test_s8(a.4, b.4),
        test_s8(a.5, b.5),
        test_s8(a.6, b.6),
        test_s8(a.7, b.7),
        test_s8(a.8, b.8),
        test_s8(a.9, b.9),
        test_s8(a.10, b.10),
        test_s8(a.11, b.11),
        test_s8(a.12, b.12),
        test_s8(a.13, b.13),
        test_s8(a.14, b.14),
        test_s8(a.15, b.15),
    )
}
