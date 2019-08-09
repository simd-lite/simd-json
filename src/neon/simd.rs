#![allow(non_camel_case_types)]
#![allow(unused)]

use crate::neon::simd_llvm;

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
                    simd_llvm::simd_extract(self, index as u32)
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
                    simd_llvm::simd_extract(self, index as u32)
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

simd_ty!(u8x8[u8]:
         u8, u8, u8, u8, u8, u8, u8, u8
         | x0, x1, x2, x3, x4, x5, x6, x7);
simd_ty!(u16x4[u16]: u16, u16, u16, u16 | x0, x1, x2, x3);
simd_ty!(u32x2[u32]: u32, u32 | x0, x1);
simd_ty!(u64x1[u64]: u64 | x1);

simd_ty!(i8x8[i8]:
         i8, i8, i8, i8, i8, i8, i8, i8
         | x0, x1, x2, x3, x4, x5, x6, x7);
simd_ty!(i16x4[i16]: i16, i16, i16, i16 | x0, x1, x2, x3);
simd_ty!(i32x2[i32]: i32, i32 | x0, x1);
simd_ty!(i64x1[i64]: i64 | x1);

simd_ty!(f32x2[f32]: f32, f32 | x0, x1);

// 128-bit wide types:

simd_ty!(u8x16[u8]:
         u8, u8, u8, u8, u8, u8, u8, u8,
         u8, u8, u8, u8, u8, u8, u8, u8
         | x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15
);
simd_ty!(u16x8[u16]:
         u16, u16, u16, u16, u16, u16, u16, u16
         | x0, x1, x2, x3, x4, x5, x6, x7);
simd_ty!(u32x4[u32]: u32, u32, u32, u32 | x0, x1, x2, x3);
simd_ty!(u64x2[u64]: u64, u64 | x0, x1);

simd_ty!(i8x16[i8]:
         i8, i8, i8, i8, i8, i8, i8, i8,
         i8, i8, i8, i8, i8, i8, i8, i8
         | x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15
);
simd_ty!(i16x8[i16]:
         i16, i16, i16, i16, i16, i16, i16, i16
         | x0, x1, x2, x3, x4, x5, x6, x7);
simd_ty!(i32x4[i32]: i32, i32, i32, i32 | x0, x1, x2, x3);
simd_ty!(i64x2[i64]: i64, i64 | x0, x1);

simd_ty!(f32x4[f32]: f32, f32, f32, f32 | x0, x1, x2, x3);
simd_ty!(f64x2[f64]: f64, f64 | x0, x1);

simd_m_ty!(m8x16[i8]:
           i8, i8, i8, i8, i8, i8, i8, i8,
           i8, i8, i8, i8, i8, i8, i8, i8
           | x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15
);
simd_m_ty!(m16x8[i16]:
           i16, i16, i16, i16, i16, i16, i16, i16
           | x0, x1, x2, x3, x4, x5, x6, x7);
simd_m_ty!(m32x4[i32]: i32, i32, i32, i32 | x0, x1, x2, x3);
simd_m_ty!(m64x2[i64]: i64, i64 | x0, x1);

// 256-bit wide types:

simd_ty!(u8x32[u8]:
         u8, u8, u8, u8, u8, u8, u8, u8,
         u8, u8, u8, u8, u8, u8, u8, u8,
         u8, u8, u8, u8, u8, u8, u8, u8,
         u8, u8, u8, u8, u8, u8, u8, u8
         | x0, x1, x2, x3, x4, x5, x6, x7,
         x8, x9, x10, x11, x12, x13, x14, x15,
         x16, x17, x18, x19, x20, x21, x22, x23,
         x24, x25, x26, x27, x28, x29, x30, x31
);
simd_ty!(u16x16[u16]:
         u16, u16, u16, u16, u16, u16, u16, u16,
         u16, u16, u16, u16, u16, u16, u16, u16
         | x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15
);
simd_ty!(u32x8[u32]:
         u32, u32, u32, u32, u32, u32, u32, u32
         | x0, x1, x2, x3, x4, x5, x6, x7);
simd_ty!(u64x4[u64]: u64, u64, u64, u64 | x0, x1, x2, x3);

simd_ty!(i8x32[i8]:
         i8, i8, i8, i8, i8, i8, i8, i8,
         i8, i8, i8, i8, i8, i8, i8, i8,
         i8, i8, i8, i8, i8, i8, i8, i8,
         i8, i8, i8, i8, i8, i8, i8, i8
         | x0, x1, x2, x3, x4, x5, x6, x7,
         x8, x9, x10, x11, x12, x13, x14, x15,
         x16, x17, x18, x19, x20, x21, x22, x23,
         x24, x25, x26, x27, x28, x29, x30, x31
);
simd_ty!(i16x16[i16]:
         i16, i16, i16, i16, i16, i16, i16, i16,
         i16, i16, i16, i16, i16, i16, i16, i16
         | x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15
);
simd_ty!(i32x8[i32]:
         i32, i32, i32, i32, i32, i32, i32, i32
         | x0, x1, x2, x3, x4, x5, x6, x7);
simd_ty!(i64x4[i64]: i64, i64, i64, i64 | x0, x1, x2, x3);

simd_ty!(f32x8[f32]:
         f32, f32, f32, f32, f32, f32, f32, f32 |
         x0, x1, x2, x3, x4, x5, x6, x7);

// 512-bit wide types:

simd_ty!(i32x16[i32]:
         i32, i32, i32, i32, i32, i32, i32, i32,
         i32, i32, i32, i32, i32, i32, i32, i32
         | x0, x1, x2, x3, x4, x5, x6, x7,
         x8, x9, x10, x11, x12, x13, x14, x15);

simd_ty!(i64x8[i64]:
         i64, i64, i64, i64, i64, i64, i64, i64
         | x0, x1, x2, x3, x4, x5, x6, x7);

#[allow(unused)]
#[macro_export]
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