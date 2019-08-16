//use std::arch::

use crate::neon::simd_llvm;

use std::mem;
use core;

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

#[allow(improper_ctypes)]
extern "C" {
    #[link_name = "llvm.aarch64.neon.addp.v16u8"]
    fn vpaddq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    #[link_name = "llvm.aarch64.neon.pmull64"]
    fn vmull_p64_(a: i64, b: i64) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.uqxtn.v2u32"]
    fn vqmovn_u64_(a: uint64x2_t) -> uint32x2_t;
    #[link_name = "llvm.aarch64.neon.uqsub.v16u8"]
    fn vqsubq_u8_(a: uint8x16_t, a: uint8x16_t) -> uint8x16_t;
    #[link_name = "llvm.aarch64.neon.uqsub.v16i8"]
    fn vqsubq_s8_(a: int8x16_t, a: int8x16_t) -> int8x16_t;
}

#[inline]
unsafe fn vaddq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_llvm::simd_add(mem::transmute(a), mem::transmute(b))
}

#[inline]
unsafe fn vaddq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    simd_llvm::simd_add(mem::transmute(a), mem::transmute(b))
}

#[inline]
unsafe fn vaddq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    simd_llvm::simd_add(mem::transmute(a), mem::transmute(b))
}

#[inline]
pub unsafe fn vnegq_u8(a: uint8x16_t) -> uint8x16_t {
    let x: u128 = mem::transmute(a);
    let nx = !x;
    mem::transmute(nx)
}

#[inline]
pub unsafe fn vnegq_s8(a: int8x16_t) -> int8x16_t {
    let x: u128 = mem::transmute(a);
    let nx = !x;
    mem::transmute(nx)
}


#[inline]
fn rotate_(a: u128, b: u128, n: u128) -> u128 {
    let az = a >> (n * 8);
    let bz = b << (128 - (n * 8));
    az | bz
}

#[inline]
pub unsafe fn vextq_u8(a: uint8x16_t, b: uint8x16_t, n: u8) -> uint8x16_t {
    mem::transmute(rotate_(mem::transmute(a), mem::transmute(b), n as u128))
}

#[inline]
pub unsafe fn vextq_s8(a: int8x16_t, b: int8x16_t, n: u8) -> int8x16_t {
    mem::transmute(rotate_(mem::transmute(a), mem::transmute(b), n as u128))
}

#[inline]
pub unsafe fn vmull_p64(a: poly64_t, b: poly64_t) -> poly128_t {
    mem::transmute(vmull_p64_(mem::transmute(a), mem::transmute(b)))
}

#[inline]
pub fn vpaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    unsafe { vpaddq_u8_(a, b) }
}

#[inline]
pub unsafe fn vshrq_n_u8(a: uint8x16_t, n: u8) -> uint8x16_t {
    uint8x16_t(
        a.0 >> n,
        a.1 >> n,
        a.2 >> n,
        a.3 >> n,
        a.4 >> n,
        a.5 >> n,
        a.6 >> n,
        a.7 >> n,
        a.8 >> n,
        a.9 >> n,
        a.10 >> n,
        a.11 >> n,
        a.12 >> n,
        a.13 >> n,
        a.14 >> n,
        a.15 >> n,
    )
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
        uint8x16_t(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p)
    }
}

impl int8x16_t {
    #[inline]
    pub fn new(a: i8, b: i8, c: i8, d: i8, e: i8, f: i8, g: i8, h: i8, i: i8, j: i8, k: i8, l: i8, m: i8, n: i8, o: i8, p: i8) -> int8x16_t {
        int8x16_t(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p)
    }
}

impl int32x4_t {
    #[inline]
    pub fn new(a: i32, b: i32, c: i32, d: i32) -> int32x4_t {
        int32x4_t(a, b, c, d)
    }
}

//#[inline]
//pub fn add_overflow(a: u64, b: u64, out: &mut u64) -> bool {
//    let (carry, did_carry) = a.overflowing_add(b);
//    *out = carry;
//    did_carry
//}

#[inline]
pub unsafe fn vld1q_s8(addr: *const i8) -> int8x16_t {
    *(addr as *const int8x16_t)
}

#[inline]
pub unsafe fn vld1q_u8(addr: *const u8) -> uint8x16_t {
    *(addr as *const uint8x16_t)
}

#[inline]
pub unsafe fn vst1q_u8(addr: *mut u8, val: uint8x16_t) {
    std::ptr::write(addr as *mut uint8x16_t, val);
}

macro_rules! aarch64_simd_2 {
    ($name: ident, $type: ty, $simd_fn: ident, $intrarm: ident, $intraarch: ident) => {
        aarch64_simd_2!($name, $type, $type, $simd_fn, $intrarm, $intraarch);
    };
    ($name: ident, $type: ty, $res: ty, $simd_fn: ident, $intrarm: ident, $intraarch: ident) => {
        #[inline]
        pub fn $name(a: $type, b: $type) -> $res {
            unsafe { simd_llvm::$simd_fn(a, b) }
        }
    }
}

macro_rules! aarch64_simd_ceq {
    ($name: ident, $type: ty, $res: ty) => {
        /// Compare bitwise Equal (vector)
        aarch64_simd_2!($name, $type, $res, simd_eq, cmeq, cmeq);
    };
}

aarch64_simd_ceq!(vceq_s8, int8x8_t, uint8x8_t);
aarch64_simd_ceq!(vceqq_s8, int8x16_t, uint8x16_t);
aarch64_simd_ceq!(vceq_s16, int16x4_t, uint16x4_t);
aarch64_simd_ceq!(vceqq_s16, int16x8_t, uint16x8_t);
aarch64_simd_ceq!(vceq_s32, int32x2_t, uint32x2_t);
aarch64_simd_ceq!(vceqq_s32, int32x4_t, uint32x4_t);
aarch64_simd_ceq!(vceq_u8, uint8x8_t, uint8x8_t);
aarch64_simd_ceq!(vceqq_u8, uint8x16_t, uint8x16_t);
aarch64_simd_ceq!(vceq_u16, uint16x4_t, uint16x4_t);
aarch64_simd_ceq!(vceqq_u16, uint16x8_t, uint16x8_t);
aarch64_simd_ceq!(vceq_u32, uint32x2_t, uint32x2_t);
aarch64_simd_ceq!(vceqq_u32, uint32x4_t, uint32x4_t);
aarch64_simd_2!(vceq_f32, float32x2_t, uint32x2_t, simd_eq, fcmeq, fcmeq);
aarch64_simd_2!(vceqq_f32, float32x4_t, uint32x4_t, simd_eq, fcmeq, fcmeq);
aarch64_simd_ceq!(vceq_p8, poly8x8_t, poly8x8_t);
aarch64_simd_ceq!(vceqq_p8, poly8x16_t, poly8x16_t);

macro_rules! aarch64_simd_cgt {
    ($name:ident, $type:ty, $res:ty) => {
        /// Compare signed Greater than (vector)
        aarch64_simd_2!($name, $type, $res, simd_gt, cmgt, cmgt);
    };
}

//macro_rules! aarch64_simd_cgtu {
//    ($name: ident, $type: ty) => {
//        /// Compare Greater than (vector)
//        aarch64_simd_2!($name, $type, simd_gt, cmhi);
//    };
//}

aarch64_simd_cgt!(vcgt_s8, int8x8_t, uint8x8_t);
aarch64_simd_cgt!(vcgtq_s8, int8x16_t, uint8x16_t);
aarch64_simd_cgt!(vcgt_s16, int16x4_t, uint16x4_t);
aarch64_simd_cgt!(vcgtq_s16, int16x8_t, uint16x8_t);
aarch64_simd_cgt!(vcgt_s32, int32x2_t, uint32x2_t);
aarch64_simd_cgt!(vcgtq_s32, int32x4_t, uint32x4_t);

//aarch64_simd_cgtu!(vcgtq_u8, uint8x16_t);
//aarch64_simd_cgt!(vcgt_s64, int64x1_t);
//aarch64_simd_cgt!(vcgtq_s64, int64x2_t);
//aarch64_simd_cgtu!(vcgt_u64, uint64x1_t);
//aarch64_simd_cgtu!(vcgtq_u64, uint64x2_t);

macro_rules! aarch64_simd_clt {
    ($name:ident, $type:ty, $res:ty) => {
        /// Compare signed Lesser than (vector)
        aarch64_simd_2!($name, $type, $res, simd_lt, cmgt, cmgt);
    };
}

//macro_rules! aarch64_simd_cltu {
//( $ name: ident, $ type: ty) => {
///// Compare Lesser than (vector)
//aarch64_simd_2 ! ( $ name, $ type, simd_lt, cmhi);
//};
//}

aarch64_simd_clt!(vclt_s8, int8x8_t, uint8x8_t);
aarch64_simd_clt!(vcltq_s8, int8x16_t, uint8x16_t);
aarch64_simd_clt!(vclt_s16, int16x4_t, uint16x4_t);
aarch64_simd_clt!(vcltq_s16, int16x8_t, uint16x8_t);
aarch64_simd_clt!(vclt_s32, int32x2_t, uint32x2_t);
aarch64_simd_clt!(vcltq_s32, int32x4_t, uint32x4_t);

//arm_simd_cltu!(vclt_u8, uint8x8_t);
//arm_simd_cltu!(vcltq_u8, uint8x16_t);
//arm_simd_cltu!(vclt_u16, uint16x4_t);
//arm_simd_cltu!(vcltq_u16, uint16x8_t);
//arm_simd_cltu!(vclt_u32, uint32x2_t);
//arm_simd_cltu!(vcltq_u32, uint32x4_t);

macro_rules! aarch64_simd_cge {
    ($name:ident, $type:ty, $res:ty) => {
        /// Compare signed Greater than equals (vector)
        aarch64_simd_2!($name, $type, $res, simd_ge, cmge, cmge);
    };
}

//macro_rules! aarch64_simd_cgeu {
//( $ name: ident, $ type: ty) => {
///// Compare Greater than (vector)
//aarch64_simd_2 ! ( $ name, $ type, simd_ge, cmhs);
//};
//}

aarch64_simd_cge!(vcge_s8, int8x8_t, uint8x8_t);
aarch64_simd_cge!(vcgeq_s8, int8x16_t, uint8x16_t);
aarch64_simd_cge!(vcge_s16, int16x4_t, uint16x4_t);
aarch64_simd_cge!(vcgeq_s16, int16x8_t, uint16x8_t);
aarch64_simd_cge!(vcge_s32, int32x2_t, uint32x2_t);
aarch64_simd_cge!(vcgeq_s32, int32x4_t, uint32x4_t);
//arm_simd_cgeu!(vcge_u8, uint8x8_t);
//arm_simd_cgeu!(vcgeq_u8, uint8x16_t);
//arm_simd_cgeu!(vcge_u16, uint16x4_t);
//arm_simd_cgeu!(vcgeq_u16, uint16x8_t);
//arm_simd_cgeu!(vcge_u32, uint32x2_t);
//arm_simd_cgeu!(vcgeq_u32, uint32x4_t);

macro_rules! aarch64_simd_cle {
    ($name:ident, $type:ty, $res:ty) => {
        /// Compare signed Lesser than equals (vector)
        aarch64_simd_2!($name, $type, $res, simd_le, cmge, cmge);
    };
}

//macro_rules! aarch64_simd_cleu {
//( $ name: ident, $ type: ty) => {
///// Compare Lesser than (vector)
//aarch64_simd_2 ! ( $ name, $ type, simd_le, cmhs);
//};
//}

aarch64_simd_cle!(vcle_s8, int8x8_t, uint8x8_t);
aarch64_simd_cle!(vcleq_s8, int8x16_t, uint8x16_t);
aarch64_simd_cle!(vcle_s16, int16x4_t, uint16x4_t);
aarch64_simd_cle!(vcleq_s16, int16x8_t, uint16x8_t);
aarch64_simd_cle!(vcle_s32, int32x2_t, uint32x2_t);
aarch64_simd_cle!(vcleq_s32, int32x4_t, uint32x4_t);
//arm_simd_cleu!(vcle_u8, uint8x8_t);
aarch64_simd_cle!(vcleq_u8, uint8x16_t, uint8x16_t);
//arm_simd_cleu!(vcle_u16, uint16x4_t);
//arm_simd_cleu!(vcleq_u16, uint16x8_t);
//arm_simd_cleu!(vcle_u32, uint32x2_t);
//arm_simd_cleu!(vcleq_u32, uint32x4_t);

#[inline]
pub fn vdupq_n_s8(a: i8) -> int8x16_t {
    int8x16_t(a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a)
}

#[inline]
pub fn zeroi8x16() -> int8x16_t {
    int8x16_t(0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00)
}

#[inline]
pub fn vdupq_n_u8(a: u8) -> uint8x16_t {
    uint8x16_t(a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a)
}

#[inline]
pub fn vmovq_n_u8(a: u8) -> uint8x16_t {
    uint8x16_t(a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a)
}

#[inline]
pub fn vmovq_n_s8(a: i8) -> int8x16_t {
    int8x16_t(a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a)
}

#[inline]
pub fn zerou8x16() -> uint8x16_t {
    uint8x16_t(0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00)
}

#[inline]
pub unsafe fn vaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    mem::transmute(vaddq_u8_(mem::transmute(a), mem::transmute(b)))
}

#[inline]
pub unsafe fn vaddq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    mem::transmute(vaddq_s8_(mem::transmute(a), mem::transmute(b)))
}

#[inline]
pub unsafe fn vaddq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    mem::transmute(vaddq_s32_(mem::transmute(a), mem::transmute(b)))
}

#[inline]
pub fn vandq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t { unsafe { simd_llvm::simd_and(a, b) } }
#[inline]
pub fn vandq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t { unsafe { simd_llvm::simd_and(a, b) } }
#[inline]
pub fn vorrq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t { unsafe { simd_llvm::simd_or(a, b) } }
#[inline]
pub fn vandq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t { unsafe { simd_llvm::simd_and(a, b) } }
#[inline]
pub fn vorrq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t { unsafe { simd_llvm::simd_or(a, b) } }

macro_rules! arm_reinterpret {
    ($name: ident, $from: ty, $to: ty) => {
        // Vector reinterpret cast operation
        #[inline]
        pub fn $name(a: $from) -> $to {
            unsafe { mem::transmute(a) }
        }
    };
}

arm_reinterpret!(vreinterpret_u64_u32, uint32x2_t, uint64x1_t);
arm_reinterpret!(vreinterpretq_u64_u32, uint32x4_t, uint64x2_t);
arm_reinterpret!(vreinterpretq_s8_u8, uint8x16_t, int8x16_t);
arm_reinterpret!(vreinterpretq_u16_u8, uint8x16_t, uint16x8_t);
arm_reinterpret!(vreinterpretq_u32_u8, uint8x16_t, uint32x4_t);
arm_reinterpret!(vreinterpretq_u64_u8, uint8x16_t, uint64x2_t);
arm_reinterpret!(vreinterpretq_u64_s8, int8x16_t, uint64x2_t);
arm_reinterpret!(vreinterpretq_u8_s8, int8x16_t, uint8x16_t);

arm_reinterpret!(vreinterpretq_s16_s8, int8x16_t, int16x8_t);
arm_reinterpret!(vreinterpretq_s32_s8, int8x16_t, int32x4_t);
arm_reinterpret!(vreinterpretq_s64_s8, int8x16_t, int64x2_t);

macro_rules! arm_vget_lane {
    ($name: ident, $to: ty, $from: ty, $lanes: literal) => {
        #[inline]
        pub unsafe fn $name(v: $from, lane: u32) -> $ to {
            simd_llvm::simd_extract(v, lane)
        }
    };
}

arm_vget_lane!(vgetq_lane_u16, u16, uint16x8_t, 7);
arm_vget_lane!(vgetq_lane_u32, u32, uint32x4_t, 3);
arm_vget_lane!(vgetq_lane_u64, u64, uint64x2_t, 1);
arm_vget_lane!(vget_lane_u64, u64, uint64x1_t, 0);

arm_vget_lane!(vgetq_lane_s16, i16, int16x8_t, 7);
arm_vget_lane!(vgetq_lane_s32, i32, int32x4_t, 3);
arm_vget_lane!(vgetq_lane_s64, i64, int64x2_t, 1);
arm_vget_lane!(vget_lane_s64, i64, int64x1_t, 0);

#[inline]
pub unsafe fn vqmovn_u64(a: uint64x2_t) -> uint32x2_t {
    vqmovn_u64_(a)
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
    vqsubq_u8_(a, b)
}

#[inline]
pub unsafe fn vqsubq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vqsubq_s8_(a, b)
}

#[inline]
fn test_u8(a: u8, b: u8) -> u8 {
    if a & b != 0 {
        0xFF
    } else {
        0x00
    }
}

#[inline]
pub unsafe fn vtstq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    uint8x16_t(
        test_u8(a.0, b.0),
        test_u8(a.1, b.1),
        test_u8(a.2, b.2),
        test_u8(a.3, b.3),
        test_u8(a.4, b.4),
        test_u8(a.5, b.5),
        test_u8(a.6, b.6),
        test_u8(a.7, b.7),
        test_u8(a.8, b.8),
        test_u8(a.9, b.9),
        test_u8(a.10, b.10),
        test_u8(a.11, b.11),
        test_u8(a.12, b.12),
        test_u8(a.13, b.13),
        test_u8(a.14, b.14),
        test_u8(a.15, b.15),
    )
}

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

#[inline]
pub unsafe fn vst1q_u32(addr: *mut u8, val: uint32x4_t) {
    std::ptr::write(addr as *mut uint32x4_t, val)
}
