use crate::charutils::*;
use crate::parsedjson::*;
use crate::portability::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// begin copypasta
// These chars yield themselves: " \ /
// b -> backspace, f -> formfeed, n -> newline, r -> cr, t -> horizontal tab
// u not handled in this table as it's complex
const ESCAPE_MAP: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x0.
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0x2f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, // 0x4.
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x5c, 0, 0, 0, // 0x5.
    0, 0, 0x08, 0, 0, 0, 0x0c, 0, 0, 0, 0, 0, 0, 0, 0x0a, 0, // 0x6.
    0, 0, 0x0d, 0, 0x09, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x7.
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// handle a unicode codepoint
// write appropriate values into dest
// src will advance 6 bytes or 12 bytes
// dest will advance a variable amount (return via pointer)
// return true if the unicode codepoint was valid
// We work in little-endian then swap at write time
#[inline(always)]
unsafe fn handle_unicode_codepoint(src_ptr: &mut *const u8, dst_ptr: &mut *mut u8) -> bool {
    // hex_to_u32_nocheck fills high 16 bits of the return value with 1s if the
    // conversion isn't valid; we defer the check for this to inside the
    // multilingual plane check
    let mut code_point: u32 = hex_to_u32_nocheck(src_ptr.offset(2));
    *src_ptr = src_ptr.add(6);
    // check for low surrogate for characters outside the Basic
    // Multilingual Plane.
    if code_point >= 0xd800 && code_point < 0xdc00 {
        if (*src_ptr.offset(0) != b'\\') || *src_ptr.offset(1) != b'u' {
            return false;
        }

        let code_point_2: u32 = hex_to_u32_nocheck(src_ptr.offset(2));

        // if the first code point is invalid we will get here, as we will go past
        // the check for being outside the Basic Multilingual plane. If we don't
        // find a \u immediately afterwards we fail out anyhow, but if we do,
        // this check catches both the case of the first code point being invalid
        // or the second code point being invalid.
        if ((code_point | code_point_2) >> 16) != 0 {
            return false;
        }

        code_point = (((code_point - 0xd800) << 10) | (code_point_2 - 0xdc00)) + 0x10000;
        *src_ptr = src_ptr.add(6);
    }
    let offset: usize = codepoint_to_utf8(code_point, *dst_ptr);
    *dst_ptr = dst_ptr.add(offset);
    return offset > 0;
}

#[inline(always)]
pub unsafe fn parse_string(
    buf: *const u8,
    _len: usize,
    pj: &mut ParsedJson,
    _depth: usize,
    offset: u32,
) -> bool {
    use std::num::Wrapping;

    pj.write_tape(pj.strings.len(), b'"');
    let mut src = buf.add(offset as usize + 1);
    let mut dst_vec = vec![0u8; 1024];
    let mut dst = dst_vec.as_mut_ptr();
    let start = dst;
    //uint8_t *dst = pj.current_string_buf_loc + sizeof(uint32_t);
    //const uint8_t *const start_of_string = dst;
    loop {
        let v: __m256i = _mm256_loadu_si256(src as *const __m256i);
        // store to dest unconditionally - we can overwrite the bits we don't like
        // later
        _mm256_storeu_si256(dst as *mut __m256i, v);
        let bs_bits: u32 =
            _mm256_movemask_epi8(_mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'\\' as i8))) as u32;
        let quote_mask = _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'"' as i8));
        let quote_bits = _mm256_movemask_epi8(quote_mask) as u32;
        if ((Wrapping(bs_bits) - Wrapping(1)).0 & quote_bits) != 0 {
            // we encountered quotes first. Move dst to point to quotes and exit
            // find out where the quote is...
            let quote_dist: u32 = trailingzeroes(quote_bits as u64);

            // NULL termination is still handy if you expect all your strings to be NULL terminated?
            // It comes at a small cost
            *dst.offset(quote_dist as isize) = 0;
            let str_length: usize = (dst.offset_from(start) + quote_dist as isize) as usize;
            dst_vec.truncate(str_length);
            ///////////////////////
            // Above, check for overflow in case someone has a crazy string (>=4GB?)
            // But only add the overflow check when the document itself exceeds 4GB
            // Currently unneeded because we refuse to parse docs larger or equal to 4GB.
            ////////////////////////

            // we advance the point, accounting for the fact that we have a NULl termination
            //pj.current_string_buf_loc = dst + quote_dist + 1;

            /*
            #ifdef JSON_TEST_STRINGS // for unit testing
            foundString(buf + offset,start_of_string,pj.current_string_buf_loc - 1);
            #endif // JSON_TEST_STRINGS
             */
            dbg!(String::from_utf8_lossy(&dst_vec));
            pj.strings.push(dst_vec);
            return true;
        }
        if ((quote_bits - 1) & bs_bits) != 0 {
            // find out where the backspace is
            let bs_dist: u32 = trailingzeroes(bs_bits as u64);
            let escape_char: u8 = *src.offset(bs_dist as isize + 1);
            // we encountered backslash first. Handle backslash
            if escape_char == b'u' {
                // move src/dst up to the start; they will be further adjusted
                // within the unicode codepoint handling code.
                src = src.add(bs_dist as usize);
                dst = dst.add(bs_dist as usize);
                if !handle_unicode_codepoint(&mut src, &mut dst) {
                    /*
                    #ifdef JSON_TEST_STRINGS // for unit testing
                    foundBadString(buf + offset);
                    #endif // JSON_TEST_STRINGS
                     */
                    pj.strings.push(dst_vec);
                    return false;
                }
            } else {
                // simple 1:1 conversion. Will eat bs_dist+2 characters in input and
                // write bs_dist+1 characters to output
                // note this may reach beyond the part of the buffer we've actually
                // seen. I think this is ok
                let escape_result: u8 = ESCAPE_MAP[escape_char as usize];
                if escape_result == 0 {
                    /*
                    #ifdef JSON_TEST_STRINGS // for unit testing
                    foundBadString(buf + offset);
                    #endif // JSON_TEST_STRINGS
                    */
                    pj.strings.push(dst_vec);
                    return false; // bogus escape value is an error
                }
                *dst.offset(bs_dist as isize) = escape_result;
                src = src.add(bs_dist as usize + 2);
                dst = dst.add(bs_dist as usize + 1);
            }
        } else {
            // they are the same. Since they can't co-occur, it means we encountered
            // neither.

            src = src.add(32);
            dst = dst.add(32);
        }
    }
}
