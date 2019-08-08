#![allow(dead_code)]

//use crate::portability::*;
use crate::neon::intrinsics::*;
//use core::arch::aarch64::*;
//use crate::neon::utf8check::*;
use crate::*;
use std::mem;

#[macro_use]
use crate::neon::simd::*;

pub const SIMDJSON_PADDING: usize = mem::size_of::<uint8x16_t>() * 4;

#[derive(Debug)]
struct SimdInput {
    v0: uint8x16_t,
    v1: uint8x16_t,
    v2: uint8x16_t,
    v3: uint8x16_t,
}

fn fill_input(ptr: &[u8]) -> SimdInput {
    unsafe {
        #[allow(clippy::cast_ptr_alignment)]
            SimdInput {
            v0: mem::transmute(ptr.as_ptr() as *const uint8x16_t),
            v1: mem::transmute(ptr.as_ptr().add(16) as *const uint8x16_t),
            v2: mem::transmute(ptr.as_ptr().add(32) as *const uint8x16_t),
            v3: mem::transmute(ptr.as_ptr().add(48) as *const uint8x16_t),
        }
    }
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
unsafe fn neon_movemask(input: uint8x16_t) -> u16 {
    const bit_mask: uint8x16_t = uint8x16_t::new(0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                                                 0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80);

    let minput: uint8x16_t = vandq_u8(input, bit_mask);
    let tmp: uint8x16_t = vpaddq_u8(minput, minput);
    let tmp = vpaddq_u8(tmp, tmp);
    let tmp = vpaddq_u8(tmp, tmp);

    unsafe {
        vgetq_lane_u16(vreinterpretq_u16_u8(tmp), 0)
    }
}


#[cfg_attr(not(feature = "no-inline"), inline(always))]
unsafe fn neon_movemask_bulk(p0: uint8x16_t, p1: uint8x16_t, p2: uint8x16_t, p3: uint8x16_t) -> u64 {
    const bit_mask: uint8x16_t = uint8x16_t::new(0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                                                 0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80);
    let t0 = vandq_u8(p0, bit_mask);
    let t1 = vandq_u8(p1, bit_mask);
    let t2 = vandq_u8(p2, bit_mask);
    let t3 = vandq_u8(p3, bit_mask);
    let sum0 = vpaddq_u8(t0, t1);
    let sum1 = vpaddq_u8(t2, t3);
    let sum0 = vpaddq_u8(sum0, sum1);
    let sum0 = vpaddq_u8(sum0, sum0);
    unsafe {
        vgetq_lane_u64(vreinterpretq_u64_u8(sum0), 0)
    }
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
unsafe fn check_ascii_neon(si: &SimdInput) -> bool {
    let high_bit: uint8x16_t = vdupq_n_u8(0x80);
    let t0: uint8x16_t = vorrq_u8(si.v0, si.v1);
    let t1: uint8x16_t = vorrq_u8(si.v2, si.v3);
    let t3: uint8x16_t = vorrq_u8(t0, t1);
    let t4: uint8x16_t = vandq_u8(t3, high_bit);

    unsafe {
        let v64: uint64x2_t = vreinterpretq_u64_u8(t4);
        let v32: uint32x2_t = vqmovn_u64(v64);
        let result: uint64x1_t = vreinterpret_u64_u32(v32);

        vget_lane_u64(result, 0) == 0
    }
}

struct utf8_checking_state {
    has_error: int8x16_t,
    previous: processed_utf_bytes,
}
//
//#[cfg_attr(not(feature = "no-inline"), inline(always))]
//unsafe fn check_utf8(
//    input: &SimdInput,
//    has_error: &mut uint8x16_t,
//    previous: &mut utf8_checking_state,
//) {
//    if check_ascii_neon(input) {
//        // All bytes are ascii. Therefore the byte that was just before must be
//        // ascii too. We only check the byte that was just before simd_input. Nines
//        // are arbitrary values.
//        const verror: int8x16_t =
//            int8x16_t::new(9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 1);
//        state.has_error =
//            vorrq_s8(vreinterpretq_s8_u8(
//                vcgtq_s8(state.previous.carried_continuations, verror)),
//                     state.has_error);
//    } else {
//        // it is not ascii so we have to do heavy work
//        state.previous = check_utf8_bytes(vreinterpretq_s8_u8(input.v0),
//                                          &(state.previous), &(state.has_error));
//        state.previous = check_utf8_bytes(vreinterpretq_s8_u8(input.v1),
//                                          &(state.previous), &(state.has_error));
//        state.previous = check_utf8_bytes(vreinterpretq_s8_u8(input.v2),
//                                          &(state.previous), &(state.has_error));
//        state.previous = check_utf8_bytes(vreinterpretq_s8_u8(input.v3),
//                                          &(state.previous), &(state.has_error));
//    }
//}

// a straightforward comparison of a mask against input
#[cfg_attr(not(feature = "no-inline"), inline(always))]
unsafe fn cmp_mask_against_input(input: &SimdInput, m: u8) -> u64 {
    const MASK: uint8x16_t = vdupq_n_u8(m);

    let cmp_res_0: uint8x16_t = vceqq_u8(input.v0, MASK);
    let cmp_res_1: uint8x16_t = vceqq_u8(input.v1, MASK);
    let cmp_res_2: uint8x16_t = vceqq_u8(input.v2, MASK);
    let cmp_res_3: uint8x16_t = vceqq_u8(input.v3, MASK);

    neon_movemask_bulk(cmp_res_0, cmp_res_1, cmp_res_2, cmp_res_3)
}

// find all values less than or equal than the content of maxval (using unsigned arithmetic)
#[cfg_attr(not(feature = "no-inline"), inline(always))]
unsafe fn unsigned_lteq_against_input(input: &SimdInput, maxval: u8) -> u64 {
    const MASK: uint8x16_t = vdupq_n_u8(maxval);

    let cmp_res_0: uint8x16_t = vcleq_u8(input.v0, MASK);
    let cmp_res_1: uint8x16_t = vcleq_u8(input.v1, MASK);
    let cmp_res_2: uint8x16_t = vcleq_u8(input.v2, MASK);
    let cmp_res_3: uint8x16_t = vcleq_u8(input.v3, MASK);

    neon_movemask_bulk(cmp_res_0, cmp_res_1, cmp_res_2, cmp_res_3)
}

unsafe fn find_odd_backslash_sequences(input: &SimdInput, prev_iter_ends_odd_backslash: &mut u64) -> u64 {
    const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
    const ODD_BITS: u64 = !EVEN_BITS;

    let bs_bits: u64 = cmp_mask_against_input(input, b'\\');
    let start_edges: u64 = bs_bits & !(bs_bits << 1);
    // flip lowest if we have an odd-length run at the end of the prior
    // iteration
    let even_start_mask: u64 = EVEN_BITS ^ *prev_iter_ends_odd_backslash;
    let even_starts: u64 = start_edges & even_start_mask;
    let odd_starts: u64 = start_edges & !even_start_mask;
    let even_carries: u64 = bs_bits.wrapping_add(even_starts);

    let mut odd_carries: u64 = 0;
    // must record the carry-out of our odd-carries out of bit 63; this
    // indicates whether the sense of any edge going to the next iteration
    // should be flipped
    let iter_ends_odd_backslash: bool = add_overflow(bs_bits, odd_starts, &mut odd_carries);

    odd_carries |= *prev_iter_ends_odd_backslash; // push in bit zero as a potential end
                                                  // if we had an odd-numbered run at the
                                                  // end of the previous iteration
    *prev_iter_ends_odd_backslash = if iter_ends_odd_backslash { 0x1 } else { 0x0 };
    let even_carry_ends: u64 = even_carries & !bs_bits;
    let odd_carry_ends: u64 = odd_carries & !bs_bits;
    let even_start_odd_end: u64 = even_carry_ends & ODD_BITS;
    let odd_start_even_end: u64 = odd_carry_ends & EVEN_BITS;
    let odd_ends: u64 = even_start_odd_end | odd_start_even_end;
    odd_ends
}

unsafe fn find_quote_mask_and_bits(input: &SimdInput, odd_ends: u64,
                                   prev_iter_inside_quote: &u64, quote_bits: &mut u64, error_mask: &u64) -> u64 {
    quote_bits = cmp_mask_against_input(input, b'"');
    quote_bits = &(quote_bits & !odd_ends);
    let quote_mask: u64 = vmull_p64(-1, quote_bits);

    quote_mask ^= prev_iter_inside_quote;

    // All Unicode characters may be placed within the
    // quotation marks, except for the characters that MUST be escaped:
    // quotation mark, reverse solidus, and the control characters (U+0000
    //through U+001F).
    // https://tools.ietf.org/html/rfc8259
    let unescaped: u64 = unsigned_lteq_against_input(input, 0x1F);
    error_mask |= quote_mask & unescaped;

    // right shift of a signed value expected to be well-defined and standard
    // compliant as of C++20,
    // John Regher from Utah U. says this is fine code
    prev_iter_inside_quote = quote_mask >> 63;

    quote_mask
}

//
//#[cfg_attr(not(feature = "no-inline"), inline(always))]
//unsafe fn find_whitespace_and_structurals(
//    input: &SimdInput,
//    whitespace: &mut u64,
//    structurals: &mut u64,
//) {
//    // do a 'shufti' to detect structural JSON characters
//    // they are
//    // * `{` 0x7b
//    // * `}` 0x7d
//    // * `:` 0x3a
//    // * `[` 0x5b
//    // * `]` 0x5d
//    // * `,` 0x2c
//    // these go into the first 3 buckets of the comparison (1/2/4)
//
//    // we are also interested in the four whitespace characters:
//    // * space 0x20
//    // * linefeed 0x0a
//    // * horizontal tab 0x09
//    // * carriage return 0x0d
//    // these go into the next 2 buckets of the comparison (8/16)
//
//    // TODO: const?
//    let low_nibble_mask: __m128i = _mm_setr_epi8(
//        16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1, 2, 9, 0, 0,
//    );
//    // TODO: const?
//    let high_nibble_mask: __m128i = _mm_setr_epi8(
//        8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3, 2, 1, 0, 0,
//    );
//
//    let structural_shufti_mask: __m128i = _mm_set1_epi8(0x7);
//    let whitespace_shufti_mask: __m128i = _mm_set1_epi8(0x18);
//
//    let v_v0: __m128i = _mm_and_si128(
//        _mm_shuffle_epi8(low_nibble_mask, input.v0),
//        _mm_shuffle_epi8(
//            high_nibble_mask,
//            _mm_and_si128(_mm_srli_epi32(input.v0, 4), _mm_set1_epi8(0x7f)),
//        ),
//    );
//    let v_v1: __m128i = _mm_and_si128(
//        _mm_shuffle_epi8(low_nibble_mask, input.v1),
//        _mm_shuffle_epi8(
//            high_nibble_mask,
//            _mm_and_si128(_mm_srli_epi32(input.v1, 4), _mm_set1_epi8(0x7f)),
//        ),
//    );
//    let v_v2: __m128i = _mm_and_si128(
//        _mm_shuffle_epi8(low_nibble_mask, input.v2),
//        _mm_shuffle_epi8(
//            high_nibble_mask,
//            _mm_and_si128(_mm_srli_epi32(input.v2, 4), _mm_set1_epi8(0x7f)),
//        ),
//    );
//    let v_v3: __m128i = _mm_and_si128(
//        _mm_shuffle_epi8(low_nibble_mask, input.v3),
//        _mm_shuffle_epi8(
//            high_nibble_mask,
//            _mm_and_si128(_mm_srli_epi32(input.v3, 4), _mm_set1_epi8(0x7f)),
//        ),
//    );
//
//    let tmp_v0: __m128i = _mm_cmpeq_epi8(
//        _mm_and_si128(v_v0, structural_shufti_mask),
//        _mm_set1_epi8(0),
//    );
//    let tmp_v1: __m128i = _mm_cmpeq_epi8(
//        _mm_and_si128(v_v1, structural_shufti_mask),
//        _mm_set1_epi8(0),
//    );
//    let tmp_v2: __m128i = _mm_cmpeq_epi8(
//        _mm_and_si128(v_v2, structural_shufti_mask),
//        _mm_set1_epi8(0),
//    );
//    let tmp_v3: __m128i = _mm_cmpeq_epi8(
//        _mm_and_si128(v_v3, structural_shufti_mask),
//        _mm_set1_epi8(0),
//    );
//
//    let structural_res_0: u64 = u64::from(static_cast_u32!(_mm_movemask_epi8(tmp_v0)));
//    let structural_res_1: u64 = _mm_movemask_epi8(tmp_v1) as u64;
//    let structural_res_2: u64 = _mm_movemask_epi8(tmp_v2) as u64;
//    let structural_res_3: u64 = _mm_movemask_epi8(tmp_v3) as u64;
//
//    *structurals = !(structural_res_0 | (structural_res_1 << 16) | (structural_res_2 << 32) | (structural_res_3 << 48));
//
//    let tmp_ws_v0: __m128i = _mm_cmpeq_epi8(
//        _mm_and_si128(v_v0, whitespace_shufti_mask),
//        _mm_set1_epi8(0),
//    );
//    let tmp_ws_v1: __m128i = _mm_cmpeq_epi8(
//        _mm_and_si128(v_v1, whitespace_shufti_mask),
//        _mm_set1_epi8(0),
//    );
//    let tmp_ws_v2: __m128i = _mm_cmpeq_epi8(
//        _mm_and_si128(v_v2, whitespace_shufti_mask),
//        _mm_set1_epi8(0),
//    );
//    let tmp_ws_v3: __m128i = _mm_cmpeq_epi8(
//        _mm_and_si128(v_v3, whitespace_shufti_mask),
//        _mm_set1_epi8(0),
//    );
//
//    let ws_res_0: u64 = u64::from(static_cast_u32!(_mm_movemask_epi8(tmp_ws_v0)));
//    let ws_res_1: u64 = _mm_movemask_epi8(tmp_ws_v1) as u64;
//    let ws_res_2: u64 = _mm_movemask_epi8(tmp_ws_v2) as u64;
//    let ws_res_3: u64 = _mm_movemask_epi8(tmp_ws_v3) as u64;
//
//    *whitespace = !(ws_res_0 | (ws_res_1 << 16) | (ws_res_2 << 32) | (ws_res_3 << 48));
//}
//

// flatten out values in 'bits' assuming that they are are to have values of idx
// plus their position in the bitvector, and store these indexes at
// base_ptr[base] incrementing base as we go
// will potentially store extra values beyond end of valid bits, so base_ptr
// needs to be large enough to handle this
//TODO: usize was u32 here does this matter?
//#[cfg_attr(not(feature = "no-inline"), inline(always))]
//fn flatten_bits(base: &mut Vec<u32>, idx: u32, mut bits: u64) {
//    let cnt: usize = hamming(bits) as usize;
//    let mut l = base.len();
//    let idx_minus_64 = idx.wrapping_sub(64);
//    let idx_64_v = unsafe {
//        _mm_set_epi32(
//            static_cast_i32!(idx_minus_64),
//            static_cast_i32!(idx_minus_64),
//            static_cast_i32!(idx_minus_64),
//            static_cast_i32!(idx_minus_64),
//        )
//    };
//
//    // We're doing some trickery here.
//    // We reserve 64 extra entries, because we've at most 64 bit to set
//    // then we trunctate the base to the next base (that we calcuate above)
//    // We later indiscriminatory writre over the len we set but that's OK
//    // since we ensure we reserve the needed space
//    base.reserve(64);
//    unsafe {
//        base.set_len(l + cnt);
//    }
//
//    while bits != 0 {
//        unsafe {
//            let v0 = static_cast_i32!(trailingzeroes(bits));
//            bits &= bits.wrapping_sub(1);
//            let v1 = static_cast_i32!(trailingzeroes(bits));
//            bits &= bits.wrapping_sub(1);
//            let v2 = static_cast_i32!(trailingzeroes(bits));
//            bits &= bits.wrapping_sub(1);
//            let v3 = static_cast_i32!(trailingzeroes(bits));
//            bits &= bits.wrapping_sub(1);
//
//            let v: __m128i = _mm_set_epi32(v3, v2, v1, v0);
//            let v: __m128i = _mm_add_epi32(idx_64_v, v);
//            #[allow(clippy::cast_ptr_alignment)]
//                _mm_storeu_si128(base.as_mut_ptr().add(l) as *mut __m128i, v);
//        }
//        l += 4;
//    }
//}

//
//// return a updated structural bit vector with quoted contents cleared out and
//// pseudo-structural characters added to the mask
//// updates prev_iter_ends_pseudo_pred which tells us whether the previous
//// iteration ended on a whitespace or a structural character (which means that
//// the next iteration
//// will have a pseudo-structural character at its start)
//#[cfg_attr(not(feature = "no-inline"), inline(always))]
//fn finalize_structurals(
//    mut structurals: u64,
//    whitespace: u64,
//    quote_mask: u64,
//    quote_bits: u64,
//    prev_iter_ends_pseudo_pred: &mut u64,
//) -> u64 {
//    // mask off anything inside quotes
//    structurals &= !quote_mask;
//    // add the real quote bits back into our bitmask as well, so we can
//    // quickly traverse the strings we've spent all this trouble gathering
//    structurals |= quote_bits;
//    // Now, establish "pseudo-structural characters". These are non-whitespace
//    // characters that are (a) outside quotes and (b) have a predecessor that's
//    // either whitespace or a structural character. This means that subsequent
//    // passes will get a chance to encounter the first character of every string
//    // of non-whitespace and, if we're parsing an atom like true/false/null or a
//    // number we can stop at the first whitespace or structural character
//    // following it.
//
//    // a qualified predecessor is something that can happen 1 position before an
//    // psuedo-structural character
//    let pseudo_pred: u64 = structurals | whitespace;
//
//    let shifted_pseudo_pred: u64 = (pseudo_pred << 1) | *prev_iter_ends_pseudo_pred;
//    *prev_iter_ends_pseudo_pred = pseudo_pred >> 63;
//    let pseudo_structurals: u64 = shifted_pseudo_pred & (!whitespace) & (!quote_mask);
//    structurals |= pseudo_structurals;
//
//    // now, we've used our close quotes all we need to. So let's switch them off
//    // they will be off in the quote mask and on in quote bits.
//    structurals &= !(quote_bits & !quote_mask);
//    structurals
//}
//

//impl<'de> Deserializer<'de> {
//    #[inline(never)]
//    pub unsafe fn find_structural_bits(input: &[u8]) -> std::result::Result<Vec<u32>, ErrorType> {
//        let len = input.len();
//        // 6 is a heuristic number to estimate it turns out a rate of 1/6 structural caracters lears
//        // almost never to relocations.
//        let mut structural_indexes = Vec::with_capacity(len / 6);
//        structural_indexes.push(0); // push extra root element
//
//        let mut has_error: __m128i = _mm_setzero_si128();
//        let mut previous = AvxProcessedUtfBytes::default();
//        // we have padded the input out to 64 byte multiple with the remainder being
//        // zeros
//
//        // persistent state across loop
//        // does the last iteration end with an odd-length sequence of backslashes?
//        // either 0 or 1, but a 64-bit value
//        let mut prev_iter_ends_odd_backslash: u64 = 0;
//        // does the previous iteration end inside a double-quote pair?
//        let mut prev_iter_inside_quote: u64 = 0;
//        // either all zeros or all ones
//        // does the previous iteration end on something that is a predecessor of a
//        // pseudo-structural character - i.e. whitespace or a structural character
//        // effectively the very first char is considered to follow "whitespace" for
//        // the
//        // purposes of pseudo-structural character detection so we initialize to 1
//        let mut prev_iter_ends_pseudo_pred: u64 = 1;
//
//        // structurals are persistent state across loop as we flatten them on the
//        // subsequent iteration into our array pointed to be base_ptr.
//        // This is harmless on the first iteration as structurals==0
//        // and is done for performance reasons; we can hide some of the latency of the
//        // expensive carryless multiply in the previous step with this work
//        let mut structurals: u64 = 0;
//
//        let lenminus64: usize = if len < 64 { 0 } else { len as usize - 64 };
//        let mut idx: usize = 0;
//        let mut error_mask: u64 = 0; // for unescaped characters within strings (ASCII code points < 0x20)
//
//        while idx < lenminus64 {
//            /*
//            #ifndef _MSC_VER
//              __builtin_prefetch(buf + idx + 128);
//            #endif
//             */
//            let input: SimdInput = fill_input(input.get_unchecked(idx as usize..));
//            check_utf8(&input, &mut has_error, &mut previous);
//            // detect odd sequences of backslashes
//            let odd_ends: u64 =
//                find_odd_backslash_sequences(&input, &mut prev_iter_ends_odd_backslash);
//
//            // detect insides of quote pairs ("quote_mask") and also our quote_bits
//            // themselves
//            let mut quote_bits: u64 = 0;
//            let quote_mask: u64 = find_quote_mask_and_bits(
//                &input,
//                odd_ends,
//                &mut prev_iter_inside_quote,
//                &mut quote_bits,
//                &mut error_mask,
//            );
//
//            // take the previous iterations structural bits, not our current iteration,
//            // and flatten
//            flatten_bits(&mut structural_indexes, idx as u32, structurals);
//
//            let mut whitespace: u64 = 0;
//            find_whitespace_and_structurals(&input, &mut whitespace, &mut structurals);
//
//            // fixup structurals to reflect quotes and add pseudo-structural characters
//            structurals = finalize_structurals(
//                structurals,
//                whitespace,
//                quote_mask,
//                quote_bits,
//                &mut prev_iter_ends_pseudo_pred,
//            );
//            idx += 64;
//        }
//
//        // we use a giant copy-paste which is ugly.
//        // but otherwise the string needs to be properly padded or else we
//        // risk invalidating the UTF-8 checks.
//        if idx < len {
//            let mut tmpbuf: [u8; 64] = [0x20; 64];
//            tmpbuf
//                .as_mut_ptr()
//                .copy_from(input.as_ptr().add(idx), len as usize - idx);
//            let input: SimdInput = fill_input(&tmpbuf);
//
//            check_utf8(&input, &mut has_error, &mut previous);
//
//            // detect odd sequences of backslashes
//            let odd_ends: u64 =
//                find_odd_backslash_sequences(&input, &mut prev_iter_ends_odd_backslash);
//
//            // detect insides of quote pairs ("quote_mask") and also our quote_bits
//            // themselves
//            let mut quote_bits: u64 = 0;
//            let quote_mask: u64 = find_quote_mask_and_bits(
//                &input,
//                odd_ends,
//                &mut prev_iter_inside_quote,
//                &mut quote_bits,
//                &mut error_mask,
//            );
//
//            // take the previous iterations structural bits, not our current iteration,
//            // and flatten
//            flatten_bits(&mut structural_indexes, idx as u32, structurals);
//
//            let mut whitespace: u64 = 0;
//            find_whitespace_and_structurals(&input, &mut whitespace, &mut structurals);
//
//            // fixup structurals to reflect quotes and add pseudo-structural characters
//            structurals = finalize_structurals(
//                structurals,
//                whitespace,
//                quote_mask,
//                quote_bits,
//                &mut prev_iter_ends_pseudo_pred,
//            );
//            idx += 64;
//        }
//        // This test isn't in upstream, for some reason the error mask is et for then.
//        if prev_iter_inside_quote != 0 {
//            return Err(ErrorType::Syntax);
//        }
//        // finally, flatten out the remaining structurals from the last iteration
//        flatten_bits(&mut structural_indexes, idx as u32, structurals);
//
//        // a valid JSON file cannot have zero structural indexes - we should have
//        // found something (note that we compare to 1 as we always add the root!)
//        if structural_indexes.len() == 1 {
//            return Err(ErrorType::EOF);
//        }
//
//        if structural_indexes.last() > Some(&(len as u32)) {
//            return Err(ErrorType::InternalError);
//        }
//
//        if error_mask != 0 {
//            return Err(ErrorType::Syntax);
//        }
//
//        if _mm_testz_si128(has_error, has_error) != 0 {
//            Ok(structural_indexes)
//        } else {
//            Err(ErrorType::InvalidUTF8)
//        }
//    }
//}
