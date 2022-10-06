use crate::{static_cast_i32, static_cast_u32, Stage1Parse};
#[cfg(target_arch = "x86")]
use std::arch::x86 as arch;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64 as arch;

#[cfg(target_arch = "x86")]
use arch::{
    __m128i, _mm_add_epi32, _mm_and_si128, _mm_cmpeq_epi8, _mm_cmpgt_epi8, _mm_loadu_si128,
    _mm_max_epu8, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8, _mm_set_epi32, _mm_setr_epi8,
    _mm_setzero_si128, _mm_shuffle_epi8, _mm_srli_epi32, _mm_storeu_si128, _mm_testz_si128,
};

#[cfg(target_arch = "x86_64")]
use arch::{
    __m128i, _mm_add_epi32, _mm_and_si128, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_max_epu8,
    _mm_movemask_epi8, _mm_set1_epi8, _mm_set_epi32, _mm_setr_epi8, _mm_setzero_si128,
    _mm_shuffle_epi8, _mm_srli_epi32, _mm_storeu_si128,
};

use std::mem;

macro_rules! low_nibble_mask {
    () => {
        _mm_setr_epi8(16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1, 2, 9, 0, 0)
    };
}

macro_rules! high_nibble_mask {
    () => {
        _mm_setr_epi8(8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3, 2, 1, 0, 0)
    };
}

pub const SIMDJSON_PADDING: usize = mem::size_of::<__m128i>() * 2;
pub const SIMDINPUT_LENGTH: usize = 64;

#[derive(Debug)]
pub(crate) struct SimdInput {
    v0: __m128i,
    v1: __m128i,
    v2: __m128i,
    v3: __m128i,
}

impl SimdInput {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_ptr_alignment)]
    pub(crate) fn new(ptr: &[u8]) -> Self {
        unsafe {
            Self {
                v0: _mm_loadu_si128(ptr.as_ptr().cast::<arch::__m128i>()),
                v1: _mm_loadu_si128(ptr.as_ptr().add(16).cast::<arch::__m128i>()),
                v2: _mm_loadu_si128(ptr.as_ptr().add(32).cast::<arch::__m128i>()),
                v3: _mm_loadu_si128(ptr.as_ptr().add(48).cast::<arch::__m128i>()),
            }
        }
    }
}

impl Stage1Parse<__m128i> for SimdInput {
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[cfg(target_feature = "pclmulqdq")]
    #[allow(clippy::cast_sign_loss)]
    fn compute_quote_mask(quote_bits: u64) -> u64 {
        use arch::{_mm_clmulepi64_si128, _mm_cvtsi128_si64, _mm_set_epi64x};

        unsafe {
            _mm_cvtsi128_si64(_mm_clmulepi64_si128(
                _mm_set_epi64x(0, static_cast_i64!(quote_bits)),
                _mm_set1_epi8(-1_i8 /* 0xFF */),
                0,
            )) as u64
        }
    }

    #[cfg(not(target_feature = "pclmulqdq"))]
    fn compute_quote_mask(mut quote_bits: u64) -> u64 {
        #[allow(clippy::cast_sign_loss)]
        let b = -1_i64 as u64;
        let mut prod = 0;

        while quote_bits != 0 {
            prod ^= b.wrapping_mul(quote_bits & 0_u64.wrapping_sub(quote_bits));
            quote_bits &= quote_bits.wrapping_sub(1);
        }

        prod
    }

    /// a straightforward comparison of a mask against input
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn cmp_mask_against_input(&self, m: u8) -> u64 {
        unsafe {
            let mask: __m128i = _mm_set1_epi8(m as i8);
            let cmp_res_0: __m128i = _mm_cmpeq_epi8(self.v0, mask);
            let res_0: u64 = u64::from(static_cast_u32!(_mm_movemask_epi8(cmp_res_0)));
            let cmp_res_1: __m128i = _mm_cmpeq_epi8(self.v1, mask);
            let res_1: u64 = _mm_movemask_epi8(cmp_res_1) as u64;
            let cmp_res_2: __m128i = _mm_cmpeq_epi8(self.v2, mask);
            let res_2: u64 = _mm_movemask_epi8(cmp_res_2) as u64;
            let cmp_res_3: __m128i = _mm_cmpeq_epi8(self.v3, mask);
            let res_3: u64 = _mm_movemask_epi8(cmp_res_3) as u64;
            res_0 | (res_1 << 16) | (res_2 << 32) | (res_3 << 48)
        }
    }

    // find all values less than or equal than the content of maxval (using unsigned arithmetic)
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn unsigned_lteq_against_input(&self, maxval: __m128i) -> u64 {
        unsafe {
            let cmp_res_0: __m128i = _mm_cmpeq_epi8(_mm_max_epu8(maxval, self.v0), maxval);
            let res_0: u64 = u64::from(static_cast_u32!(_mm_movemask_epi8(cmp_res_0)));
            let cmp_res_1: __m128i = _mm_cmpeq_epi8(_mm_max_epu8(maxval, self.v1), maxval);
            let res_1: u64 = _mm_movemask_epi8(cmp_res_1) as u64;
            let cmp_res_2: __m128i = _mm_cmpeq_epi8(_mm_max_epu8(maxval, self.v2), maxval);
            let res_2: u64 = _mm_movemask_epi8(cmp_res_2) as u64;
            let cmp_res_3: __m128i = _mm_cmpeq_epi8(_mm_max_epu8(maxval, self.v3), maxval);
            let res_3: u64 = _mm_movemask_epi8(cmp_res_3) as u64;
            res_0 | (res_1 << 16) | (res_2 << 32) | (res_3 << 48)
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64) {
        unsafe {
            // do a 'shufti' to detect structural JSON characters
            // they are
            // * `{` 0x7b
            // * `}` 0x7d
            // * `:` 0x3a
            // * `[` 0x5b
            // * `]` 0x5d
            // * `,` 0x2c
            // these go into the first 3 buckets of the comparison (1/2/4)

            // we are also interested in the four whitespace characters:
            // * space 0x20
            // * linefeed 0x0a
            // * horizontal tab 0x09
            // * carriage return 0x0d
            // these go into the next 2 buckets of the comparison (8/16)

            let low_nibble_mask: __m128i = low_nibble_mask!();
            let high_nibble_mask: __m128i = high_nibble_mask!();

            let structural_shufti_mask: __m128i = _mm_set1_epi8(0x7);
            let whitespace_shufti_mask: __m128i = _mm_set1_epi8(0x18);

            let v_v0: __m128i = _mm_and_si128(
                _mm_shuffle_epi8(low_nibble_mask, self.v0),
                _mm_shuffle_epi8(
                    high_nibble_mask,
                    _mm_and_si128(_mm_srli_epi32(self.v0, 4), _mm_set1_epi8(0x7f)),
                ),
            );
            let v_v1: __m128i = _mm_and_si128(
                _mm_shuffle_epi8(low_nibble_mask, self.v1),
                _mm_shuffle_epi8(
                    high_nibble_mask,
                    _mm_and_si128(_mm_srli_epi32(self.v1, 4), _mm_set1_epi8(0x7f)),
                ),
            );
            let v_v2: __m128i = _mm_and_si128(
                _mm_shuffle_epi8(low_nibble_mask, self.v2),
                _mm_shuffle_epi8(
                    high_nibble_mask,
                    _mm_and_si128(_mm_srli_epi32(self.v2, 4), _mm_set1_epi8(0x7f)),
                ),
            );
            let v_v3: __m128i = _mm_and_si128(
                _mm_shuffle_epi8(low_nibble_mask, self.v3),
                _mm_shuffle_epi8(
                    high_nibble_mask,
                    _mm_and_si128(_mm_srli_epi32(self.v3, 4), _mm_set1_epi8(0x7f)),
                ),
            );
            let tmp_v0: __m128i = _mm_cmpeq_epi8(
                _mm_and_si128(v_v0, structural_shufti_mask),
                _mm_set1_epi8(0),
            );
            let tmp_v1: __m128i = _mm_cmpeq_epi8(
                _mm_and_si128(v_v1, structural_shufti_mask),
                _mm_set1_epi8(0),
            );
            let tmp_v2: __m128i = _mm_cmpeq_epi8(
                _mm_and_si128(v_v2, structural_shufti_mask),
                _mm_set1_epi8(0),
            );
            let tmp_v3: __m128i = _mm_cmpeq_epi8(
                _mm_and_si128(v_v3, structural_shufti_mask),
                _mm_set1_epi8(0),
            );

            let structural_res_0: u64 = u64::from(static_cast_u32!(_mm_movemask_epi8(tmp_v0)));
            let structural_res_1: u64 = _mm_movemask_epi8(tmp_v1) as u64;
            let structural_res_2: u64 = _mm_movemask_epi8(tmp_v2) as u64;
            let structural_res_3: u64 = _mm_movemask_epi8(tmp_v3) as u64;

            *structurals = !(structural_res_0
                | (structural_res_1 << 16)
                | (structural_res_2 << 32)
                | (structural_res_3 << 48));

            let tmp_ws_v0: __m128i = _mm_cmpeq_epi8(
                _mm_and_si128(v_v0, whitespace_shufti_mask),
                _mm_set1_epi8(0),
            );
            let tmp_ws_v1: __m128i = _mm_cmpeq_epi8(
                _mm_and_si128(v_v1, whitespace_shufti_mask),
                _mm_set1_epi8(0),
            );
            let tmp_ws_v2: __m128i = _mm_cmpeq_epi8(
                _mm_and_si128(v_v2, whitespace_shufti_mask),
                _mm_set1_epi8(0),
            );
            let tmp_ws_v3: __m128i = _mm_cmpeq_epi8(
                _mm_and_si128(v_v3, whitespace_shufti_mask),
                _mm_set1_epi8(0),
            );

            let ws_res_0: u64 = u64::from(static_cast_u32!(_mm_movemask_epi8(tmp_ws_v0)));
            let ws_res_1: u64 = _mm_movemask_epi8(tmp_ws_v1) as u64;
            let ws_res_2: u64 = _mm_movemask_epi8(tmp_ws_v2) as u64;
            let ws_res_3: u64 = _mm_movemask_epi8(tmp_ws_v3) as u64;
            *whitespace = !(ws_res_0 | (ws_res_1 << 16) | (ws_res_2 << 32) | (ws_res_3 << 48));
        }
    }

    // flatten out values in 'bits' assuming that they are are to have values of idx
    // plus their position in the bitvector, and store these indexes at
    // base_ptr[base] incrementing base as we go
    // will potentially store extra values beyond end of valid bits, so base_ptr
    // needs to be large enough to handle this
    //TODO: usize was u32 here does this matter?
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_ptr_alignment,
        clippy::uninit_vec
    )]
    fn flatten_bits(base: &mut Vec<u32>, idx: u32, mut bits: u64) {
        let cnt: usize = bits.count_ones() as usize;
        let mut l = base.len();
        let idx_minus_64 = idx.wrapping_sub(64);
        let idx_64_v = unsafe {
            _mm_set_epi32(
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
            )
        };

        // We're doing some trickery here.
        // We reserve 64 extra entries, because we've at most 64 bit to set
        // then we trunctate the base to the next base (that we calcuate above)
        // We later indiscriminatory writre over the len we set but that's OK
        // since we ensure we reserve the needed space
        base.reserve(64);
        unsafe {
            base.set_len(l + cnt);
        }

        while bits != 0 {
            unsafe {
                let v0 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v1 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v2 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v3 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);

                let v: __m128i = _mm_set_epi32(v3, v2, v1, v0);
                let v: __m128i = _mm_add_epi32(idx_64_v, v);
                _mm_storeu_si128(base.as_mut_ptr().add(l).cast::<arch::__m128i>(), v);
            }
            l += 4;
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn fill_s8(n: i8) -> __m128i {
        unsafe { _mm_set1_epi8(n) }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn zero() -> __m128i {
        unsafe { _mm_setzero_si128() }
    }
}
