use std::arch::wasm32::*;
use std::mem;

use crate::Stage1Parse;

pub const SIMDJSON_PADDING: usize = mem::size_of::<v128>() * 2;
pub const SIMDINPUT_LENGTH: usize = 64;

#[derive(Debug)]
pub(crate) struct SimdInput {
    v0: v128,
    v1: v128,
    v2: v128,
    v3: v128,
}

impl SimdInput {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_ptr_alignment)]
    pub(crate) fn new(ptr: &[u8]) -> Self {
        unsafe {
            Self {
                v0: v128_load(ptr.as_ptr().cast::<v128>()),
                v1: v128_load(ptr.as_ptr().add(16).cast::<v128>()),
                v2: v128_load(ptr.as_ptr().add(32).cast::<v128>()),
                v3: v128_load(ptr.as_ptr().add(48).cast::<v128>()),
            }
        }
    }
}

impl Stage1Parse<v128> for SimdInput {
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
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
    fn cmp_mask_against_input(&self, m: u8) -> u64 {
        let mask = u8x16_splat(m);
        let cmp_res_0 = u8x16_eq(self.v0, mask);
        let res_0 = u8x16_bitmask(cmp_res_0) as u64;
        let cmp_res_1 = u8x16_eq(self.v1, mask);
        let res_1 = u8x16_bitmask(cmp_res_1) as u64;
        let cmp_res_2 = u8x16_eq(self.v2, mask);
        let res_2 = u8x16_bitmask(cmp_res_2) as u64;
        let cmp_res_3 = u8x16_eq(self.v3, mask);
        let res_3 = u8x16_bitmask(cmp_res_3) as u64;
        res_0 | (res_1 << 16) | (res_2 << 32) | (res_3 << 48)
    }

    // find all values less than or equal than the content of maxval (using unsigned arithmetic)
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn unsigned_lteq_against_input(&self, maxval: v128) -> u64 {
        let cmp_res_0 = u8x16_le(self.v0, maxval);
        let res_0 = u8x16_bitmask(cmp_res_0) as u64;
        let cmp_res_1 = u8x16_le(self.v1, maxval);
        let res_1 = u8x16_bitmask(cmp_res_1) as u64;
        let cmp_res_2 = u8x16_le(self.v2, maxval);
        let res_2 = u8x16_bitmask(cmp_res_2) as u64;
        let cmp_res_3 = u8x16_le(self.v3, maxval);
        let res_3 = u8x16_bitmask(cmp_res_3) as u64;
        res_0 | (res_1 << 16) | (res_2 << 32) | (res_3 << 48)
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64) {
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
        const LOW_NIBBLE_MASK: v128 = u8x16(16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1, 2, 9, 0, 0);
        const HIGH_NIBBLE_MASK: v128 = u8x16(8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3, 2, 1, 0, 0);

        let structural_shufti_mask = u8x16_splat(0x7);
        let whitespace_shufti_mask = u8x16_splat(0x18);
        let low_nib_and_mask = u8x16_splat(0xf);
        let high_nib_and_mask = u8x16_splat(0x7f);
        let zero_mask = u8x16_splat(0);

        let v_v0 = v128_and(
            u8x16_swizzle(LOW_NIBBLE_MASK, v128_and(self.v0, low_nib_and_mask)),
            u8x16_swizzle(
                HIGH_NIBBLE_MASK,
                v128_and(u8x16_shr(self.v0, 4), high_nib_and_mask),
            ),
        );
        let v_v1 = v128_and(
            u8x16_swizzle(LOW_NIBBLE_MASK, v128_and(self.v1, low_nib_and_mask)),
            u8x16_swizzle(
                HIGH_NIBBLE_MASK,
                v128_and(u8x16_shr(self.v1, 4), high_nib_and_mask),
            ),
        );
        let v_v2 = v128_and(
            u8x16_swizzle(LOW_NIBBLE_MASK, v128_and(self.v2, low_nib_and_mask)),
            u8x16_swizzle(
                HIGH_NIBBLE_MASK,
                v128_and(u8x16_shr(self.v2, 4), high_nib_and_mask),
            ),
        );
        let v_v3 = v128_and(
            u8x16_swizzle(LOW_NIBBLE_MASK, v128_and(self.v3, low_nib_and_mask)),
            u8x16_swizzle(
                HIGH_NIBBLE_MASK,
                v128_and(u8x16_shr(self.v3, 4), high_nib_and_mask),
            ),
        );
        let tmp_v0 = u8x16_eq(v128_and(v_v0, structural_shufti_mask), zero_mask);
        let tmp_v1 = u8x16_eq(v128_and(v_v1, structural_shufti_mask), zero_mask);
        let tmp_v2 = u8x16_eq(v128_and(v_v2, structural_shufti_mask), zero_mask);
        let tmp_v3 = u8x16_eq(v128_and(v_v3, structural_shufti_mask), zero_mask);

        let structural_res_0 = u8x16_bitmask(tmp_v0) as u64;
        let structural_res_1 = u8x16_bitmask(tmp_v1) as u64;
        let structural_res_2 = u8x16_bitmask(tmp_v2) as u64;
        let structural_res_3 = u8x16_bitmask(tmp_v3) as u64;

        *structurals = !(structural_res_0
            | (structural_res_1 << 16)
            | (structural_res_2 << 32)
            | (structural_res_3 << 48));

        let tmp_ws_v0 = u8x16_eq(v128_and(v_v0, whitespace_shufti_mask), zero_mask);
        let tmp_ws_v1 = u8x16_eq(v128_and(v_v1, whitespace_shufti_mask), zero_mask);
        let tmp_ws_v2 = u8x16_eq(v128_and(v_v2, whitespace_shufti_mask), zero_mask);
        let tmp_ws_v3 = u8x16_eq(v128_and(v_v3, whitespace_shufti_mask), zero_mask);

        let ws_res_0 = u8x16_bitmask(tmp_ws_v0) as u64;
        let ws_res_1 = u8x16_bitmask(tmp_ws_v1) as u64;
        let ws_res_2 = u8x16_bitmask(tmp_ws_v2) as u64;
        let ws_res_3 = u8x16_bitmask(tmp_ws_v3) as u64;

        *whitespace = !(ws_res_0 | (ws_res_1 << 16) | (ws_res_2 << 32) | (ws_res_3 << 48));
    }

    // flatten out values in 'bits' assuming that they are are to have values of idx
    // plus their position in the bitvector, and store these indexes at
    // base_ptr[base] incrementing base as we go
    // will potentially store extra values beyond end of valid bits, so base_ptr
    // needs to be large enough to handle this
    //TODO: usize was u32 here does this matter?
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn flatten_bits(base: &mut Vec<u32>, idx: u32, mut bits: u64) {
        let cnt: usize = bits.count_ones() as usize;
        let mut l = base.len();
        let idx_minus_64 = idx.wrapping_sub(64);
        let idx_64_v = u32x4_splat(idx_minus_64);

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
            let v0 = bits.trailing_zeros() as u32;
            bits &= bits.wrapping_sub(1);
            let v1 = bits.trailing_zeros() as u32;
            bits &= bits.wrapping_sub(1);
            let v2 = bits.trailing_zeros() as u32;
            bits &= bits.wrapping_sub(1);
            let v3 = bits.trailing_zeros() as u32;
            bits &= bits.wrapping_sub(1);

            let v = u32x4(v0, v1, v2, v3);
            let v = u32x4_add(idx_64_v, v);
            unsafe {
                v128_store(base.as_mut_ptr().add(l).cast::<v128>(), v);
            }
            l += 4;
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn fill_s8(n: i8) -> v128 {
        i8x16_splat(n)
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn zero() -> v128 {
        i8x16_splat(0)
    }
}
