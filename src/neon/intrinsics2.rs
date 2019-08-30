//! ARMv8 ASIMD intrinsics

#![allow(non_camel_case_types)]
#![allow(unused)]

use std::hint::unreachable_unchecked;
use std::mem::transmute;
use std::mem::zeroed;
use std::ptr;

impl uint8x16_t {
    pub fn new(
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        g: u8,
        h: u8,
        i: u8,
        j: u8,
        k: u8,
        l: u8,
        m: u8,
        n: u8,
        o: u8,
        p: u8,
    ) -> uint8x16_t {
        uint8x16_t(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p)
    }
}

impl int8x16_t {
    pub fn new(
        a: i8,
        b: i8,
        c: i8,
        d: i8,
        e: i8,
        f: i8,
        g: i8,
        h: i8,
        i: i8,
        j: i8,
        k: i8,
        l: i8,
        m: i8,
        n: i8,
        o: i8,
        p: i8,
    ) -> int8x16_t {
        int8x16_t(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p)
    }
}

impl int32x4_t {
    pub fn new(a: i32, b: i32, c: i32, d: i32) -> int32x4_t {
        int32x4_t(a, b, c, d)
    }
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
    if a & b == 0 {
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
fn test_u64(a: u64, b: u64) -> u64 {
    if a & b == 0 {
        0xFFFFFFFFFFFFFFFF
    } else {
        0x00u64
    }
}

#[inline]
pub unsafe fn vtstq_s64(a: u128, b: u128) -> u128 {
    (test_u64(a as u64, b as u64) as u128) | (test_u64((a >> 64) as u64, (b >> 64) as u64) as u128) << 64
}

//pub fn vtstq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
//    unsafe {
//        let v : u128 = std::mem::transmute(vceqq_u8(a, b));
//        let w = !v;
//        std::mem::transmute(w)
//    }
//}
//
////pub fn vmovq_n_u8(a: u8) -> uint8x16_t {
////    uint8x16_t(a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a)
////}
//
pub fn vmovq_n_s8(a: i8) -> int8x16_t {
    int8x16_t(a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a)
}

#[allow(unused)]
macro_rules! constify_imm8 {
    ($imm8:expr, $expand:ident) => {
        #[allow(overflowing_literals)]
        match ($imm8) & 0b1111_1111 {
            0 => $expand!(0),
            1 => $expand!(1),
            2 => $expand!(2),
            3 => $expand!(3),
            4 => $expand!(4),
            5 => $expand!(5),
            6 => $expand!(6),
            7 => $expand!(7),
            8 => $expand!(8),
            9 => $expand!(9),
            10 => $expand!(10),
            11 => $expand!(11),
            12 => $expand!(12),
            13 => $expand!(13),
            14 => $expand!(14),
            15 => $expand!(15),
            16 => $expand!(16),
            17 => $expand!(17),
            18 => $expand!(18),
            19 => $expand!(19),
            20 => $expand!(20),
            21 => $expand!(21),
            22 => $expand!(22),
            23 => $expand!(23),
            24 => $expand!(24),
            25 => $expand!(25),
            26 => $expand!(26),
            27 => $expand!(27),
            28 => $expand!(28),
            29 => $expand!(29),
            30 => $expand!(30),
            31 => $expand!(31),
            32 => $expand!(32),
            33 => $expand!(33),
            34 => $expand!(34),
            35 => $expand!(35),
            36 => $expand!(36),
            37 => $expand!(37),
            38 => $expand!(38),
            39 => $expand!(39),
            40 => $expand!(40),
            41 => $expand!(41),
            42 => $expand!(42),
            43 => $expand!(43),
            44 => $expand!(44),
            45 => $expand!(45),
            46 => $expand!(46),
            47 => $expand!(47),
            48 => $expand!(48),
            49 => $expand!(49),
            50 => $expand!(50),
            51 => $expand!(51),
            52 => $expand!(52),
            53 => $expand!(53),
            54 => $expand!(54),
            55 => $expand!(55),
            56 => $expand!(56),
            57 => $expand!(57),
            58 => $expand!(58),
            59 => $expand!(59),
            60 => $expand!(60),
            61 => $expand!(61),
            62 => $expand!(62),
            63 => $expand!(63),
            64 => $expand!(64),
            65 => $expand!(65),
            66 => $expand!(66),
            67 => $expand!(67),
            68 => $expand!(68),
            69 => $expand!(69),
            70 => $expand!(70),
            71 => $expand!(71),
            72 => $expand!(72),
            73 => $expand!(73),
            74 => $expand!(74),
            75 => $expand!(75),
            76 => $expand!(76),
            77 => $expand!(77),
            78 => $expand!(78),
            79 => $expand!(79),
            80 => $expand!(80),
            81 => $expand!(81),
            82 => $expand!(82),
            83 => $expand!(83),
            84 => $expand!(84),
            85 => $expand!(85),
            86 => $expand!(86),
            87 => $expand!(87),
            88 => $expand!(88),
            89 => $expand!(89),
            90 => $expand!(90),
            91 => $expand!(91),
            92 => $expand!(92),
            93 => $expand!(93),
            94 => $expand!(94),
            95 => $expand!(95),
            96 => $expand!(96),
            97 => $expand!(97),
            98 => $expand!(98),
            99 => $expand!(99),
            100 => $expand!(100),
            101 => $expand!(101),
            102 => $expand!(102),
            103 => $expand!(103),
            104 => $expand!(104),
            105 => $expand!(105),
            106 => $expand!(106),
            107 => $expand!(107),
            108 => $expand!(108),
            109 => $expand!(109),
            110 => $expand!(110),
            111 => $expand!(111),
            112 => $expand!(112),
            113 => $expand!(113),
            114 => $expand!(114),
            115 => $expand!(115),
            116 => $expand!(116),
            117 => $expand!(117),
            118 => $expand!(118),
            119 => $expand!(119),
            120 => $expand!(120),
            121 => $expand!(121),
            122 => $expand!(122),
            123 => $expand!(123),
            124 => $expand!(124),
            125 => $expand!(125),
            126 => $expand!(126),
            127 => $expand!(127),
            128 => $expand!(128),
            129 => $expand!(129),
            130 => $expand!(130),
            131 => $expand!(131),
            132 => $expand!(132),
            133 => $expand!(133),
            134 => $expand!(134),
            135 => $expand!(135),
            136 => $expand!(136),
            137 => $expand!(137),
            138 => $expand!(138),
            139 => $expand!(139),
            140 => $expand!(140),
            141 => $expand!(141),
            142 => $expand!(142),
            143 => $expand!(143),
            144 => $expand!(144),
            145 => $expand!(145),
            146 => $expand!(146),
            147 => $expand!(147),
            148 => $expand!(148),
            149 => $expand!(149),
            150 => $expand!(150),
            151 => $expand!(151),
            152 => $expand!(152),
            153 => $expand!(153),
            154 => $expand!(154),
            155 => $expand!(155),
            156 => $expand!(156),
            157 => $expand!(157),
            158 => $expand!(158),
            159 => $expand!(159),
            160 => $expand!(160),
            161 => $expand!(161),
            162 => $expand!(162),
            163 => $expand!(163),
            164 => $expand!(164),
            165 => $expand!(165),
            166 => $expand!(166),
            167 => $expand!(167),
            168 => $expand!(168),
            169 => $expand!(169),
            170 => $expand!(170),
            171 => $expand!(171),
            172 => $expand!(172),
            173 => $expand!(173),
            174 => $expand!(174),
            175 => $expand!(175),
            176 => $expand!(176),
            177 => $expand!(177),
            178 => $expand!(178),
            179 => $expand!(179),
            180 => $expand!(180),
            181 => $expand!(181),
            182 => $expand!(182),
            183 => $expand!(183),
            184 => $expand!(184),
            185 => $expand!(185),
            186 => $expand!(186),
            187 => $expand!(187),
            188 => $expand!(188),
            189 => $expand!(189),
            190 => $expand!(190),
            191 => $expand!(191),
            192 => $expand!(192),
            193 => $expand!(193),
            194 => $expand!(194),
            195 => $expand!(195),
            196 => $expand!(196),
            197 => $expand!(197),
            198 => $expand!(198),
            199 => $expand!(199),
            200 => $expand!(200),
            201 => $expand!(201),
            202 => $expand!(202),
            203 => $expand!(203),
            204 => $expand!(204),
            205 => $expand!(205),
            206 => $expand!(206),
            207 => $expand!(207),
            208 => $expand!(208),
            209 => $expand!(209),
            210 => $expand!(210),
            211 => $expand!(211),
            212 => $expand!(212),
            213 => $expand!(213),
            214 => $expand!(214),
            215 => $expand!(215),
            216 => $expand!(216),
            217 => $expand!(217),
            218 => $expand!(218),
            219 => $expand!(219),
            220 => $expand!(220),
            221 => $expand!(221),
            222 => $expand!(222),
            223 => $expand!(223),
            224 => $expand!(224),
            225 => $expand!(225),
            226 => $expand!(226),
            227 => $expand!(227),
            228 => $expand!(228),
            229 => $expand!(229),
            230 => $expand!(230),
            231 => $expand!(231),
            232 => $expand!(232),
            233 => $expand!(233),
            234 => $expand!(234),
            235 => $expand!(235),
            236 => $expand!(236),
            237 => $expand!(237),
            238 => $expand!(238),
            239 => $expand!(239),
            240 => $expand!(240),
            241 => $expand!(241),
            242 => $expand!(242),
            243 => $expand!(243),
            244 => $expand!(244),
            245 => $expand!(245),
            246 => $expand!(246),
            247 => $expand!(247),
            248 => $expand!(248),
            249 => $expand!(249),
            250 => $expand!(250),
            251 => $expand!(251),
            252 => $expand!(252),
            253 => $expand!(253),
            254 => $expand!(254),
            _ => $expand!(255),
        }
    };
}

//immediate value: 0:31
#[allow(unused)]
macro_rules! constify_imm5 {
    ($imm8:expr, $expand:ident) => {
        #[allow(overflowing_literals)]
        match ($imm8) & 0b1_1111 {
            0 => $expand!(0),
            1 => $expand!(1),
            2 => $expand!(2),
            3 => $expand!(3),
            4 => $expand!(4),
            5 => $expand!(5),
            6 => $expand!(6),
            7 => $expand!(7),
            8 => $expand!(8),
            9 => $expand!(9),
            10 => $expand!(10),
            11 => $expand!(11),
            12 => $expand!(12),
            13 => $expand!(13),
            14 => $expand!(14),
            15 => $expand!(15),
            16 => $expand!(16),
            17 => $expand!(17),
            18 => $expand!(18),
            19 => $expand!(19),
            20 => $expand!(20),
            21 => $expand!(21),
            22 => $expand!(22),
            23 => $expand!(23),
            24 => $expand!(24),
            25 => $expand!(25),
            26 => $expand!(26),
            27 => $expand!(27),
            28 => $expand!(28),
            29 => $expand!(29),
            30 => $expand!(30),
            _ => $expand!(31),
        }
    };
}

//immediate value: -16:15
#[allow(unused)]
macro_rules! constify_imm5 {
    ($imm8:expr, $expand:ident) => {
        #[allow(overflowing_literals)]
        match ($imm8) & 0b1_1111 {
            0 => $expand!(0),
            1 => $expand!(1),
            2 => $expand!(2),
            3 => $expand!(3),
            4 => $expand!(4),
            5 => $expand!(5),
            6 => $expand!(6),
            7 => $expand!(7),
            8 => $expand!(8),
            9 => $expand!(9),
            10 => $expand!(10),
            11 => $expand!(11),
            12 => $expand!(12),
            13 => $expand!(13),
            14 => $expand!(14),
            15 => $expand!(15),
            16 => $expand!(16),
            17 => $expand!(17),
            18 => $expand!(18),
            19 => $expand!(19),
            20 => $expand!(20),
            21 => $expand!(21),
            22 => $expand!(22),
            23 => $expand!(23),
            24 => $expand!(24),
            25 => $expand!(25),
            26 => $expand!(26),
            27 => $expand!(27),
            28 => $expand!(28),
            29 => $expand!(29),
            30 => $expand!(30),
            _ => $expand!(31),
        }
    };
}

//immediate value: 0:16
#[allow(unused)]
macro_rules! constify_imm4 {
    ($imm8:expr, $expand:ident) => {
        #[allow(overflowing_literals)]
        match ($imm8) & 0b1111 {
            0 => $expand!(0),
            1 => $expand!(1),
            2 => $expand!(2),
            3 => $expand!(3),
            4 => $expand!(4),
            5 => $expand!(5),
            6 => $expand!(6),
            7 => $expand!(7),
            8 => $expand!(8),
            9 => $expand!(9),
            10 => $expand!(10),
            11 => $expand!(11),
            12 => $expand!(12),
            13 => $expand!(13),
            14 => $expand!(14),
            _ => $expand!(15),
        }
    };
}

//immediate value: 0:7
#[allow(unused)]
macro_rules! constify_imm3 {
    ($imm8:expr, $expand:ident) => {
        #[allow(overflowing_literals)]
        match ($imm8) & 0b111 {
            0 => $expand!(0),
            1 => $expand!(1),
            2 => $expand!(2),
            3 => $expand!(3),
            4 => $expand!(4),
            5 => $expand!(5),
            6 => $expand!(6),
            _ => $expand!(7),
        }
    };
}

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

extern "platform-intrinsic" {
    pub fn simd_eq<T, U>(x: T, y: T) -> U;
    pub fn simd_ne<T, U>(x: T, y: T) -> U;
    pub fn simd_lt<T, U>(x: T, y: T) -> U;
    pub fn simd_le<T, U>(x: T, y: T) -> U;
    pub fn simd_gt<T, U>(x: T, y: T) -> U;
    pub fn simd_ge<T, U>(x: T, y: T) -> U;

    pub fn simd_shuffle2<T, U>(x: T, y: T, idx: [u32; 2]) -> U;
    pub fn simd_shuffle4<T, U>(x: T, y: T, idx: [u32; 4]) -> U;
    pub fn simd_shuffle8<T, U>(x: T, y: T, idx: [u32; 8]) -> U;
    pub fn simd_shuffle16<T, U>(x: T, y: T, idx: [u32; 16]) -> U;
    pub fn simd_shuffle32<T, U>(x: T, y: T, idx: [u32; 32]) -> U;
    pub fn simd_shuffle64<T, U>(x: T, y: T, idx: [u32; 64]) -> U;
    pub fn simd_shuffle128<T, U>(x: T, y: T, idx: [u32; 128]) -> U;

    pub fn simd_insert<T, U>(x: T, idx: u32, val: U) -> T;
    pub fn simd_extract<T, U>(x: T, idx: u32) -> U;

    pub fn simd_cast<T, U>(x: T) -> U;

    pub fn simd_add<T>(x: T, y: T) -> T;
    pub fn simd_sub<T>(x: T, y: T) -> T;
    pub fn simd_mul<T>(x: T, y: T) -> T;
    pub fn simd_div<T>(x: T, y: T) -> T;
    pub fn simd_shl<T>(x: T, y: T) -> T;
    pub fn simd_shr<T>(x: T, y: T) -> T;
    pub fn simd_and<T>(x: T, y: T) -> T;
    pub fn simd_or<T>(x: T, y: T) -> T;
    pub fn simd_xor<T>(x: T, y: T) -> T;

    pub fn simd_reduce_add_unordered<T, U>(x: T) -> U;
    pub fn simd_reduce_mul_unordered<T, U>(x: T) -> U;
    pub fn simd_reduce_add_ordered<T, U>(x: T, acc: U) -> U;
    pub fn simd_reduce_mul_ordered<T, U>(x: T, acc: U) -> U;
    pub fn simd_reduce_min<T, U>(x: T) -> U;
    pub fn simd_reduce_max<T, U>(x: T) -> U;
    pub fn simd_reduce_min_nanless<T, U>(x: T) -> U;
    pub fn simd_reduce_max_nanless<T, U>(x: T) -> U;
    pub fn simd_reduce_and<T, U>(x: T) -> U;
    pub fn simd_reduce_or<T, U>(x: T) -> U;
    pub fn simd_reduce_xor<T, U>(x: T) -> U;
    pub fn simd_reduce_all<T>(x: T) -> bool;
    pub fn simd_reduce_any<T>(x: T) -> bool;

    pub fn simd_select<M, T>(m: M, a: T, b: T) -> T;
    pub fn simd_select_bitmask<M, T>(m: M, a: T, b: T) -> T;

    pub fn simd_fmin<T>(a: T, b: T) -> T;
    pub fn simd_fmax<T>(a: T, b: T) -> T;

    pub fn simd_fsqrt<T>(a: T) -> T;
    pub fn simd_fma<T>(a: T, b: T, c: T) -> T;
}

macro_rules! simd_ty {
    ($id:ident [$ety:ident]: $($elem_ty:ident),* | $($elem_name:ident),*) => {
        #[repr(simd)]
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub(crate) struct $id($(pub $elem_ty),*);

        #[allow(clippy::use_self)]
        impl $id {
            #[inline]
            pub(crate) const fn new($($elem_name: $elem_ty),*) -> Self {
                $id($($elem_name),*)
            }
            // FIXME: Workaround rust@60637
            #[inline(always)]
            pub(crate) const fn splat(value: $ety) -> Self {
                $id($({
                    #[allow(non_camel_case_types, dead_code)]
                    struct $elem_name;
                    value
                }),*)
            }

            // FIXME: Workaround rust@60637
            #[inline(always)]
            pub(crate) fn extract(self, index: usize) -> $ety {
                unsafe {
                    simd_extract(self, index as u32)
                }
            }
        }
    }
}

macro_rules! simd_m_ty {
    ($id:ident [$ety:ident]: $($elem_ty:ident),* | $($elem_name:ident),*) => {
        #[repr(simd)]
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub(crate) struct $id($(pub $elem_ty),*);

        #[allow(clippy::use_self)]
        impl $id {
            #[inline]
            const fn bool_to_internal(x: bool) -> $ety {
                [0 as $ety, !(0 as $ety)][x as usize]
            }

            #[inline]
            pub(crate) const fn new($($elem_name: bool),*) -> Self {
                $id($(Self::bool_to_internal($elem_name)),*)
            }

            // FIXME: Workaround rust@60637
            #[inline(always)]
            pub(crate) const fn splat(value: bool) -> Self {
                $id($({
                    #[allow(non_camel_case_types, dead_code)]
                    struct $elem_name;
                    Self::bool_to_internal(value)
                }),*)
            }

            // FIXME: Workaround rust@60637
            #[inline(always)]
            pub(crate) fn extract(self, index: usize) -> bool {
                let r: $ety = unsafe {
                    simd_extract(self, index as u32)
                };
                r != 0
            }
        }
    }
}

// 16-bit wide types:

simd_ty!(u8x2[u8]: u8, u8 | x0, x1);
simd_ty!(i8x2[i8]: i8, i8 | x0, x1);

// 32-bit wide types:

simd_ty!(u8x4[u8]: u8, u8, u8, u8 | x0, x1, x2, x3);
simd_ty!(u16x2[u16]: u16, u16 | x0, x1);

simd_ty!(i8x4[i8]: i8, i8, i8, i8 | x0, x1, x2, x3);
simd_ty!(i16x2[i16]: i16, i16 | x0, x1);

// 64-bit wide types:

simd_ty!(
    u8x8[u8]: u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7
);
simd_ty!(u16x4[u16]: u16, u16, u16, u16 | x0, x1, x2, x3);
simd_ty!(u32x2[u32]: u32, u32 | x0, x1);
simd_ty!(u64x1[u64]: u64 | x1);

simd_ty!(
    i8x8[i8]: i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7
);
simd_ty!(i16x4[i16]: i16, i16, i16, i16 | x0, x1, x2, x3);
simd_ty!(i32x2[i32]: i32, i32 | x0, x1);
simd_ty!(i64x1[i64]: i64 | x1);

simd_ty!(f32x2[f32]: f32, f32 | x0, x1);

// 128-bit wide types:

simd_ty!(
    u8x16[u8]: u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15
);
simd_ty!(
    u16x8[u16]: u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7
);
simd_ty!(u32x4[u32]: u32, u32, u32, u32 | x0, x1, x2, x3);
simd_ty!(u64x2[u64]: u64, u64 | x0, x1);

simd_ty!(
    i8x16[i8]: i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15
);
simd_ty!(
    i16x8[i16]: i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7
);
simd_ty!(i32x4[i32]: i32, i32, i32, i32 | x0, x1, x2, x3);
simd_ty!(i64x2[i64]: i64, i64 | x0, x1);

simd_ty!(f32x4[f32]: f32, f32, f32, f32 | x0, x1, x2, x3);
simd_ty!(f64x2[f64]: f64, f64 | x0, x1);

simd_m_ty!(
    m8x16[i8]: i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15
);
simd_m_ty!(
    m16x8[i16]: i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7
);
simd_m_ty!(m32x4[i32]: i32, i32, i32, i32 | x0, x1, x2, x3);
simd_m_ty!(m64x2[i64]: i64, i64 | x0, x1);

// 256-bit wide types:

simd_ty!(
    u8x32[u8]: u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15,
    x16,
    x17,
    x18,
    x19,
    x20,
    x21,
    x22,
    x23,
    x24,
    x25,
    x26,
    x27,
    x28,
    x29,
    x30,
    x31
);
simd_ty!(
    u16x16[u16]: u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15
);
simd_ty!(
    u32x8[u32]: u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7
);
simd_ty!(u64x4[u64]: u64, u64, u64, u64 | x0, x1, x2, x3);

simd_ty!(
    i8x32[i8]: i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8,
    i8 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15,
    x16,
    x17,
    x18,
    x19,
    x20,
    x21,
    x22,
    x23,
    x24,
    x25,
    x26,
    x27,
    x28,
    x29,
    x30,
    x31
);
simd_ty!(
    i16x16[i16]: i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16,
    i16 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15
);
simd_ty!(
    i32x8[i32]: i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7
);
simd_ty!(i64x4[i64]: i64, i64, i64, i64 | x0, x1, x2, x3);

simd_ty!(
    f32x8[f32]: f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7
);

// 512-bit wide types:

simd_ty!(
    i32x16[i32]: i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15
);

simd_ty!(
    i64x8[i64]: i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64 | x0,
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7
);

types! {
    /// ARM-specific 64-bit wide vector of one packed `f64`.
    pub struct float64x1_t(f64); // FIXME: check this!
    /// ARM-specific 128-bit wide vector of two packed `f64`.
    pub struct float64x2_t(f64, f64);
    /// ARM-specific 64-bit wide vector of one packed `p64`.
    pub struct poly64x1_t(i64); // FIXME: check this!
    /// ARM-specific 64-bit wide vector of one packed `p64`.
    pub struct poly64_t(i64); // FIXME: check this!
    /// ARM-specific 64-bit wide vector of two packed `p64`.
    pub struct poly64x2_t(i64, i64); // FIXME: check this!
    /// ARM-specific 128-bit wide vector of one packed `p64`.
    pub struct poly128_t(i128); // FIXME: check this!
}

/// ARM-specific type containing two `int8x16_t` vectors.
#[derive(Copy, Clone)]
pub struct int8x16x2_t(pub int8x16_t, pub int8x16_t);
/// ARM-specific type containing three `int8x16_t` vectors.
#[derive(Copy, Clone)]
pub struct int8x16x3_t(pub int8x16_t, pub int8x16_t, pub int8x16_t);
/// ARM-specific type containing four `int8x16_t` vectors.
#[derive(Copy, Clone)]
pub struct int8x16x4_t(pub int8x16_t, pub int8x16_t, pub int8x16_t, pub int8x16_t);

/// ARM-specific type containing two `uint8x16_t` vectors.
#[derive(Copy, Clone)]
pub struct uint8x16x2_t(pub uint8x16_t, pub uint8x16_t);
/// ARM-specific type containing three `uint8x16_t` vectors.
#[derive(Copy, Clone)]
pub struct uint8x16x3_t(pub uint8x16_t, pub uint8x16_t, pub uint8x16_t);
/// ARM-specific type containing four `uint8x16_t` vectors.
#[derive(Copy, Clone)]
pub struct uint8x16x4_t(
    pub uint8x16_t,
    pub uint8x16_t,
    pub uint8x16_t,
    pub uint8x16_t,
);

/// ARM-specific type containing two `poly8x16_t` vectors.
#[derive(Copy, Clone)]
pub struct poly8x16x2_t(pub poly8x16_t, pub poly8x16_t);
/// ARM-specific type containing three `poly8x16_t` vectors.
#[derive(Copy, Clone)]
pub struct poly8x16x3_t(pub poly8x16_t, pub poly8x16_t, pub poly8x16_t);
/// ARM-specific type containing four `poly8x16_t` vectors.
#[derive(Copy, Clone)]
pub struct poly8x16x4_t(
    pub poly8x16_t,
    pub poly8x16_t,
    pub poly8x16_t,
    pub poly8x16_t,
);

#[allow(improper_ctypes)]
extern "C" {
    #[link_name = "llvm.aarch64.neon.pmull64"]
    fn vmull_p64_(a: i64, b: i64) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.addp.v16i8"]
    fn vpaddq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;

    #[link_name = "llvm.aarch64.neon.smaxv.i8.v8i8"]
    fn vmaxv_s8_(a: int8x8_t) -> i8;
    #[link_name = "llvm.aarch64.neon.smaxv.i8.6i8"]
    fn vmaxvq_s8_(a: int8x16_t) -> i8;
    #[link_name = "llvm.aarch64.neon.smaxv.i16.v4i16"]
    fn vmaxv_s16_(a: int16x4_t) -> i16;
    #[link_name = "llvm.aarch64.neon.smaxv.i16.v8i16"]
    fn vmaxvq_s16_(a: int16x8_t) -> i16;
    #[link_name = "llvm.aarch64.neon.smaxv.i32.v2i32"]
    fn vmaxv_s32_(a: int32x2_t) -> i32;
    #[link_name = "llvm.aarch64.neon.smaxv.i32.v4i32"]
    fn vmaxvq_s32_(a: int32x4_t) -> i32;

    #[link_name = "llvm.aarch64.neon.umaxv.i8.v8i8"]
    fn vmaxv_u8_(a: uint8x8_t) -> u8;
    #[link_name = "llvm.aarch64.neon.umaxv.i8.6i8"]
    fn vmaxvq_u8_(a: uint8x16_t) -> u8;
    #[link_name = "llvm.aarch64.neon.umaxv.i16.v4i16"]
    fn vmaxv_u16_(a: uint16x4_t) -> u16;
    #[link_name = "llvm.aarch64.neon.umaxv.i16.v8i16"]
    fn vmaxvq_u16_(a: uint16x8_t) -> u16;
    #[link_name = "llvm.aarch64.neon.umaxv.i32.v2i32"]
    fn vmaxv_u32_(a: uint32x2_t) -> u32;
    #[link_name = "llvm.aarch64.neon.umaxv.i32.v4i32"]
    fn vmaxvq_u32_(a: uint32x4_t) -> u32;

    #[link_name = "llvm.aarch64.neon.fmaxv.f32.v2f32"]
    fn vmaxv_f32_(a: float32x2_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fmaxv.f32.v4f32"]
    fn vmaxvq_f32_(a: float32x4_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fmaxv.f64.v2f64"]
    fn vmaxvq_f64_(a: float64x2_t) -> f64;

    #[link_name = "llvm.aarch64.neon.sminv.i8.v8i8"]
    fn vminv_s8_(a: int8x8_t) -> i8;
    #[link_name = "llvm.aarch64.neon.sminv.i8.6i8"]
    fn vminvq_s8_(a: int8x16_t) -> i8;
    #[link_name = "llvm.aarch64.neon.sminv.i16.v4i16"]
    fn vminv_s16_(a: int16x4_t) -> i16;
    #[link_name = "llvm.aarch64.neon.sminv.i16.v8i16"]
    fn vminvq_s16_(a: int16x8_t) -> i16;
    #[link_name = "llvm.aarch64.neon.sminv.i32.v2i32"]
    fn vminv_s32_(a: int32x2_t) -> i32;
    #[link_name = "llvm.aarch64.neon.sminv.i32.v4i32"]
    fn vminvq_s32_(a: int32x4_t) -> i32;

    #[link_name = "llvm.aarch64.neon.uminv.i8.v8i8"]
    fn vminv_u8_(a: uint8x8_t) -> u8;
    #[link_name = "llvm.aarch64.neon.uminv.i8.6i8"]
    fn vminvq_u8_(a: uint8x16_t) -> u8;
    #[link_name = "llvm.aarch64.neon.uminv.i16.v4i16"]
    fn vminv_u16_(a: uint16x4_t) -> u16;
    #[link_name = "llvm.aarch64.neon.uminv.i16.v8i16"]
    fn vminvq_u16_(a: uint16x8_t) -> u16;
    #[link_name = "llvm.aarch64.neon.uminv.i32.v2i32"]
    fn vminv_u32_(a: uint32x2_t) -> u32;
    #[link_name = "llvm.aarch64.neon.uminv.i32.v4i32"]
    fn vminvq_u32_(a: uint32x4_t) -> u32;

    #[link_name = "llvm.aarch64.neon.fminv.f32.v2f32"]
    fn vminv_f32_(a: float32x2_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fminv.f32.v4f32"]
    fn vminvq_f32_(a: float32x4_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fminv.f64.v2f64"]
    fn vminvq_f64_(a: float64x2_t) -> f64;

    #[link_name = "llvm.aarch64.neon.sminp.v16i8"]
    fn vpminq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.sminp.v8i16"]
    fn vpminq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.sminp.v4i32"]
    fn vpminq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.uminp.v16i8"]
    fn vpminq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    #[link_name = "llvm.aarch64.neon.uminp.v8i16"]
    fn vpminq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    #[link_name = "llvm.aarch64.neon.uminp.v4i32"]
    fn vpminq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    #[link_name = "llvm.aarch64.neon.fminp.4f32"]
    fn vpminq_f32_(a: float32x4_t, b: float32x4_t) -> float32x4_t;
    #[link_name = "llvm.aarch64.neon.fminp.v2f64"]
    fn vpminq_f64_(a: float64x2_t, b: float64x2_t) -> float64x2_t;

    #[link_name = "llvm.aarch64.neon.smaxp.v16i8"]
    fn vpmaxq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.smaxp.v8i16"]
    fn vpmaxq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.smaxp.v4i32"]
    fn vpmaxq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.umaxp.v16i8"]
    fn vpmaxq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    #[link_name = "llvm.aarch64.neon.umaxp.v8i16"]
    fn vpmaxq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    #[link_name = "llvm.aarch64.neon.umaxp.v4i32"]
    fn vpmaxq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    #[link_name = "llvm.aarch64.neon.fmaxp.4f32"]
    fn vpmaxq_f32_(a: float32x4_t, b: float32x4_t) -> float32x4_t;
    #[link_name = "llvm.aarch64.neon.fmaxp.v2f64"]
    fn vpmaxq_f64_(a: float64x2_t, b: float64x2_t) -> float64x2_t;

    #[link_name = "llvm.aarch64.neon.tbl1.v8i8"]
    fn vqtbl1(a: int8x16_t, b: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbl1.v16i8"]
    fn vqtbl1q(a: int8x16_t, b: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbx1.v8i8"]
    fn vqtbx1(a: int8x8_t, b: int8x16_t, c: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbx1.v16i8"]
    fn vqtbx1q(a: int8x16_t, b: int8x16_t, c: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbl2.v8i8"]
    fn vqtbl2(a0: int8x16_t, a1: int8x16_t, b: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbl2.v16i8"]
    fn vqtbl2q(a0: int8x16_t, a1: int8x16_t, b: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbx2.v8i8"]
    fn vqtbx2(a: int8x8_t, b0: int8x16_t, b1: int8x16_t, c: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbx2.v16i8"]
    fn vqtbx2q(a: int8x16_t, b0: int8x16_t, b1: int8x16_t, c: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbl3.v8i8"]
    fn vqtbl3(a0: int8x16_t, a1: int8x16_t, a2: int8x16_t, b: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbl3.v16i8"]
    fn vqtbl3q(a0: int8x16_t, a1: int8x16_t, a2: int8x16_t, b: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbx3.v8i8"]
    fn vqtbx3(a: int8x8_t, b0: int8x16_t, b1: int8x16_t, b2: int8x16_t, c: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbx3.v16i8"]
    fn vqtbx3q(
        a: int8x16_t,
        b0: int8x16_t,
        b1: int8x16_t,
        b2: int8x16_t,
        c: uint8x16_t,
    ) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbl4.v8i8"]
    fn vqtbl4(a0: int8x16_t, a1: int8x16_t, a2: int8x16_t, a3: int8x16_t, b: uint8x8_t)
        -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbl4.v16i8"]
    fn vqtbl4q(
        a0: int8x16_t,
        a1: int8x16_t,
        a2: int8x16_t,
        a3: int8x16_t,
        b: uint8x16_t,
    ) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbx4.v8i8"]
    fn vqtbx4(
        a: int8x8_t,
        b0: int8x16_t,
        b1: int8x16_t,
        b2: int8x16_t,
        b3: int8x16_t,
        c: uint8x8_t,
    ) -> int8x8_t;

    #[link_name = "llvm.aarch64.neon.tbx4.v16i8"]
    fn vqtbx4q(
        a: int8x16_t,
        b0: int8x16_t,
        b1: int8x16_t,
        b2: int8x16_t,
        b3: int8x16_t,
        c: uint8x16_t,
    ) -> int8x16_t;
}

/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn vpaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    vpaddq_u8_(a, b)
}

/// Polynomial multiply long
#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn vmull_p64(a: poly64_t, b: poly64_t) -> poly128_t {
    transmute(vmull_p64_(transmute(a), transmute(b)))
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn vadd_f64(a: float64x1_t, b: float64x1_t) -> float64x1_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn vaddq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn vaddd_s64(a: i64, b: i64) -> i64 {
    let a: int64x1_t = transmute(a);
    let b: int64x1_t = transmute(b);
    simd_extract(simd_add(a, b), 0)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn vaddd_u64(a: u64, b: u64) -> u64 {
    let a: uint64x1_t = transmute(a);
    let b: uint64x1_t = transmute(b);
    simd_extract(simd_add(a, b), 0)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxv_s8(a: int8x8_t) -> i8 {
    vmaxv_s8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxvq_s8(a: int8x16_t) -> i8 {
    vmaxvq_s8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxv_s16(a: int16x4_t) -> i16 {
    vmaxv_s16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxvq_s16(a: int16x8_t) -> i16 {
    vmaxvq_s16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxv_s32(a: int32x2_t) -> i32 {
    vmaxv_s32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxvq_s32(a: int32x4_t) -> i32 {
    vmaxvq_s32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxv_u8(a: uint8x8_t) -> u8 {
    vmaxv_u8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxvq_u8(a: uint8x16_t) -> u8 {
    vmaxvq_u8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxv_u16(a: uint16x4_t) -> u16 {
    vmaxv_u16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxvq_u16(a: uint16x8_t) -> u16 {
    vmaxvq_u16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxv_u32(a: uint32x2_t) -> u32 {
    vmaxv_u32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxvq_u32(a: uint32x4_t) -> u32 {
    vmaxvq_u32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxv_f32(a: float32x2_t) -> f32 {
    vmaxv_f32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxvq_f32(a: float32x4_t) -> f32 {
    vmaxvq_f32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmaxvq_f64(a: float64x2_t) -> f64 {
    vmaxvq_f64_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminv_s8(a: int8x8_t) -> i8 {
    vminv_s8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminvq_s8(a: int8x16_t) -> i8 {
    vminvq_s8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminv_s16(a: int16x4_t) -> i16 {
    vminv_s16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminvq_s16(a: int16x8_t) -> i16 {
    vminvq_s16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminv_s32(a: int32x2_t) -> i32 {
    vminv_s32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminvq_s32(a: int32x4_t) -> i32 {
    vminvq_s32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminv_u8(a: uint8x8_t) -> u8 {
    vminv_u8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminvq_u8(a: uint8x16_t) -> u8 {
    vminvq_u8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminv_u16(a: uint16x4_t) -> u16 {
    vminv_u16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminvq_u16(a: uint16x8_t) -> u16 {
    vminvq_u16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminv_u32(a: uint32x2_t) -> u32 {
    vminv_u32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminvq_u32(a: uint32x4_t) -> u32 {
    vminvq_u32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminv_f32(a: float32x2_t) -> f32 {
    vminv_f32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminvq_f32(a: float32x4_t) -> f32 {
    vminvq_f32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vminvq_f64(a: float64x2_t) -> f64 {
    vminvq_f64_(a)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpminq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vpminq_s8_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpminq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    vpminq_s16_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpminq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    vpminq_s32_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpminq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    vpminq_u8_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpminq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    vpminq_u16_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpminq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    vpminq_u32_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpminq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    vpminq_f32_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpminq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    vpminq_f64_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpmaxq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vpmaxq_s8_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpmaxq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    vpmaxq_s16_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpmaxq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    vpmaxq_s32_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpmaxq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    vpmaxq_u8_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpmaxq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    vpmaxq_u16_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpmaxq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    vpmaxq_u32_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpmaxq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    vpmaxq_f32_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vpmaxq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    vpmaxq_f64_(a, b)
}

///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_s8(low: int8x8_t, high: int8x8_t) -> int8x16_t {
//    simd_shuffle16(
//        low,
//        high,
//        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
//    )
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_s16(low: int16x4_t, high: int16x4_t) -> int16x8_t {
//    simd_shuffle8(low, high, [0, 1, 2, 3, 4, 5, 6, 7])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_s32(low: int32x2_t, high: int32x2_t) -> int32x4_t {
//    simd_shuffle4(low, high, [0, 1, 2, 3])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_s64(low: int64x1_t, high: int64x1_t) -> int64x2_t {
//    simd_shuffle2(low, high, [0, 1])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_u8(low: uint8x8_t, high: uint8x8_t) -> uint8x16_t {
//    simd_shuffle16(
//        low,
//        high,
//        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
//    )
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_u16(low: uint16x4_t, high: uint16x4_t) -> uint16x8_t {
//    simd_shuffle8(low, high, [0, 1, 2, 3, 4, 5, 6, 7])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_u32(low: uint32x2_t, high: uint32x2_t) -> uint32x4_t {
//    simd_shuffle4(low, high, [0, 1, 2, 3])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_u64(low: uint64x1_t, high: uint64x1_t) -> uint64x2_t {
//    simd_shuffle2(low, high, [0, 1])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_p64(low: poly64x1_t, high: poly64x1_t) -> poly64x2_t {
//    simd_shuffle2(low, high, [0, 1])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_f16 ( low: float16x4_t,  high: float16x4_t) -> float16x8_t {
//    simd_shuffle8(low, high, [0, 1, 2, 3, 4, 5, 6, 7])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_f32(low: float32x2_t, high: float32x2_t) -> float32x4_t {
//    simd_shuffle4(low, high, [0, 1, 2, 3])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_p8(low: poly8x8_t, high: poly8x8_t) -> poly8x16_t {
//    simd_shuffle16(
//        low,
//        high,
//        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
//    )
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_p16(low: poly16x4_t, high: poly16x4_t) -> poly16x8_t {
//    simd_shuffle8(low, high, [0, 1, 2, 3, 4, 5, 6, 7])
//}
//
///// Vector combine
//#[inline]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vcombine_f64(low: float64x1_t, high: float64x1_t) -> float64x2_t {
//    simd_shuffle2(low, high, [0, 1])
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl1_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
//    vqtbl1_s8(vcombine_s8(a, zeroed()), transmute(b))
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl1_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
//    vqtbl1_u8(vcombine_u8(a, zeroed()), b)
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl1_p8(a: poly8x8_t, b: uint8x8_t) -> poly8x8_t {
//    vqtbl1_p8(vcombine_p8(a, zeroed()), b)
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl2_s8(a: int8x8x2_t, b: int8x8_t) -> int8x8_t {
//    vqtbl1_s8(vcombine_s8(a.0, a.1), transmute(b))
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl2_u8(a: uint8x8x2_t, b: uint8x8_t) -> uint8x8_t {
//    vqtbl1_u8(vcombine_u8(a.0, a.1), b)
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl2_p8(a: poly8x8x2_t, b: uint8x8_t) -> poly8x8_t {
//    vqtbl1_p8(vcombine_p8(a.0, a.1), b)
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl3_s8(a: int8x8x3_t, b: int8x8_t) -> int8x8_t {
//    vqtbl2_s8(
//        int8x16x2_t(vcombine_s8(a.0, a.1), vcombine_s8(a.2, zeroed())),
//        transmute(b),
//    )
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl3_u8(a: uint8x8x3_t, b: uint8x8_t) -> uint8x8_t {
//    vqtbl2_u8(
//        uint8x16x2_t(vcombine_u8(a.0, a.1), vcombine_u8(a.2, zeroed())),
//        b,
//    )
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl3_p8(a: poly8x8x3_t, b: uint8x8_t) -> poly8x8_t {
//    vqtbl2_p8(
//        poly8x16x2_t(vcombine_p8(a.0, a.1), vcombine_p8(a.2, zeroed())),
//        b,
//    )
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl4_s8(a: int8x8x4_t, b: int8x8_t) -> int8x8_t {
//    vqtbl2_s8(
//        int8x16x2_t(vcombine_s8(a.0, a.1), vcombine_s8(a.2, a.3)),
//        transmute(b),
//    )
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl4_u8(a: uint8x8x4_t, b: uint8x8_t) -> uint8x8_t {
//    vqtbl2_u8(
//        uint8x16x2_t(vcombine_u8(a.0, a.1), vcombine_u8(a.2, a.3)),
//        b,
//    )
//}
//
///// Table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbl4_p8(a: poly8x8x4_t, b: uint8x8_t) -> poly8x8_t {
//    vqtbl2_p8(
//        poly8x16x2_t(vcombine_p8(a.0, a.1), vcombine_p8(a.2, a.3)),
//        b,
//    )
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx1_s8(a: int8x8_t, b: int8x8_t, c: int8x8_t) -> int8x8_t {
//    //    use i8x8;
//    let r = vqtbx1_s8(a, vcombine_s8(b, zeroed()), transmute(c));
//    let m: int8x8_t = simd_lt(c, transmute(i8x8::splat(8)));
//    simd_select(m, r, a)
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx1_u8(a: uint8x8_t, b: uint8x8_t, c: uint8x8_t) -> uint8x8_t {
//    //    use u8x8;
//    let r = vqtbx1_u8(a, vcombine_u8(b, zeroed()), c);
//    let m: int8x8_t = simd_lt(c, transmute(u8x8::splat(8)));
//    simd_select(m, r, a)
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx1_p8(a: poly8x8_t, b: poly8x8_t, c: uint8x8_t) -> poly8x8_t {
//    //    use u8x8;
//    let r = vqtbx1_p8(a, vcombine_p8(b, zeroed()), c);
//    let m: int8x8_t = simd_lt(c, transmute(u8x8::splat(8)));
//    simd_select(m, r, a)
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx2_s8(a: int8x8_t, b: int8x8x2_t, c: int8x8_t) -> int8x8_t {
//    vqtbx1_s8(a, vcombine_s8(b.0, b.1), transmute(c))
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx2_u8(a: uint8x8_t, b: uint8x8x2_t, c: uint8x8_t) -> uint8x8_t {
//    vqtbx1_u8(a, vcombine_u8(b.0, b.1), c)
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx2_p8(a: poly8x8_t, b: poly8x8x2_t, c: uint8x8_t) -> poly8x8_t {
//    vqtbx1_p8(a, vcombine_p8(b.0, b.1), c)
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx3_s8(a: int8x8_t, b: int8x8x3_t, c: int8x8_t) -> int8x8_t {
//    //    use i8x8;
//    let r = vqtbx2_s8(
//        a,
//        int8x16x2_t(vcombine_s8(b.0, b.1), vcombine_s8(b.2, zeroed())),
//        transmute(c),
//    );
//    let m: int8x8_t = simd_lt(c, transmute(i8x8::splat(24)));
//    simd_select(m, r, a)
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx3_u8(a: uint8x8_t, b: uint8x8x3_t, c: uint8x8_t) -> uint8x8_t {
//    //    use u8x8;
//    let r = vqtbx2_u8(
//        a,
//        uint8x16x2_t(vcombine_u8(b.0, b.1), vcombine_u8(b.2, zeroed())),
//        c,
//    );
//    let m: int8x8_t = simd_lt(c, transmute(u8x8::splat(24)));
//    simd_select(m, r, a)
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx3_p8(a: poly8x8_t, b: poly8x8x3_t, c: uint8x8_t) -> poly8x8_t {
//    //    use u8x8;
//    let r = vqtbx2_p8(
//        a,
//        poly8x16x2_t(vcombine_p8(b.0, b.1), vcombine_p8(b.2, zeroed())),
//        c,
//    );
//    let m: int8x8_t = simd_lt(c, transmute(u8x8::splat(24)));
//    simd_select(m, r, a)
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx4_s8(a: int8x8_t, b: int8x8x4_t, c: int8x8_t) -> int8x8_t {
//    vqtbx2_s8(
//        a,
//        int8x16x2_t(vcombine_s8(b.0, b.1), vcombine_s8(b.2, b.3)),
//        transmute(c),
//    )
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx4_u8(a: uint8x8_t, b: uint8x8x4_t, c: uint8x8_t) -> uint8x8_t {
//    vqtbx2_u8(
//        a,
//        uint8x16x2_t(vcombine_u8(b.0, b.1), vcombine_u8(b.2, b.3)),
//        c,
//    )
//}
//
///// Extended table look-up
//#[inline]
//#[cfg(target_endian = "little")]
//#[target_feature(enable = "neon")]
//
//pub unsafe fn vtbx4_p8(a: poly8x8_t, b: poly8x8x4_t, c: uint8x8_t) -> poly8x8_t {
//    vqtbx2_p8(
//        a,
//        poly8x16x2_t(vcombine_p8(b.0, b.1), vcombine_p8(b.2, b.3)),
//        c,
//    )
//}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl1_s8(t: int8x16_t, idx: uint8x8_t) -> int8x8_t {
    vqtbl1(t, idx)
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl1q_s8(t: int8x16_t, idx: uint8x16_t) -> int8x16_t {
    vqtbl1q(t, idx)
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl1_u8(t: uint8x16_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbl1(transmute(t), transmute(idx)))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl1q_u8(t: uint8x16_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbl1q(transmute(t), transmute(idx)))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl1_p8(t: poly8x16_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbl1(transmute(t), transmute(idx)))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl1q_p8(t: poly8x16_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbl1q(transmute(t), transmute(idx)))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx1_s8(a: int8x8_t, t: int8x16_t, idx: uint8x8_t) -> int8x8_t {
    vqtbx1(a, t, idx)
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx1q_s8(a: int8x16_t, t: int8x16_t, idx: uint8x16_t) -> int8x16_t {
    vqtbx1q(a, t, idx)
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx1_u8(a: uint8x8_t, t: uint8x16_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbx1(transmute(a), transmute(t), transmute(idx)))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx1q_u8(a: uint8x16_t, t: uint8x16_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbx1q(transmute(a), transmute(t), transmute(idx)))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx1_p8(a: poly8x8_t, t: poly8x16_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbx1(transmute(a), transmute(t), transmute(idx)))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx1q_p8(a: poly8x16_t, t: poly8x16_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbx1q(transmute(a), transmute(t), transmute(idx)))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl2_s8(t: int8x16x2_t, idx: uint8x8_t) -> int8x8_t {
    vqtbl2(t.0, t.1, idx)
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl2q_s8(t: int8x16x2_t, idx: uint8x16_t) -> int8x16_t {
    vqtbl2q(t.0, t.1, idx)
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl2_u8(t: uint8x16x2_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbl2(transmute(t.0), transmute(t.1), transmute(idx)))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl2q_u8(t: uint8x16x2_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbl2q(transmute(t.0), transmute(t.1), transmute(idx)))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl2_p8(t: poly8x16x2_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbl2(transmute(t.0), transmute(t.1), transmute(idx)))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl2q_p8(t: poly8x16x2_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbl2q(transmute(t.0), transmute(t.1), transmute(idx)))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx2_s8(a: int8x8_t, t: int8x16x2_t, idx: uint8x8_t) -> int8x8_t {
    vqtbx2(a, t.0, t.1, idx)
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx2q_s8(a: int8x16_t, t: int8x16x2_t, idx: uint8x16_t) -> int8x16_t {
    vqtbx2q(a, t.0, t.1, idx)
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx2_u8(a: uint8x8_t, t: uint8x16x2_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbx2(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx2q_u8(a: uint8x16_t, t: uint8x16x2_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbx2q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx2_p8(a: poly8x8_t, t: poly8x16x2_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbx2(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx2q_p8(a: poly8x16_t, t: poly8x16x2_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbx2q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl3_s8(t: int8x16x3_t, idx: uint8x8_t) -> int8x8_t {
    vqtbl3(t.0, t.1, t.2, idx)
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl3q_s8(t: int8x16x3_t, idx: uint8x16_t) -> int8x16_t {
    vqtbl3q(t.0, t.1, t.2, idx)
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl3_u8(t: uint8x16x3_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbl3(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl3q_u8(t: uint8x16x3_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbl3q(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl3_p8(t: poly8x16x3_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbl3(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl3q_p8(t: poly8x16x3_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbl3q(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx3_s8(a: int8x8_t, t: int8x16x3_t, idx: uint8x8_t) -> int8x8_t {
    vqtbx3(a, t.0, t.1, t.2, idx)
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx3q_s8(a: int8x16_t, t: int8x16x3_t, idx: uint8x16_t) -> int8x16_t {
    vqtbx3q(a, t.0, t.1, t.2, idx)
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx3_u8(a: uint8x8_t, t: uint8x16x3_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbx3(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx3q_u8(a: uint8x16_t, t: uint8x16x3_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbx3q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx3_p8(a: poly8x8_t, t: poly8x16x3_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbx3(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx3q_p8(a: poly8x16_t, t: poly8x16x3_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbx3q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl4_s8(t: int8x16x4_t, idx: uint8x8_t) -> int8x8_t {
    vqtbl4(t.0, t.1, t.2, t.3, idx)
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl4q_s8(t: int8x16x4_t, idx: uint8x16_t) -> int8x16_t {
    vqtbl4q(t.0, t.1, t.2, t.3, idx)
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl4_u8(t: uint8x16x4_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbl4(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl4q_u8(t: uint8x16x4_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbl4q(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl4_p8(t: poly8x16x4_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbl4(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}
/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbl4q_p8(t: poly8x16x4_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbl4q(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx4_s8(a: int8x8_t, t: int8x16x4_t, idx: uint8x8_t) -> int8x8_t {
    vqtbx4(a, t.0, t.1, t.2, t.3, idx)
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx4q_s8(a: int8x16_t, t: int8x16x4_t, idx: uint8x16_t) -> int8x16_t {
    vqtbx4q(a, t.0, t.1, t.2, t.3, idx)
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx4_u8(a: uint8x8_t, t: uint8x16x4_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbx4(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx4q_u8(a: uint8x16_t, t: uint8x16x4_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbx4q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx4_p8(a: poly8x8_t, t: poly8x16x4_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbx4(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}
/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]

pub unsafe fn vqtbx4q_p8(a: poly8x16_t, t: poly8x16x4_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbx4q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vceq_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vceqq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vceq_s64(a: int64x1_t, b: int64x1_t) -> uint64x1_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vceqq_s64(a: int64x2_t, b: int64x2_t) -> uint64x2_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vceq_p64(a: poly64x1_t, b: poly64x1_t) -> uint64x1_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vceqq_p64(a: poly64x2_t, b: poly64x2_t) -> uint64x2_t {
    simd_eq(a, b)
}

/// Floating-point compare equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vceq_f64(a: float64x1_t, b: float64x1_t) -> uint64x1_t {
    simd_eq(a, b)
}

/// Floating-point compare equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vceqq_f64(a: float64x2_t, b: float64x2_t) -> uint64x2_t {
    simd_eq(a, b)
}

/// Compare signed greater than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcgt_s64(a: int64x1_t, b: int64x1_t) -> uint64x1_t {
    simd_gt(a, b)
}

/// Compare signed greater than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcgtq_s64(a: int64x2_t, b: int64x2_t) -> uint64x2_t {
    simd_gt(a, b)
}

/// Compare unsigned highe
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcgt_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_gt(a, b)
}

/// Compare unsigned highe
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcgtq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_gt(a, b)
}

/// Floating-point compare greater than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcgt_f64(a: float64x1_t, b: float64x1_t) -> uint64x1_t {
    simd_gt(a, b)
}

/// Floating-point compare greater than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcgtq_f64(a: float64x2_t, b: float64x2_t) -> uint64x2_t {
    simd_gt(a, b)
}

/// Compare signed less than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vclt_s64(a: int64x1_t, b: int64x1_t) -> uint64x1_t {
    simd_lt(a, b)
}

/// Compare signed less than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcltq_s64(a: int64x2_t, b: int64x2_t) -> uint64x2_t {
    simd_lt(a, b)
}

/// Compare unsigned less than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vclt_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_lt(a, b)
}

/// Compare unsigned less than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcltq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_lt(a, b)
}

/// Floating-point compare less than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vclt_f64(a: float64x1_t, b: float64x1_t) -> uint64x1_t {
    simd_lt(a, b)
}

/// Floating-point compare less than
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcltq_f64(a: float64x2_t, b: float64x2_t) -> uint64x2_t {
    simd_lt(a, b)
}

/// Compare signed less than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcle_s64(a: int64x1_t, b: int64x1_t) -> uint64x1_t {
    simd_le(a, b)
}

/// Compare signed less than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcleq_s64(a: int64x2_t, b: int64x2_t) -> uint64x2_t {
    simd_le(a, b)
}

/// Compare unsigned less than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcle_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_le(a, b)
}

/// Compare unsigned less than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcleq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_le(a, b)
}

/// Floating-point compare less than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcle_f64(a: float64x1_t, b: float64x1_t) -> uint64x1_t {
    simd_le(a, b)
}

/// Floating-point compare less than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcleq_f64(a: float64x2_t, b: float64x2_t) -> uint64x2_t {
    simd_le(a, b)
}

/// Compare signed greater than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcge_s64(a: int64x1_t, b: int64x1_t) -> uint64x1_t {
    simd_ge(a, b)
}

/// Compare signed greater than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcgeq_s64(a: int64x2_t, b: int64x2_t) -> uint64x2_t {
    simd_ge(a, b)
}

/// Compare unsigned greater than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcge_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_ge(a, b)
}

/// Compare unsigned greater than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcgeq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_ge(a, b)
}

/// Floating-point compare greater than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcge_f64(a: float64x1_t, b: float64x1_t) -> uint64x1_t {
    simd_ge(a, b)
}

/// Floating-point compare greater than or equal
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vcgeq_f64(a: float64x2_t, b: float64x2_t) -> uint64x2_t {
    simd_ge(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmul_f64(a: float64x1_t, b: float64x1_t) -> float64x1_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vmulq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    simd_mul(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vsub_f64(a: float64x1_t, b: float64x1_t) -> float64x1_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vsubq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    simd_sub(a, b)
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
    // FIXME: ARM-specific 64-bit wide vector of four packed `f16`.
    // pub struct float16x4_t(f16, f16, f16, f16);
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
    // FIXME: ARM-specific 128-bit wide vector of eight packed `f16`.
    // pub struct float16x8_t(f16, f16, f16, f16, f16, f16, f16);
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
}

/// ARM-specific type containing two `int8x8_t` vectors.
#[derive(Copy, Clone)]
pub struct int8x8x2_t(pub int8x8_t, pub int8x8_t);
/// ARM-specific type containing three `int8x8_t` vectors.
#[derive(Copy, Clone)]
pub struct int8x8x3_t(pub int8x8_t, pub int8x8_t, pub int8x8_t);
/// ARM-specific type containing four `int8x8_t` vectors.
#[derive(Copy, Clone)]
pub struct int8x8x4_t(pub int8x8_t, pub int8x8_t, pub int8x8_t, pub int8x8_t);

/// ARM-specific type containing two `uint8x8_t` vectors.
#[derive(Copy, Clone)]
pub struct uint8x8x2_t(pub uint8x8_t, pub uint8x8_t);
/// ARM-specific type containing three `uint8x8_t` vectors.
#[derive(Copy, Clone)]
pub struct uint8x8x3_t(pub uint8x8_t, pub uint8x8_t, pub uint8x8_t);
/// ARM-specific type containing four `uint8x8_t` vectors.
#[derive(Copy, Clone)]
pub struct uint8x8x4_t(pub uint8x8_t, pub uint8x8_t, pub uint8x8_t, pub uint8x8_t);

/// ARM-specific type containing two `poly8x8_t` vectors.
#[derive(Copy, Clone)]
pub struct poly8x8x2_t(pub poly8x8_t, pub poly8x8_t);
/// ARM-specific type containing three `poly8x8_t` vectors.
#[derive(Copy, Clone)]
pub struct poly8x8x3_t(pub poly8x8_t, pub poly8x8_t, pub poly8x8_t);
/// ARM-specific type containing four `poly8x8_t` vectors.
#[derive(Copy, Clone)]
pub struct poly8x8x4_t(pub poly8x8_t, pub poly8x8_t, pub poly8x8_t, pub poly8x8_t);

#[allow(improper_ctypes)]
extern "C" {
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrsqrte.v2f32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.frsqrte.v2f32")]
    fn frsqrte_v2f32(a: float32x2_t) -> float32x2_t;

    //uint32x2_t vqmovn_u64 (uint64x2_t a)
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqmovnu.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqxtn.v2i32")]
    fn vqmovn_u64_(a: uint64x2_t) -> uint32x2_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmins.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sminp.v8i8")]
    fn vpmins_v8i8(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmins.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sminp.v4i16")]
    fn vpmins_v4i16(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmins.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sminp.v2i32")]
    fn vpmins_v2i32(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpminu.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uminp.v8i8")]
    fn vpminu_v8i8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpminu.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uminp.v4i16")]
    fn vpminu_v4i16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpminu.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uminp.v2i32")]
    fn vpminu_v2i32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmins.v2f32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.fminp.v2f32")]
    fn vpminf_v2f32(a: float32x2_t, b: float32x2_t) -> float32x2_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxs.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.smaxp.v8i8")]
    fn vpmaxs_v8i8(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxs.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.smaxp.v4i16")]
    fn vpmaxs_v4i16(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxs.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.smaxp.v2i32")]
    fn vpmaxs_v2i32(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxu.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.umaxp.v8i8")]
    fn vpmaxu_v8i8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxu.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.umaxp.v4i16")]
    fn vpmaxu_v4i16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxu.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.umaxp.v2i32")]
    fn vpmaxu_v2i32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxs.v2f32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.fmaxp.v2f32")]
    fn vpmaxf_v2f32(a: float32x2_t, b: float32x2_t) -> float32x2_t;
}

#[cfg(target_arch = "arm")]
#[allow(improper_ctypes)]
extern "C" {
    #[link_name = "llvm.arm.neon.vtbl1"]
    fn vtbl1(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    #[link_name = "llvm.arm.neon.vtbl2"]
    fn vtbl2(a: int8x8_t, b: int8x8_t, b: int8x8_t) -> int8x8_t;
    #[link_name = "llvm.arm.neon.vtbl3"]
    fn vtbl3(a: int8x8_t, b: int8x8_t, b: int8x8_t, c: int8x8_t) -> int8x8_t;
    #[link_name = "llvm.arm.neon.vtbl4"]
    fn vtbl4(a: int8x8_t, b: int8x8_t, b: int8x8_t, c: int8x8_t, d: int8x8_t) -> int8x8_t;

    #[link_name = "llvm.arm.neon.vtbx1"]
    fn vtbx1(a: int8x8_t, b: int8x8_t, b: int8x8_t) -> int8x8_t;
    #[link_name = "llvm.arm.neon.vtbx2"]
    fn vtbx2(a: int8x8_t, b: int8x8_t, b: int8x8_t, c: int8x8_t) -> int8x8_t;
    #[link_name = "llvm.arm.neon.vtbx3"]
    fn vtbx3(a: int8x8_t, b: int8x8_t, b: int8x8_t, c: int8x8_t, d: int8x8_t) -> int8x8_t;
    #[link_name = "llvm.arm.neon.vtbx4"]
    fn vtbx4(
        a: int8x8_t,
        b: int8x8_t,
        b: int8x8_t,
        c: int8x8_t,
        d: int8x8_t,
        e: int8x8_t,
    ) -> int8x8_t;
}

/// Unsigned saturating extract narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqmovn_u64(a: uint64x2_t) -> uint32x2_t {
    vqmovn_u64_(a)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vadd_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vadd_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vadd_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddq_s64(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vadd_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vadd_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vadd_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vadd_f32(a: float32x2_t, b: float32x2_t) -> float32x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    simd_add(a, b)
}

/// Vector long add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddl_s8(a: int8x8_t, b: int8x8_t) -> int16x8_t {
    let a: int16x8_t = simd_cast(a);
    let b: int16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Vector long add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddl_s16(a: int16x4_t, b: int16x4_t) -> int32x4_t {
    let a: int32x4_t = simd_cast(a);
    let b: int32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Vector long add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddl_s32(a: int32x2_t, b: int32x2_t) -> int64x2_t {
    let a: int64x2_t = simd_cast(a);
    let b: int64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Vector long add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddl_u8(a: uint8x8_t, b: uint8x8_t) -> uint16x8_t {
    let a: uint16x8_t = simd_cast(a);
    let b: uint16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Vector long add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddl_u16(a: uint16x4_t, b: uint16x4_t) -> uint32x4_t {
    let a: uint32x4_t = simd_cast(a);
    let b: uint32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Vector long add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vaddl_u32(a: uint32x2_t, b: uint32x2_t) -> uint64x2_t {
    let a: uint64x2_t = simd_cast(a);
    let b: uint64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovn_s16(a: int16x8_t) -> int8x8_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovn_s32(a: int32x4_t) -> int16x4_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovn_s64(a: int64x2_t) -> int32x2_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovn_u16(a: uint16x8_t) -> uint8x8_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovn_u32(a: uint32x4_t) -> uint16x4_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovn_u64(a: uint64x2_t) -> uint32x2_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovl_s8(a: int8x8_t) -> int16x8_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovl_s16(a: int16x4_t) -> int32x4_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovl_s32(a: int32x2_t) -> int64x2_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovl_u8(a: uint8x8_t) -> uint16x8_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovl_u16(a: uint16x4_t) -> uint32x4_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovl_u32(a: uint32x2_t) -> uint64x2_t {
    simd_cast(a)
}

/// Reciprocal square-root estimate.
#[inline]
#[target_feature(enable = "neon")]

pub unsafe fn vrsqrte_f32(a: float32x2_t) -> float32x2_t {
    frsqrte_v2f32(a)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvn_s8(a: int8x8_t) -> int8x8_t {
    let b = int8x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvnq_s8(a: int8x16_t) -> int8x16_t {
    let b = int8x16_t(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvn_s16(a: int16x4_t) -> int16x4_t {
    let b = int16x4_t(-1, -1, -1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvnq_s16(a: int16x8_t) -> int16x8_t {
    let b = int16x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvn_s32(a: int32x2_t) -> int32x2_t {
    let b = int32x2_t(-1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvnq_s32(a: int32x4_t) -> int32x4_t {
    let b = int32x4_t(-1, -1, -1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvn_u8(a: uint8x8_t) -> uint8x8_t {
    let b = uint8x8_t(255, 255, 255, 255, 255, 255, 255, 255);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvnq_u8(a: uint8x16_t) -> uint8x16_t {
    let b = uint8x16_t(
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    );
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvn_u16(a: uint16x4_t) -> uint16x4_t {
    let b = uint16x4_t(65_535, 65_535, 65_535, 65_535);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvnq_u16(a: uint16x8_t) -> uint16x8_t {
    let b = uint16x8_t(
        65_535, 65_535, 65_535, 65_535, 65_535, 65_535, 65_535, 65_535,
    );
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvn_u32(a: uint32x2_t) -> uint32x2_t {
    let b = uint32x2_t(4_294_967_295, 4_294_967_295);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvnq_u32(a: uint32x4_t) -> uint32x4_t {
    let b = uint32x4_t(4_294_967_295, 4_294_967_295, 4_294_967_295, 4_294_967_295);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvn_p8(a: poly8x8_t) -> poly8x8_t {
    let b = poly8x8_t(255, 255, 255, 255, 255, 255, 255, 255);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmvnq_p8(a: poly8x16_t) -> poly8x16_t {
    let b = poly8x16_t(
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    );
    simd_xor(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmin_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    vpmins_v8i8(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmin_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    vpmins_v4i16(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmin_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    vpmins_v2i32(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmin_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    vpminu_v8i8(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmin_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    vpminu_v4i16(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmin_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    vpminu_v2i32(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmin_f32(a: float32x2_t, b: float32x2_t) -> float32x2_t {
    vpminf_v2f32(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmax_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    vpmaxs_v8i8(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmax_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    vpmaxs_v4i16(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmax_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    vpmaxs_v2i32(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmax_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    vpmaxu_v8i8(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmax_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    vpmaxu_v4i16(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmax_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    vpmaxu_v2i32(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vpmax_f32(a: float32x2_t, b: float32x2_t) -> float32x2_t {
    vpmaxf_v2f32(a, b)
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl1_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    vtbl1(a, b)
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl1_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    transmute(vtbl1(transmute(a), transmute(b)))
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl1_p8(a: poly8x8_t, b: uint8x8_t) -> poly8x8_t {
    transmute(vtbl1(transmute(a), transmute(b)))
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl2_s8(a: int8x8x2_t, b: int8x8_t) -> int8x8_t {
    vtbl2(a.0, a.1, b)
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl2_u8(a: uint8x8x2_t, b: uint8x8_t) -> uint8x8_t {
    transmute(vtbl2(transmute(a.0), transmute(a.1), transmute(b)))
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl2_p8(a: poly8x8x2_t, b: uint8x8_t) -> poly8x8_t {
    transmute(vtbl2(transmute(a.0), transmute(a.1), transmute(b)))
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl3_s8(a: int8x8x3_t, b: int8x8_t) -> int8x8_t {
    vtbl3(a.0, a.1, a.2, b)
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl3_u8(a: uint8x8x3_t, b: uint8x8_t) -> uint8x8_t {
    transmute(vtbl3(
        transmute(a.0),
        transmute(a.1),
        transmute(a.2),
        transmute(b),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl3_p8(a: poly8x8x3_t, b: uint8x8_t) -> poly8x8_t {
    transmute(vtbl3(
        transmute(a.0),
        transmute(a.1),
        transmute(a.2),
        transmute(b),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl4_s8(a: int8x8x4_t, b: int8x8_t) -> int8x8_t {
    vtbl4(a.0, a.1, a.2, a.3, b)
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl4_u8(a: uint8x8x4_t, b: uint8x8_t) -> uint8x8_t {
    transmute(vtbl4(
        transmute(a.0),
        transmute(a.1),
        transmute(a.2),
        transmute(a.3),
        transmute(b),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbl4_p8(a: poly8x8x4_t, b: uint8x8_t) -> poly8x8_t {
    transmute(vtbl4(
        transmute(a.0),
        transmute(a.1),
        transmute(a.2),
        transmute(a.3),
        transmute(b),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx1_s8(a: int8x8_t, b: int8x8_t, c: int8x8_t) -> int8x8_t {
    vtbx1(a, b, c)
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx1_u8(a: uint8x8_t, b: uint8x8_t, c: uint8x8_t) -> uint8x8_t {
    transmute(vtbx1(transmute(a), transmute(b), transmute(c)))
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx1_p8(a: poly8x8_t, b: poly8x8_t, c: uint8x8_t) -> poly8x8_t {
    transmute(vtbx1(transmute(a), transmute(b), transmute(c)))
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx2_s8(a: int8x8_t, b: int8x8x2_t, c: int8x8_t) -> int8x8_t {
    vtbx2(a, b.0, b.1, c)
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx2_u8(a: uint8x8_t, b: uint8x8x2_t, c: uint8x8_t) -> uint8x8_t {
    transmute(vtbx2(
        transmute(a),
        transmute(b.0),
        transmute(b.1),
        transmute(c),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx2_p8(a: poly8x8_t, b: poly8x8x2_t, c: uint8x8_t) -> poly8x8_t {
    transmute(vtbx2(
        transmute(a),
        transmute(b.0),
        transmute(b.1),
        transmute(c),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx3_s8(a: int8x8_t, b: int8x8x3_t, c: int8x8_t) -> int8x8_t {
    vtbx3(a, b.0, b.1, b.2, c)
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx3_u8(a: uint8x8_t, b: uint8x8x3_t, c: uint8x8_t) -> uint8x8_t {
    transmute(vtbx3(
        transmute(a),
        transmute(b.0),
        transmute(b.1),
        transmute(b.2),
        transmute(c),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx3_p8(a: poly8x8_t, b: poly8x8x3_t, c: uint8x8_t) -> poly8x8_t {
    transmute(vtbx3(
        transmute(a),
        transmute(b.0),
        transmute(b.1),
        transmute(b.2),
        transmute(c),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx4_s8(a: int8x8_t, b: int8x8x4_t, c: int8x8_t) -> int8x8_t {
    vtbx4(a, b.0, b.1, b.2, b.3, c)
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx4_u8(a: uint8x8_t, b: uint8x8x4_t, c: uint8x8_t) -> uint8x8_t {
    transmute(vtbx4(
        transmute(a),
        transmute(b.0),
        transmute(b.1),
        transmute(b.2),
        transmute(b.3),
        transmute(c),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_arch = "arm")]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon,v7")]

pub unsafe fn vtbx4_p8(a: poly8x8_t, b: poly8x8x4_t, c: uint8x8_t) -> poly8x8_t {
    transmute(vtbx4(
        transmute(a),
        transmute(b.0),
        transmute(b.1),
        transmute(b.2),
        transmute(b.3),
        transmute(c),
    ))
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
//#[rustc_args_required_const(1)]

// Based on the discussioj in https://github.com/rust-lang/stdarch/pull/792
// `mov` seems to be an acceptable intrinsic to compile to

pub unsafe fn vgetq_lane_u64(v: uint64x2_t, imm5: i32) -> u64 {
    if (imm5) < 0 || (imm5) > 1 {
        unreachable_unchecked()
    }
    let imm5 = (imm5 & 0b1) as u32;
    simd_extract(v, imm5)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
//#[rustc_args_required_const(1)]

// gcc also turns this into a fmov instead of a umove
// https://clang.godbolt.org/z/J5xS2T

pub unsafe fn vget_lane_u64(v: uint64x1_t, imm5: i32) -> u64 {
    if imm5 != 0 {
        unreachable_unchecked()
    }
    simd_extract(v, 0)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
//#[rustc_args_required_const(1)]

pub unsafe fn vgetq_lane_u16(v: uint16x8_t, imm5: i32) -> u16 {
    if (imm5) < 0 || (imm5) > 7 {
        unreachable_unchecked()
    }
    let imm5 = (imm5 & 0b111) as u32;
    simd_extract(v, imm5)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
//#[rustc_args_required_const(1)]
// see: https://clang.godbolt.org/z/J5xS2T

pub unsafe fn vgetq_lane_u32(v: uint32x4_t, imm5: i32) -> u32 {
    if (imm5) < 0 || (imm5) > 3 {
        unreachable_unchecked()
    }
    let imm5 = (imm5 & 0b11) as u32;
    simd_extract(v, imm5)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
//#[rustc_args_required_const(1)]

pub unsafe fn vget_lane_u8(v: uint8x8_t, imm5: i32) -> u8 {
    if (imm5) < 0 || (imm5) > 7 {
        unreachable_unchecked()
    }
    let imm5 = (imm5 & 7) as u32;
    simd_extract(v, imm5)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vdupq_n_s8(value: i8) -> int8x16_t {
    int8x16_t(
        value, value, value, value, value, value, value, value, value, value, value, value, value,
        value, value, value,
    )
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vdupq_n_u8(value: u8) -> uint8x16_t {
    uint8x16_t(
        value, value, value, value, value, value, value, value, value, value, value, value, value,
        value, value, value,
    )
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmovq_n_u8(value: u8) -> uint8x16_t {
    vdupq_n_u8(value)
}

/// Vector reinterpret cast operation
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vreinterpret_u64_u32(a: uint32x2_t) -> uint64x1_t {
    transmute(a)
}

/// Vector reinterpret cast operation
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vreinterpretq_s8_u8(a: uint8x16_t) -> int8x16_t {
    transmute(a)
}

/// Vector reinterpret cast operation
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vreinterpretq_u16_u8(a: uint8x16_t) -> uint16x8_t {
    transmute(a)
}

/// Vector reinterpret cast operation
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vreinterpretq_u32_u8(a: uint8x16_t) -> uint32x4_t {
    transmute(a)
}

/// Vector reinterpret cast operation
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vreinterpretq_u64_u8(a: uint8x16_t) -> uint64x2_t {
    transmute(a)
}

/// Vector reinterpret cast operation
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vreinterpretq_u8_s8(a: int8x16_t) -> uint8x16_t {
    transmute(a)
}

/// Unsigned shift right
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
//#[rustc_args_required_const(1)]
pub unsafe fn vshrq_n_u8(a: uint8x16_t, imm3: i32) -> uint8x16_t {
    if imm3 < 0 || imm3 > 7 {
        unreachable_unchecked();
    } else {
        uint8x16_t(
            a.0 >> imm3,
            a.1 >> imm3,
            a.2 >> imm3,
            a.3 >> imm3,
            a.4 >> imm3,
            a.5 >> imm3,
            a.6 >> imm3,
            a.7 >> imm3,
            a.8 >> imm3,
            a.9 >> imm3,
            a.10 >> imm3,
            a.11 >> imm3,
            a.12 >> imm3,
            a.13 >> imm3,
            a.14 >> imm3,
            a.15 >> imm3,
        )
    }
}

/// Shift right
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_args_required_const(1)]
pub unsafe fn vshlq_n_u8(a: uint8x16_t, imm3: i32) -> uint8x16_t {
    if imm3 < 0 || imm3 > 7 {
        unreachable_unchecked();
    } else {
        uint8x16_t(
            a.0 << imm3,
            a.1 << imm3,
            a.2 << imm3,
            a.3 << imm3,
            a.4 << imm3,
            a.5 << imm3,
            a.6 << imm3,
            a.7 << imm3,
            a.8 << imm3,
            a.9 << imm3,
            a.10 << imm3,
            a.11 << imm3,
            a.12 << imm3,
            a.13 << imm3,
            a.14 << imm3,
            a.15 << imm3,
        )
    }
}

/// Extract vector from pair of vectors
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
//#[rustc_args_required_const(2)]
pub unsafe fn vextq_s8(a: int8x16_t, b: int8x16_t, n: i32) -> int8x16_t {
    if n < 0 || n > 15 {
        unreachable_unchecked();
    };
    match n & 0b1111 {
        0 => simd_shuffle16(a, b, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
        1 => simd_shuffle16(
            a,
            b,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        ),
        2 => simd_shuffle16(
            a,
            b,
            [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
        ),
        3 => simd_shuffle16(
            a,
            b,
            [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18],
        ),
        4 => simd_shuffle16(
            a,
            b,
            [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
        ),
        5 => simd_shuffle16(
            a,
            b,
            [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
        ),
        6 => simd_shuffle16(
            a,
            b,
            [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21],
        ),
        7 => simd_shuffle16(
            a,
            b,
            [7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22],
        ),
        8 => simd_shuffle16(
            a,
            b,
            [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
        ),
        9 => simd_shuffle16(
            a,
            b,
            [
                9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            ],
        ),
        10 => simd_shuffle16(
            a,
            b,
            [
                10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
            ],
        ),
        11 => simd_shuffle16(
            a,
            b,
            [
                11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            ],
        ),
        12 => simd_shuffle16(
            a,
            b,
            [
                12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
            ],
        ),
        13 => simd_shuffle16(
            a,
            b,
            [
                13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
            ],
        ),
        14 => simd_shuffle16(
            a,
            b,
            [
                14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            ],
        ),
        15 => simd_shuffle16(
            a,
            b,
            [
                15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
            ],
        ),
        _ => unreachable_unchecked(),
    }
}

/// Extract vector from pair of vectors
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
//#[rustc_args_required_const(2)]
pub unsafe fn vextq_u8(a: uint8x16_t, b: uint8x16_t, n: i32) -> uint8x16_t {
    if n < 0 || n > 15 {
        unreachable_unchecked();
    };
    match n & 0b1111 {
        0 => simd_shuffle16(a, b, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
        1 => simd_shuffle16(
            a,
            b,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        ),
        2 => simd_shuffle16(
            a,
            b,
            [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
        ),
        3 => simd_shuffle16(
            a,
            b,
            [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18],
        ),
        4 => simd_shuffle16(
            a,
            b,
            [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
        ),
        5 => simd_shuffle16(
            a,
            b,
            [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
        ),
        6 => simd_shuffle16(
            a,
            b,
            [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21],
        ),
        7 => simd_shuffle16(
            a,
            b,
            [7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22],
        ),
        8 => simd_shuffle16(
            a,
            b,
            [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
        ),
        9 => simd_shuffle16(
            a,
            b,
            [
                9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            ],
        ),
        10 => simd_shuffle16(
            a,
            b,
            [
                10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
            ],
        ),
        11 => simd_shuffle16(
            a,
            b,
            [
                11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            ],
        ),
        12 => simd_shuffle16(
            a,
            b,
            [
                12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
            ],
        ),
        13 => simd_shuffle16(
            a,
            b,
            [
                13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
            ],
        ),
        14 => simd_shuffle16(
            a,
            b,
            [
                14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            ],
        ),
        15 => simd_shuffle16(
            a,
            b,
            [
                15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
            ],
        ),
        _ => unreachable_unchecked(),
    }
}

/// Load multiple single-element structures to one, two, three, or four registers
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

// even gcc compiles this to ldr: https://clang.godbolt.org/z/1bvH2x

pub unsafe fn vld1q_s8(addr: *const i8) -> int8x16_t {
    ptr::read(addr as *const int8x16_t)
}

/// Load multiple single-element structures to one, two, three, or four registers
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

// even gcc compiles this to ldr: https://clang.godbolt.org/z/1bvH2x

pub unsafe fn vld1q_u8(addr: *const u8) -> uint8x16_t {
    ptr::read(addr as *const uint8x16_t)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vand_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vandq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vand_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vandq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vand_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vandq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vand_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vandq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vand_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vandq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vand_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vandq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vand_s64(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vandq_s64(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vand_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_and(a, b)
}

/// Vector bitwise and
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vandq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_and(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorr_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorrq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorr_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorrq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorr_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorrq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorr_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorrq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorr_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorrq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorr_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorrq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorr_s64(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorrq_s64(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorr_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_or(a, b)
}

/// Vector bitwise or (immediate, inclusive)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vorrq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_or(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veor_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veorq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veor_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veorq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veor_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veorq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veor_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veorq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veor_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veorq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veor_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veorq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veor_s64(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veorq_s64(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veor_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_xor(a, b)
}

/// Vector bitwise exclusive or (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn veorq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_xor(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceq_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceqq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceq_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceqq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceq_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceqq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceq_s8(a: int8x8_t, b: int8x8_t) -> uint8x8_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceqq_s8(a: int8x16_t, b: int8x16_t) -> uint8x16_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceq_s16(a: int16x4_t, b: int16x4_t) -> uint16x4_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceqq_s16(a: int16x8_t, b: int16x8_t) -> uint16x8_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceq_s32(a: int32x2_t, b: int32x2_t) -> uint32x2_t {
    simd_eq(a, b)
}

/// Compare bitwise Equal (vector)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceqq_s32(a: int32x4_t, b: int32x4_t) -> uint32x4_t {
    simd_eq(a, b)
}

/// Floating-point compare equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceq_f32(a: float32x2_t, b: float32x2_t) -> uint32x2_t {
    simd_eq(a, b)
}

/// Floating-point compare equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vceqq_f32(a: float32x4_t, b: float32x4_t) -> uint32x4_t {
    simd_eq(a, b)
}

/// Compare signed greater than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgt_s8(a: int8x8_t, b: int8x8_t) -> uint8x8_t {
    simd_gt(a, b)
}

/// Compare signed greater than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgtq_s8(a: int8x16_t, b: int8x16_t) -> uint8x16_t {
    simd_gt(a, b)
}

/// Compare signed greater than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgt_s16(a: int16x4_t, b: int16x4_t) -> uint16x4_t {
    simd_gt(a, b)
}

/// Compare signed greater than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgtq_s16(a: int16x8_t, b: int16x8_t) -> uint16x8_t {
    simd_gt(a, b)
}

/// Compare signed greater than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgt_s32(a: int32x2_t, b: int32x2_t) -> uint32x2_t {
    simd_gt(a, b)
}

/// Compare signed greater than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgtq_s32(a: int32x4_t, b: int32x4_t) -> uint32x4_t {
    simd_gt(a, b)
}

/// Compare unsigned highe
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgt_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_gt(a, b)
}

/// Compare unsigned highe
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgtq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_gt(a, b)
}

/// Compare unsigned highe
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgt_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_gt(a, b)
}

/// Compare unsigned highe
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgtq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_gt(a, b)
}

/// Compare unsigned highe
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgt_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_gt(a, b)
}

/// Compare unsigned highe
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgtq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_gt(a, b)
}

/// Floating-point compare greater than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgt_f32(a: float32x2_t, b: float32x2_t) -> uint32x2_t {
    simd_gt(a, b)
}

/// Floating-point compare greater than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgtq_f32(a: float32x4_t, b: float32x4_t) -> uint32x4_t {
    simd_gt(a, b)
}

/// Compare signed less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vclt_s8(a: int8x8_t, b: int8x8_t) -> uint8x8_t {
    simd_lt(a, b)
}

/// Compare signed less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcltq_s8(a: int8x16_t, b: int8x16_t) -> uint8x16_t {
    simd_lt(a, b)
}

/// Compare signed less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vclt_s16(a: int16x4_t, b: int16x4_t) -> uint16x4_t {
    simd_lt(a, b)
}

/// Compare signed less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcltq_s16(a: int16x8_t, b: int16x8_t) -> uint16x8_t {
    simd_lt(a, b)
}

/// Compare signed less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vclt_s32(a: int32x2_t, b: int32x2_t) -> uint32x2_t {
    simd_lt(a, b)
}

/// Compare signed less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcltq_s32(a: int32x4_t, b: int32x4_t) -> uint32x4_t {
    simd_lt(a, b)
}

/// Compare unsigned less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vclt_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_lt(a, b)
}

/// Compare unsigned less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcltq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_lt(a, b)
}

/// Compare unsigned less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vclt_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_lt(a, b)
}

/// Compare unsigned less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcltq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_lt(a, b)
}

/// Compare unsigned less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vclt_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_lt(a, b)
}

/// Compare unsigned less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcltq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_lt(a, b)
}

/// Floating-point compare less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vclt_f32(a: float32x2_t, b: float32x2_t) -> uint32x2_t {
    simd_lt(a, b)
}

/// Floating-point compare less than
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcltq_f32(a: float32x4_t, b: float32x4_t) -> uint32x4_t {
    simd_lt(a, b)
}

/// Compare signed less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcle_s8(a: int8x8_t, b: int8x8_t) -> uint8x8_t {
    simd_le(a, b)
}

/// Compare signed less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcleq_s8(a: int8x16_t, b: int8x16_t) -> uint8x16_t {
    simd_le(a, b)
}

/// Compare signed less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcle_s16(a: int16x4_t, b: int16x4_t) -> uint16x4_t {
    simd_le(a, b)
}

/// Compare signed less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcleq_s16(a: int16x8_t, b: int16x8_t) -> uint16x8_t {
    simd_le(a, b)
}

/// Compare signed less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcle_s32(a: int32x2_t, b: int32x2_t) -> uint32x2_t {
    simd_le(a, b)
}

/// Compare signed less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcleq_s32(a: int32x4_t, b: int32x4_t) -> uint32x4_t {
    simd_le(a, b)
}

/// Compare unsigned less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcle_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_le(a, b)
}

/// Compare unsigned less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcleq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_le(a, b)
}

/// Compare unsigned less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcle_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_le(a, b)
}

/// Compare unsigned less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcleq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_le(a, b)
}

/// Compare unsigned less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcle_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_le(a, b)
}

/// Compare unsigned less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcleq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_le(a, b)
}

/// Floating-point compare less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcle_f32(a: float32x2_t, b: float32x2_t) -> uint32x2_t {
    simd_le(a, b)
}

/// Floating-point compare less than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcleq_f32(a: float32x4_t, b: float32x4_t) -> uint32x4_t {
    simd_le(a, b)
}

/// Compare signed greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcge_s8(a: int8x8_t, b: int8x8_t) -> uint8x8_t {
    simd_ge(a, b)
}

/// Compare signed greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgeq_s8(a: int8x16_t, b: int8x16_t) -> uint8x16_t {
    simd_ge(a, b)
}

/// Compare signed greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcge_s16(a: int16x4_t, b: int16x4_t) -> uint16x4_t {
    simd_ge(a, b)
}

/// Compare signed greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgeq_s16(a: int16x8_t, b: int16x8_t) -> uint16x8_t {
    simd_ge(a, b)
}

/// Compare signed greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcge_s32(a: int32x2_t, b: int32x2_t) -> uint32x2_t {
    simd_ge(a, b)
}

/// Compare signed greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgeq_s32(a: int32x4_t, b: int32x4_t) -> uint32x4_t {
    simd_ge(a, b)
}

/// Compare unsigned greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcge_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_ge(a, b)
}

/// Compare unsigned greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgeq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_ge(a, b)
}

/// Compare unsigned greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcge_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_ge(a, b)
}

/// Compare unsigned greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgeq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_ge(a, b)
}

/// Compare unsigned greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcge_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_ge(a, b)
}

/// Compare unsigned greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgeq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_ge(a, b)
}

/// Floating-point compare greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcge_f32(a: float32x2_t, b: float32x2_t) -> uint32x2_t {
    simd_ge(a, b)
}

/// Floating-point compare greater than or equal
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vcgeq_f32(a: float32x4_t, b: float32x4_t) -> uint32x4_t {
    simd_ge(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
pub unsafe fn vqsub_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubu.v8u8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqsub.v8i8")]
        fn vqsub_u8_(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t;
    }
    vqsub_u8_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
pub unsafe fn vqsubq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubu.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqsub.v16i8")]
        fn vqsubq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    }
    vqsubq_u8_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
pub unsafe fn vqsub_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubu.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqsub.v4i16")]
        fn vqsub_u16_(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t;
    }
    vqsub_u16_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
pub unsafe fn vqsubq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubu.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqsub.v8i16")]
        fn vqsubq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    }
    vqsubq_u16_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqsub_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubu.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqsub.v2i32")]
        fn vqsub_u32_(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t;
    }
    vqsub_u32_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqsubq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubu.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqsub.v4i32")]
        fn vqsubq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    }
    vqsubq_u32_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
pub unsafe fn vqsub_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubs.v8i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqsub.v8i8")]
        fn vqsub_s8_(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    }
    vqsub_s8_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
pub unsafe fn vqsubq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubs.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqsub.v16i8")]
        fn vqsubq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    }
    vqsubq_s8_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqsub_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
//        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubs.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqsub.v4i16")]
        fn vqsub_s16_(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    }
    vqsub_s16_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqsubq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubs.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqsub.v8i16")]
        fn vqsubq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    }
    vqsubq_s16_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqsub_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubs.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqsub.v2i32")]
        fn vqsub_s32_(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    }
    vqsub_s32_(a, b)
}

/// Saturating subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqsubq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqsubs.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqsub.v4i32")]
        fn vqsubq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    }
    vqsubq_s32_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhadd_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhaddu.v8i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhadd.v8i8")]
        fn vhadd_u8_(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t;
    }
    vhadd_u8_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhaddu.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhadd.v16i8")]
        fn vhaddq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    }
    vhaddq_u8_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhadd_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhaddu.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhadd.v4i16")]
        fn vhadd_u16_(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t;
    }
    vhadd_u16_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhaddq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhaddu.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhadd.v8i16")]
        fn vhaddq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    }
    vhaddq_u16_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhadd_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhaddu.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhadd.v2i32")]
        fn vhadd_u32_(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t;
    }
    vhadd_u32_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhaddq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhaddu.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhadd.v4i32")]
        fn vhaddq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    }
    vhaddq_u32_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhadd_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhadds.v8i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shadd.v8i8")]
        fn vhadd_s8_(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    }
    vhadd_s8_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhaddq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhadds.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shadd.v16i8")]
        fn vhaddq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    }
    vhaddq_s8_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhadd_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhadds.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shadd.v4i16")]
        fn vhadd_s16_(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    }
    vhadd_s16_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhaddq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhadds.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shadd.v8i16")]
        fn vhaddq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    }
    vhaddq_s16_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhadd_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhadds.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shadd.v2i32")]
        fn vhadd_s32_(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    }
    vhadd_s32_(a, b)
}

/// Halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhaddq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhadds.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shadd.v4i32")]
        fn vhaddq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    }
    vhaddq_s32_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhadd_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhaddu.v8i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.urhadd.v8i8")]
        fn vrhadd_u8_(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t;
    }
    vrhadd_u8_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhaddu.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.urhadd.v16i8")]
        fn vrhaddq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    }
    vrhaddq_u8_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhadd_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhaddu.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.urhadd.v4i16")]
        fn vrhadd_u16_(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t;
    }
    vrhadd_u16_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhaddq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhaddu.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.urhadd.v8i16")]
        fn vrhaddq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    }
    vrhaddq_u16_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhadd_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhaddu.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.urhadd.v2i32")]
        fn vrhadd_u32_(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t;
    }
    vrhadd_u32_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhaddq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhaddu.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.urhadd.v4i32")]
        fn vrhaddq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    }
    vrhaddq_u32_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhadd_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhadds.v8i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.srhadd.v8i8")]
        fn vrhadd_s8_(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    }
    vrhadd_s8_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhaddq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhadds.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.srhadd.v16i8")]
        fn vrhaddq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    }
    vrhaddq_s8_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhadd_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhadds.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.srhadd.v4i16")]
        fn vrhadd_s16_(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    }
    vrhadd_s16_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhaddq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhadds.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.srhadd.v8i16")]
        fn vrhaddq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    }
    vrhaddq_s16_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhadd_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhadds.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.srhadd.v2i32")]
        fn vrhadd_s32_(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    }
    vrhadd_s32_(a, b)
}

/// Rounding halving add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vrhaddq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vrhadds.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.srhadd.v4i32")]
        fn vrhaddq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    }
    vrhaddq_s32_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqadd_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqaddu.v8i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqadd.v8i8")]
        fn vqadd_u8_(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t;
    }
    vqadd_u8_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqaddu.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqadd.v16i8")]
        fn vqaddq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    }
    vqaddq_u8_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqadd_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqaddu.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqadd.v4i16")]
        fn vqadd_u16_(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t;
    }
    vqadd_u16_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqaddq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqaddu.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqadd.v8i16")]
        fn vqaddq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    }
    vqaddq_u16_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqadd_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqaddu.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqadd.v2i32")]
        fn vqadd_u32_(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t;
    }
    vqadd_u32_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqaddq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqaddu.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uqadd.v4i32")]
        fn vqaddq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    }
    vqaddq_u32_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqadd_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqadds.v8i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqadd.v8i8")]
        fn vqadd_s8_(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    }
    vqadd_s8_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqaddq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqadds.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqadd.v16i8")]
        fn vqaddq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    }
    vqaddq_s8_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqadd_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqadds.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqadd.v4i16")]
        fn vqadd_s16_(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    }
    vqadd_s16_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqaddq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqadds.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqadd.v8i16")]
        fn vqaddq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    }
    vqaddq_s16_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqadd_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqadds.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqadd.v2i32")]
        fn vqadd_s32_(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    }
    vqadd_s32_(a, b)
}

/// Saturating add
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vqaddq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vqadds.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sqadd.v4i32")]
        fn vqaddq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    }
    vqaddq_s32_(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmul_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmulq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmul_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmulq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmul_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmulq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmul_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmulq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmul_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmulq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmul_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmulq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmul_f32(a: float32x2_t, b: float32x2_t) -> float32x2_t {
    simd_mul(a, b)
}

/// Multiply
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vmulq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    simd_mul(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsub_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsubq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsub_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsubq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsub_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsubq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsub_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsubq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsub_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsubq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsub_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsubq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsub_s64(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsubq_s64(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsub_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsubq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsub_f32(a: float32x2_t, b: float32x2_t) -> float32x2_t {
    simd_sub(a, b)
}

/// Subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vsubq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    simd_sub(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsub_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubu.v8i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhsub.v8i8")]
        fn vhsub_u8_(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t;
    }
    vhsub_u8_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsubq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubu.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhsub.v16i8")]
        fn vhsubq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    }
    vhsubq_u8_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsub_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubu.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhsub.v4i16")]
        fn vhsub_u16_(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t;
    }
    vhsub_u16_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsubq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubu.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhsub.v8i16")]
        fn vhsubq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    }
    vhsubq_u16_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsub_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubu.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhsub.v2i32")]
        fn vhsub_u32_(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t;
    }
    vhsub_u32_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsubq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubu.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uhsub.v4i32")]
        fn vhsubq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    }
    vhsubq_u32_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsub_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubs.v8i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shsub.v8i8")]
        fn vhsub_s8_(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    }
    vhsub_s8_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsubq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubs.v16i8")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shsub.v16i8")]
        fn vhsubq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    }
    vhsubq_s8_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsub_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubs.v4i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shsub.v4i16")]
        fn vhsub_s16_(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    }
    vhsub_s16_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsubq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubs.v8i16")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shsub.v8i16")]
        fn vhsubq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    }
    vhsubq_s16_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsub_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubs.v2i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shsub.v2i32")]
        fn vhsub_s32_(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    }
    vhsub_s32_(a, b)
}

/// Signed halving subtract
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]

pub unsafe fn vhsubq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    #[allow(improper_ctypes)]
    extern "C" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vhsubs.v4i32")]
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.shsub.v4i32")]
        fn vhsubq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    }
    vhsubq_s32_(a, b)
}
