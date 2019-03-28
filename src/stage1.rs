#![allow(dead_code)]
use crate::parsedjson::*;
use crate::portability::*;
use crate::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::mem;

#[derive(Debug)]
struct SimdInput {
    lo: __m256i,
    hi: __m256i,
}

unsafe fn fill_input(ptr: &[u8]) -> SimdInput {
    SimdInput {
        lo: _mm256_loadu_si256(ptr[0..32].as_ptr() as *const __m256i),
        hi: _mm256_loadu_si256(ptr[32..64].as_ptr() as *const __m256i),
    }
}

/*

#[inline(always)]
unsafe fn check_utf8(input: SimdInput, has_error: &mut __m256i,
                              struct avx_processed_utf_bytes &previous) {
    __m256i highbit = _mm256_set1_epi8(0x80);
    if ((_mm256_testz_si256(_mm256_or_si256(in.lo, in.hi), highbit)) == 1) {
        // it is ascii, we just check continuation
        has_error = _mm256_or_si256(
            _mm256_cmpgt_epi8(
                previous.carried_continuations,
                _mm256_setr_epi8(9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                                 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 1)),
            has_error);
    } else {
        // it is not ascii so we have to do heavy work
        previous = avxcheckUTF8Bytes(in.lo, &previous, &has_error);
        previous = avxcheckUTF8Bytes(in.hi, &previous, &has_error);
    }
}
 */

/// a straightforward comparison of a mask against input. 5 uops; would be
/// cheaper in AVX512.
#[inline(always)]
fn cmp_mask_against_input(input: &SimdInput, mask: __m256i) -> u64 {
    unsafe {
        let cmp_res_0: __m256i = _mm256_cmpeq_epi8(input.lo, mask);
        // TODO: c++ uses static cast, here what are the implications?
        let res_0: u64 = static_cast_u32!(_mm256_movemask_epi8(cmp_res_0)) as u64;
        let cmp_res_1: __m256i = _mm256_cmpeq_epi8(input.hi, mask);
        let res_1: u64 = _mm256_movemask_epi8(cmp_res_1) as u64;
        res_0 | (res_1 << 32)
    }
}

// find all values less than or equal than the content of maxval (using unsigned arithmetic)
#[inline(always)]
fn unsigned_lteq_against_input(input: &SimdInput, maxval: __m256i) -> u64 {
    unsafe {
        let cmp_res_0: __m256i = _mm256_cmpeq_epi8(_mm256_max_epu8(maxval, input.lo), maxval);
        // TODO: c++ uses static cast, here what are the implications?
        let res_0: u64 = static_cast_u32!(_mm256_movemask_epi8(cmp_res_0)) as u64;
        let cmp_res_1: __m256i = _mm256_cmpeq_epi8(_mm256_max_epu8(maxval, input.hi), maxval);
        let res_1: u64 = _mm256_movemask_epi8(cmp_res_1) as u64;
        res_0 | (res_1 << 32)
    }
}

// return a bitvector indicating where we have characters that end an odd-length
// sequence of backslashes (and thus change the behavior of the next character
// to follow). A even-length sequence of backslashes, and, for that matter, the
// largest even-length prefix of our odd-length sequence of backslashes, simply
// modify the behavior of the backslashes themselves.
// We also update the prev_iter_ends_odd_backslash reference parameter to
// indicate whether we end an iteration on an odd-length sequence of
// backslashes, which modifies our subsequent search for odd-length
// sequences of backslashes in an obvious way.
#[inline(always)]
fn find_odd_backslash_sequences(input: &SimdInput, prev_iter_ends_odd_backslash: &mut u64) -> u64 {
    use std::num::Wrapping;
    // TODO: const?
    let even_bits: u64 = 0x5555555555555555;
    // TODO: const?
    let odd_bits: u64 = !even_bits;
    let bs_bits: u64 = cmp_mask_against_input(&input, unsafe { _mm256_set1_epi8(b'\\' as i8) });
    let start_edges: u64 = bs_bits & !(bs_bits << 1);
    // flip lowest if we have an odd-length run at the end of the prior
    // iteration
    let even_start_mask: u64 = even_bits ^ *prev_iter_ends_odd_backslash;
    let even_starts: u64 = start_edges & even_start_mask;
    let odd_starts: u64 = start_edges & !even_start_mask;
    let even_carries: u64 = (Wrapping(bs_bits) + Wrapping(even_starts)).0;

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
    let even_start_odd_end: u64 = even_carry_ends & odd_bits;
    let odd_start_even_end: u64 = odd_carry_ends & even_bits;
    let odd_ends: u64 = even_start_odd_end | odd_start_even_end;
    return odd_ends;
}

// return both the quote mask (which is a half-open mask that covers the first
// quote
// in an unescaped quote pair and everything in the quote pair) and the quote
// bits, which are the simple
// unescaped quoted bits. We also update the prev_iter_inside_quote value to
// tell the next iteration
// whether we finished the final iteration inside a quote pair; if so, this
// inverts our behavior of
// whether we're inside quotes for the next iteration.
// Note that we don't do any error checking to see if we have backslash
// sequences outside quotes; these
// backslash sequences (of any length) will be detected elsewhere.
#[inline(always)]
unsafe fn find_quote_mask_and_bits(
    input: &SimdInput,
    odd_ends: u64,
    prev_iter_inside_quote: &mut u64,
    quote_bits: &mut u64,
    error_mask: &mut u64,
) -> u64 {
    *quote_bits = cmp_mask_against_input(&input, _mm256_set1_epi8(b'"' as i8));
    *quote_bits = *quote_bits & !odd_ends;
    // remove from the valid quoted region the unescapted characters.
    #[allow(overflowing_literals)]
    let mut quote_mask: u64 = _mm_cvtsi128_si64(_mm_clmulepi64_si128(
        _mm_set_epi64x(0, *quote_bits as i64),
        _mm_set1_epi8(0xFF),
        0,
    )) as u64;
    quote_mask ^= *prev_iter_inside_quote;
    // All Unicode characters may be placed within the
    // quotation marks, except for the characters that MUST be escaped:
    // quotation mark, reverse solidus, and the control characters (U+0000
    //through U+001F).
    // https://tools.ietf.org/html/rfc8259
    let unescaped: u64 = unsigned_lteq_against_input(&input, _mm256_set1_epi8(0x1F));
    *error_mask |= quote_mask & unescaped;
    // right shift of a signed value expected to be well-defined and standard
    // compliant as of C++20,
    // John Regher from Utah U. says this is fine code
    *prev_iter_inside_quote = static_cast_u64!(static_cast_i64!(quote_mask) >> 63);
    return quote_mask;
}

#[inline(always)]
unsafe fn find_whitespace_and_structurals(
    input: &SimdInput,
    whitespace: &mut u64,
    structurals: &mut u64,
) {
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

    // TODO: const?
    let low_nibble_mask: __m256i = _mm256_setr_epi8(
        16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1, 2, 9, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1, 2,
        9, 0, 0,
    );
    // TODO: const?
    let high_nibble_mask: __m256i = _mm256_setr_epi8(
        8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3, 2, 1, 0, 0, 8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3, 2, 1,
        0, 0,
    );

    let structural_shufti_mask: __m256i = _mm256_set1_epi8(0x7);
    let whitespace_shufti_mask: __m256i = _mm256_set1_epi8(0x18);

    let v_lo: __m256i = _mm256_and_si256(
        _mm256_shuffle_epi8(low_nibble_mask, input.lo),
        _mm256_shuffle_epi8(
            high_nibble_mask,
            _mm256_and_si256(_mm256_srli_epi32(input.lo, 4), _mm256_set1_epi8(0x7f)),
        ),
    );

    let v_hi: __m256i = _mm256_and_si256(
        _mm256_shuffle_epi8(low_nibble_mask, input.hi),
        _mm256_shuffle_epi8(
            high_nibble_mask,
            _mm256_and_si256(_mm256_srli_epi32(input.hi, 4), _mm256_set1_epi8(0x7f)),
        ),
    );
    let tmp_lo: __m256i = _mm256_cmpeq_epi8(
        _mm256_and_si256(v_lo, structural_shufti_mask),
        _mm256_set1_epi8(0),
    );
    let tmp_hi: __m256i = _mm256_cmpeq_epi8(
        _mm256_and_si256(v_hi, structural_shufti_mask),
        _mm256_set1_epi8(0),
    );

    let structural_res_0: u64 = static_cast_u32!(_mm256_movemask_epi8(tmp_lo)) as u64;
    let structural_res_1: u64 = _mm256_movemask_epi8(tmp_hi) as u64;
    *structurals = !(structural_res_0 | (structural_res_1 << 32));

    let tmp_ws_lo: __m256i = _mm256_cmpeq_epi8(
        _mm256_and_si256(v_lo, whitespace_shufti_mask),
        _mm256_set1_epi8(0),
    );
    let tmp_ws_hi: __m256i = _mm256_cmpeq_epi8(
        _mm256_and_si256(v_hi, whitespace_shufti_mask),
        _mm256_set1_epi8(0),
    );

    let ws_res_0: u64 = static_cast_u32!(_mm256_movemask_epi8(tmp_ws_lo)) as u64;
    let ws_res_1: u64 = _mm256_movemask_epi8(tmp_ws_hi) as u64;
    *whitespace = !(ws_res_0 | (ws_res_1 << 32));
}

// flatten out values in 'bits' assuming that they are are to have values of idx
// plus their position in the bitvector, and store these indexes at
// base_ptr[base] incrementing base as we go
// will potentially store extra values beyond end of valid bits, so base_ptr
// needs to be large enough to handle this
//TODO: usize was u32 here does this matter?
#[inline(always)]
fn flatten_bits(base: &mut Vec<u32>, idx: u32, mut bits: u64) {
    use std::num::Wrapping;
    let cnt: usize = hamming(bits) as usize;
    let next_base: usize = base.len() + cnt;
    while bits != 0 {
        unsafe {
            base.push(static_cast_u32!(idx) - 64 + trailingzeroes(bits));
            bits = bits & (Wrapping(bits) - Wrapping(1)).0;
            base.push(static_cast_u32!(idx) - 64 + trailingzeroes(bits));
            bits = bits & (Wrapping(bits) - Wrapping(1)).0;
            base.push(static_cast_u32!(idx) - 64 + trailingzeroes(bits));
            bits = bits & (Wrapping(bits) - Wrapping(1)).0;
            base.push(static_cast_u32!(idx) - 64 + trailingzeroes(bits));
            bits = bits & (Wrapping(bits) - Wrapping(1)).0;
            base.push(static_cast_u32!(idx) - 64 + trailingzeroes(bits));
            bits = bits & (Wrapping(bits) - Wrapping(1)).0;
            base.push(static_cast_u32!(idx) - 64 + trailingzeroes(bits));
            bits = bits & (Wrapping(bits) - Wrapping(1)).0;
            base.push(static_cast_u32!(idx) - 64 + trailingzeroes(bits));
            bits = bits & (Wrapping(bits) - Wrapping(1)).0;
            base.push(static_cast_u32!(idx) - 64 + trailingzeroes(bits));
            bits = bits & (Wrapping(bits) - Wrapping(1)).0;
        }
    }
    base.truncate(next_base);
}

// return a updated structural bit vector with quoted contents cleared out and
// pseudo-structural characters added to the mask
// updates prev_iter_ends_pseudo_pred which tells us whether the previous
// iteration ended on a whitespace or a structural character (which means that
// the next iteration
// will have a pseudo-structural character at its start)
#[inline(always)]
fn finalize_structurals(
    mut structurals: u64,
    whitespace: u64,
    quote_mask: u64,
    quote_bits: u64,
    prev_iter_ends_pseudo_pred: &mut u64,
) -> u64 {
    // mask off anything inside quotes
    structurals &= !quote_mask;
    // add the real quote bits back into our bitmask as well, so we can
    // quickly traverse the strings we've spent all this trouble gathering
    structurals |= quote_bits;
    // Now, establish "pseudo-structural characters". These are non-whitespace
    // characters that are (a) outside quotes and (b) have a predecessor that's
    // either whitespace or a structural character. This means that subsequent
    // passes will get a chance to encounter the first character of every string
    // of non-whitespace and, if we're parsing an atom like true/false/null or a
    // number we can stop at the first whitespace or structural character
    // following it.

    // a qualified predecessor is something that can happen 1 position before an
    // psuedo-structural character
    let pseudo_pred: u64 = structurals | whitespace;

    let shifted_pseudo_pred: u64 = (pseudo_pred << 1) | *prev_iter_ends_pseudo_pred;
    *prev_iter_ends_pseudo_pred = pseudo_pred >> 63;
    let pseudo_structurals: u64 = shifted_pseudo_pred & (!whitespace) & (!quote_mask);
    structurals |= pseudo_structurals;

    // now, we've used our close quotes all we need to. So let's switch them off
    // they will be off in the quote mask and on in quote bits.
    structurals &= !(quote_bits & !quote_mask);
    return structurals;
}

//WARN_UNUSED
/*never_inline*/
//#[inline(never)]
pub unsafe fn find_structural_bits(buf: &[u8], len: u32, pj: &mut ParsedJson) -> bool {
    /*
    #ifdef SIMDJSON_UTF8VALIDATE
      __m256i has_error = _mm256_setzero_si256();
      struct avx_processed_utf_bytes previous {};
      previous.rawbytes = _mm256_setzero_si256();
      previous.high_nibbles = _mm256_setzero_si256();
      previous.carried_continuations = _mm256_setzero_si256();
    #endif
    */
    // we have padded the input out to 64 byte multiple with the remainder being
    // zeros

    // persistent state across loop
    // does the last iteration end with an odd-length sequence of backslashes?
    // either 0 or 1, but a 64-bit value
    let mut prev_iter_ends_odd_backslash: u64 = 0;
    // does the previous iteration end inside a double-quote pair?
    let mut prev_iter_inside_quote: u64 = 0; // either all zeros or all ones
                                             // does the previous iteration end on something that is a predecessor of a
                                             // pseudo-structural character - i.e. whitespace or a structural character
                                             // effectively the very first char is considered to follow "whitespace" for
                                             // the
                                             // purposes of pseudo-structural character detection so we initialize to 1
    let mut prev_iter_ends_pseudo_pred: u64 = 1;

    // structurals are persistent state across loop as we flatten them on the
    // subsequent iteration into our array pointed to be base_ptr.
    // This is harmless on the first iteration as structurals==0
    // and is done for performance reasons; we can hide some of the latency of the
    // expensive carryless multiply in the previous step with this work
    let mut structurals: u64 = 0;

    let lenminus64: usize = if len < 64 { 0 } else { len as usize - 64 };
    let mut idx: usize = 0;
    let mut error_mask: u64 = 0; // for unescaped characters within strings (ASCII code points < 0x20)

    while idx < lenminus64 {
        /*
        #ifndef _MSC_VER
          __builtin_prefetch(buf + idx + 128);
        #endif
         */
        let input: SimdInput = fill_input(&buf[idx as usize..]);
        /*
        #ifdef SIMDJSON_UTF8VALIDATE
          check_utf8(in, has_error, previous);
        #endif
         */
        // detect odd sequences of backslashes
        let odd_ends: u64 = find_odd_backslash_sequences(&input, &mut prev_iter_ends_odd_backslash);

        // detect insides of quote pairs ("quote_mask") and also our quote_bits
        // themselves
        let mut quote_bits: u64 = 0;
        let quote_mask: u64 = find_quote_mask_and_bits(
            &input,
            odd_ends,
            &mut prev_iter_inside_quote,
            &mut quote_bits,
            &mut error_mask,
        );

        // take the previous iterations structural bits, not our current iteration,
        // and flatten
        flatten_bits(&mut pj.structural_indexes, idx as u32, structurals);

        let mut whitespace: u64 = 0;
        find_whitespace_and_structurals(&input, &mut whitespace, &mut structurals);

        // fixup structurals to reflect quotes and add pseudo-structural characters
        structurals = finalize_structurals(
            structurals,
            whitespace,
            quote_mask,
            quote_bits,
            &mut prev_iter_ends_pseudo_pred,
        );
        idx += 64;
    }

    // we use a giant copy-paste which is ugly.
    // but otherwise the string needs to be properly padded or else we
    // risk invalidating the UTF-8 checks.
    if idx < len as usize {
        let mut tmpbuf: [u8; 64] = [0x20; 64];
        tmpbuf
            .as_mut_ptr()
            .copy_from(buf.as_ptr().offset(idx as isize), len as usize - idx);
        let input: SimdInput = fill_input(&tmpbuf);
        /*
        #ifdef SIMDJSON_UTF8VALIDATE
            check_utf8(in, has_error, previous);
        #endif
         */

        // detect odd sequences of backslashes
        let odd_ends: u64 = find_odd_backslash_sequences(&input, &mut prev_iter_ends_odd_backslash);

        // detect insides of quote pairs ("quote_mask") and also our quote_bits
        // themselves
        let mut quote_bits: u64 = 0;
        let quote_mask: u64 = find_quote_mask_and_bits(
            &input,
            odd_ends,
            &mut prev_iter_inside_quote,
            &mut quote_bits,
            &mut error_mask,
        );

        // take the previous iterations structural bits, not our current iteration,
        // and flatten
        flatten_bits(&mut pj.structural_indexes, idx as u32, structurals);

        let mut whitespace: u64 = 0;
        find_whitespace_and_structurals(&input, &mut whitespace, &mut structurals);

        // fixup structurals to reflect quotes and add pseudo-structural characters
        structurals = finalize_structurals(
            structurals,
            whitespace,
            quote_mask,
            quote_bits,
            &mut prev_iter_ends_pseudo_pred,
        );
        idx += 64;
    }
    // finally, flatten out the remaining structurals from the last iteration
    flatten_bits(&mut pj.structural_indexes, idx as u32, structurals);

    // a valid JSON file cannot have zero structural indexes - we should have
    // found something
    if pj.n_structural_indexes == 0 {
        return false;
    }
    if pj.structural_indexes.last() > Some(&len) {
        eprintln!("Internal bug");
        return false;
    }
    /*
    if Some(&len) != pj.structural_indexes.last() {
        // the string might not be NULL terminated, but we add a virtual NULL ending
        // character.
        pj.structural_indexes.push(len);
        pj.n_structural_indexes += 1;
    }
    */
    // make it safe to dereference one beyond this array
    pj.structural_indexes.push(0);
    pj.n_structural_indexes = pj.structural_indexes.len();
    if error_mask != 0 {
        return false;
    }
    /*
    #ifdef SIMDJSON_UTF8VALIDATE
      return _mm256_testz_si256(has_error, has_error) != 0;
    #else
     */
    return true;
}
