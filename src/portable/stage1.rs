use std::{
    ops::Shr,
    simd::{prelude::*, ToBitMask},
};

macro_rules! low_nibble_mask {
    () => {
        [
            16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1, 2, 9, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1,
            2, 9, 0, 0,
        ]
    };
}

macro_rules! high_nibble_mask {
    () => {
        [
            8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3, 2, 1, 0, 0, 8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3,
            2, 1, 0, 0,
        ]
    };
}

use crate::{static_cast_i32, Stage1Parse};
#[derive(Debug)]
pub(crate) struct SimdInputPortable {
    v0: u8x32,
    v1: u8x32,
}

impl Stage1Parse for SimdInputPortable {
    type Utf8Validator = simdutf8::basic::imp::portable::ChunkedUtf8ValidatorImp;
    type SimdRepresentation = u8x32;
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn new(ptr: &[u8]) -> Self {
        Self {
            v0: u8x32::from_array(*ptr.as_ptr().cast::<[u8; 32]>()),
            v1: u8x32::from_array(*ptr.as_ptr().add(32).cast::<[u8; 32]>()),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_sign_loss)]
    unsafe fn compute_quote_mask(quote_bits: u64) -> u64 {
        let mut quote_mask: u64 = quote_bits ^ (quote_bits << 1);
        quote_mask = quote_mask ^ (quote_mask << 2);
        quote_mask = quote_mask ^ (quote_mask << 4);
        quote_mask = quote_mask ^ (quote_mask << 8);
        quote_mask = quote_mask ^ (quote_mask << 16);
        quote_mask = quote_mask ^ (quote_mask << 32);
        quote_mask
    }

    /// a straightforward comparison of a mask against input
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    unsafe fn cmp_mask_against_input(&self, m: u8) -> u64 {
        let mask = u8x32::splat(m);
        let cmp_res_0 = self.v0.simd_eq(mask);
        let res_0 = u64::from(cmp_res_0.to_bitmask());
        let cmp_res_1 = self.v1.simd_eq(mask);
        let res_1 = u64::from(cmp_res_1.to_bitmask());
        res_0 | (res_1 << 32)
    }

    // find all values less than or equal than the content of maxval (using unsigned arithmetic)
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_sign_loss)]
    unsafe fn unsigned_lteq_against_input(&self, maxval: u8x32) -> u64 {
        let res_0: u64 = u64::from(self.v0.simd_le(maxval).to_bitmask());
        let res_1: u64 = u64::from(self.v1.simd_le(maxval).to_bitmask());
        res_0 | (res_1 << 32)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_sign_loss, clippy::cast_lossless)]
    unsafe fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64) {
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

        const LOW_NIBBLE_MASK: u8x32 = u8x32::from_array(low_nibble_mask!());
        const HIGH_NIBBLE_MASK: u8x32 = u8x32::from_array(high_nibble_mask!());

        let structural_shufti_mask: u8x32 = u8x32::splat(0b0000_0111); // 0x07
        let whitespace_shufti_mask: u8x32 = u8x32::splat(0b0001_1000); // 0x18

        // FIXME: do we need this dance?

        let v0_32 = i32x8::from_array(std::mem::transmute(*self.v0.as_array()));
        let v0_shifted: Simd<i32, 8> = v0_32.shr(i32x8::splat(4));
        let v0_shifted = u8x32::from_array(std::mem::transmute(v0_shifted));

        // We have to adjust the index here the reason being that while the avx instruction
        // only uses the lower 4 bits for index and the 8th bit for overflow (set to 0)
        // std::simd::swizzle uses all bits 5-8 for overflow, so we need to mask out
        // bit 6, 5 and 7 to get the correct behaviour
        let v0_idx_low = self.v0 & u8x32::splat(0b1000_1111);
        let v0_swizzle_low = LOW_NIBBLE_MASK.swizzle_dyn(v0_idx_low);
        let v0_idx_high = v0_shifted & u8x32::splat(0b0000_1111);
        let v0_swizzle_high = HIGH_NIBBLE_MASK.swizzle_dyn(v0_idx_high);
        let v_lo = v0_swizzle_low & v0_swizzle_high;

        let v1_32 = i32x8::from_array(std::mem::transmute(*self.v1.as_array()));
        let v1_shifted: Simd<i32, 8> = v1_32.shr(i32x8::splat(4));
        let v1_shifted = u8x32::from_array(std::mem::transmute(v1_shifted));

        // We have to adjust the index here the reason being that while the avx instruction
        // only uses the lower 4 bits for index and the 8th bit for overflow (set to 0)
        // std::simd::swizzle uses all bits 5-8 for overflow, so we need to mask out
        // bit 6, 5 and 7 to get the correct behaviour
        let v1_idx_low = self.v1 & u8x32::splat(0b1000_1111);
        let v1_swizzle_low = LOW_NIBBLE_MASK.swizzle_dyn(v1_idx_low);
        let v1_idx_high = v1_shifted & u8x32::splat(0b0000_1111);
        let v1_swizzle_high = HIGH_NIBBLE_MASK.swizzle_dyn(v1_idx_high);
        let v_hi = v1_swizzle_low & v1_swizzle_high;

        let tmp_lo = (v_lo & structural_shufti_mask).simd_eq(u8x32::splat(0));
        let tmp_hi = (v_hi & structural_shufti_mask).simd_eq(u8x32::splat(0));

        let structural_res_0 = u64::from(tmp_lo.to_bitmask());
        let structural_res_1 = u64::from(tmp_hi.to_bitmask());
        *structurals = !(structural_res_0 | (structural_res_1 << 32));

        let tmp_ws_lo = (v_lo & whitespace_shufti_mask).simd_eq(u8x32::splat(0));
        let tmp_ws_hi = (v_hi & whitespace_shufti_mask).simd_eq(u8x32::splat(0));

        let ws_res_0 = u64::from(tmp_ws_lo.to_bitmask());
        let ws_res_1 = u64::from(tmp_ws_hi.to_bitmask());

        *whitespace = !(ws_res_0 | (ws_res_1 << 32));
    }

    // flatten out values in 'bits' assuming that they are are to have values of idx
    // plus their position in the bitvector, and store these indexes at
    // base_ptr[base] incrementing base as we go
    // will potentially store extra values beyond end of valid bits, so base_ptr
    // needs to be large enough to handle this
    //TODO: usize was u32 here does this matter?
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_ptr_alignment,
        clippy::uninit_vec
    )]
    unsafe fn flatten_bits(base: &mut Vec<u32>, idx: u32, mut bits: u64) {
        let cnt: usize = bits.count_ones() as usize;
        let mut l = base.len();
        let idx_minus_64 = static_cast_i32!(idx.wrapping_sub(64));
        let idx_64_v: i32x8 = i32x8::from_array([
            idx_minus_64,
            idx_minus_64,
            idx_minus_64,
            idx_minus_64,
            idx_minus_64,
            idx_minus_64,
            idx_minus_64,
            idx_minus_64,
        ]);

        // We're doing some trickery here.
        // We reserve 64 extra entries, because we've at most 64 bit to set
        // then we trunctate the base to the next base (that we calcuate above)
        // We later indiscriminatory writre over the len we set but that's OK
        // since we ensure we reserve the needed space
        base.reserve(64);
        base.set_len(l + cnt);

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

            let v = i32x8::from_array([v0, v1, v2, v3, v4, v5, v6, v7]);
            let v = idx_64_v + v;
            base.as_mut_ptr()
                .add(l)
                .cast::<[i32; 8]>()
                .write(*v.as_array());
            //
            l += 8;
        }
    }

    #[allow(clippy::cast_sign_loss)]
    #[cfg_attr(not(feature = "no-inline"), inline)]
    unsafe fn fill_s8(n: i8) -> u8x32 {
        u8x32::splat(n as u8)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    unsafe fn zero() -> u8x32 {
        u8x32::splat(0)
    }
}
