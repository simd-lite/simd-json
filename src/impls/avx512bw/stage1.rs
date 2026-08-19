#![allow(dead_code)]
use crate::{
    Stage1Parse,
    macros::{static_cast_i32, static_cast_i64, static_cast_u32},
};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m128i, __m512i, __mmask64, _mm_setr_epi8, _mm512_broadcast_i32x4, _mm512_cmpeq_epi8_mask,
    _mm512_cmpeq_epu8_mask, _mm512_cmpge_epi8_mask, _mm512_cmpge_epu8_mask, _mm512_loadu_si512,
    _mm512_set1_epi8, _mm512_srli_epi32, _mm512_storeu_si512,
};
use std::arch::x86_64::{
    __m256i, _MM_CMPINT_NE, _mm_loadu_si128, _mm256_add_epi32, _mm256_set_epi32,
    _mm256_storeu_si256, _mm512_add_epi32, _mm512_and_si512, _mm512_cmp_epi8_mask,
    _mm512_cvtepu8_epi32, _mm512_maskz_compress_epi8, _mm512_set_epi32, _mm512_shuffle_epi8,
};

macro_rules! low_nibble_mask128 {
    () => {
        _mm_setr_epi8(16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1, 2, 9, 0, 0)
    };
}

macro_rules! high_nibble_mask128 {
    () => {
        _mm_setr_epi8(8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3, 2, 1, 0, 0)
    };
}

#[derive(Debug)]
pub(crate) struct SimdInput {
    v0: __m512i,
}

impl Stage1Parse for SimdInput {
    type Utf8Validator = simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp;
    type SimdRepresentation = __m512i;
    #[cfg_attr(not(feature = "no-inline"), inline)]
    // _mm512_loadu_si512 does not need alignment
    #[allow(clippy::cast_ptr_alignment)]
    #[target_feature(enable = "avx512bw")]
    unsafe fn new(ptr: &[u8]) -> Self {
        unsafe {
            Self {
                v0: _mm512_loadu_si512(ptr.as_ptr().cast::<__m512i>()),
            }
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_sign_loss)]
    #[target_feature(enable = "avx512bw", enable = "pclmulqdq")]
    #[cfg(target_arch = "x86_64")]
    unsafe fn compute_quote_mask(quote_bits: u64) -> u64 {
        unsafe {
            use std::arch::x86_64::{_mm_clmulepi64_si128, _mm_set_epi64x, _mm_set1_epi8};

            std::arch::x86_64::_mm_cvtsi128_si64(_mm_clmulepi64_si128(
                _mm_set_epi64x(0, static_cast_i64!(quote_bits)),
                _mm_set1_epi8(-1_i8 /* 0xFF */),
                0,
            )) as u64
        }
    }

    /// a straightforward comparison of a mask against input
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    #[target_feature(enable = "avx512bw")]
    unsafe fn cmp_mask_against_input(&self, m: u8) -> u64 {
        unsafe {
            let mask: __m512i = Self::fill_s8(m as i8);
            let res_0: __mmask64 = _mm512_cmpeq_epi8_mask(self.v0, mask);
            res_0 as u64
        }
    }

    // find all values less than or equal than the content of maxval (using unsigned arithmetic)
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_sign_loss)]
    #[target_feature(enable = "avx512bw")]
    unsafe fn unsigned_lteq_against_input(&self, maxval: __m512i) -> u64 {
        unsafe { _mm512_cmpge_epu8_mask(maxval, self.v0) }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_sign_loss)]
    #[target_feature(enable = "avx512bw")]
    unsafe fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64) {
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

            let low_nibble_mask: __m512i = _mm512_broadcast_i32x4(low_nibble_mask128!());
            let high_nibble_mask: __m512i = _mm512_broadcast_i32x4(high_nibble_mask128!());

            let structural_shufti_mask: __m512i = Self::fill_s8(0x7);
            let whitespace_shufti_mask: __m512i = Self::fill_s8(0x18);

            let msb_mak = Self::fill_s8(0x7f);
            let filtered_lower_4 = _mm512_and_si512(self.v0, msb_mak);
            let result_lower_4 = _mm512_shuffle_epi8(low_nibble_mask, filtered_lower_4);
            let filtered_high_4 = _mm512_and_si512(_mm512_srli_epi32::<4>(self.v0), msb_mak);
            let result_high_4 = _mm512_shuffle_epi8(high_nibble_mask, filtered_high_4);

            let result = _mm512_and_si512(result_high_4, result_lower_4);

            *structurals = _mm512_cmp_epi8_mask(
                _mm512_and_si512(result, structural_shufti_mask),
                Self::fill_s8(0),
                _MM_CMPINT_NE, // Not Equal to zero means it's a structural
            );

            *whitespace = _mm512_cmp_epi8_mask(
                _mm512_and_si512(result, whitespace_shufti_mask),
                Self::fill_s8(0),
                _MM_CMPINT_NE, // Not Equal to zero means it's whitespace
            );
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_possible_wrap, clippy::cast_ptr_alignment)]
    #[target_feature(enable = "avx512bw")]
    unsafe fn flatten_bits(base: &mut Vec<u32>, idx: u32, bits: u64) {
        unsafe {
            if std::is_x86_feature_detected!("avx512vbmi2") {
                FlattenBits::flatten_bits_fast(base, idx, bits);
            } else {
                FlattenBits::flatten_bits_slow(base, idx, bits);
            }
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[target_feature(enable = "avx512bw")]
    unsafe fn fill_s8(n: i8) -> __m512i {
        unsafe { _mm512_set1_epi8(n) }
    }
}

struct FlattenBits;

impl FlattenBits {
    // flatten out values in 'bits' assuming that they are are to have values of idx
    // plus their position in the bitvector, and store these indexes at
    // base_ptr[base] incrementing base as we go
    // will potentially store extra values beyond end of valid bits, so base_ptr
    // needs to be large enough to handle this
    //TODO: usize was u32 here does this matter?
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_possible_wrap, clippy::cast_ptr_alignment)]
    #[target_feature(enable = "avx512bw")]
    unsafe fn flatten_bits_slow(base: &mut Vec<u32>, idx: u32, mut bits: u64) {
        unsafe {
            let cnt: usize = bits.count_ones() as usize;
            let mut l: usize = base.len();
            let idx_minus_64 = idx.wrapping_sub(64);
            let idx_64_v = _mm512_set_epi32(
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
            );

            // We're doing some trickery here.
            // We reserve 64 extra entries, because we've at most 64 bit to set
            // then we truncate the base to the next base (that we calculated above)
            // We later indiscriminatory write over the len we set but that's OK
            // since we ensure we reserve the needed space
            base.reserve(64);
            let final_len = l + cnt;

            while bits != 0 {
                let v0 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v1 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v2 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v3 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v4 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v5 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v6 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v7 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v8 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v9 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v10 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v11 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v12 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v13 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v14 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);
                let v15 = bits.trailing_zeros() as i32;
                bits &= bits.wrapping_sub(1);

                let v: __m512i = _mm512_set_epi32(
                    v15, v14, v13, v12, v11, v10, v9, v8, v7, v6, v5, v4, v3, v2, v1, v0,
                );
                let v: __m512i = _mm512_add_epi32(idx_64_v, v);
                _mm512_storeu_si512(base.as_mut_ptr().add(l).cast::<__m512i>(), v);
                l += 16;
            }
            // We have written all the data
            base.set_len(final_len);
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_possible_wrap, clippy::cast_ptr_alignment)]
    #[target_feature(enable = "avx512vbmi2")]
    unsafe fn flatten_bits_fast(base: &mut Vec<u32>, idx: u32, bits: u64) {
        const INDICES: [u8; 64] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
        ];

        unsafe {
            let cnt: usize = bits.count_ones() as usize;
            let l: usize = base.len();
            let idx_minus_64 = idx.wrapping_sub(64);
            let idx_64_v = _mm512_set_epi32(
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
                static_cast_i32!(idx_minus_64),
            );
            base.reserve(64);
            let base_indices = _mm512_loadu_si512(INDICES.as_ptr().cast());
            let result_vec_u8 = _mm512_maskz_compress_epi8(bits, base_indices);

            let mut tmp = [0u8; 64];
            _mm512_storeu_si512(tmp.as_mut_ptr().cast(), result_vec_u8);

            for i in 0..4 {
                let chunk = _mm_loadu_si128(tmp.as_ptr().add(i * 16).cast());
                let widened = _mm512_cvtepu8_epi32(chunk);
                let v: __m512i = _mm512_add_epi32(idx_64_v, widened);
                _mm512_storeu_si512(base.as_mut_ptr().add(l + i * 16).cast(), v);
            }

            base.set_len(cnt + l);
        }
    }
}
