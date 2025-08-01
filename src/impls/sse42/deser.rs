#[cfg(target_arch = "x86")]
use std::arch::x86 as arch;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64 as arch;

use crate::{
    Deserializer, Result, SillyWrapper,
    error::ErrorType,
    safer_unchecked::GetSaferUnchecked,
    stringparse::{ESCAPE_MAP, handle_unicode_codepoint},
};
use arch::{
    __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8, _mm_storeu_si128,
};

#[target_feature(enable = "sse4.2")]
#[allow(clippy::if_not_else, clippy::cast_possible_wrap)]
#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) unsafe fn parse_str<'invoke, 'de>(
    input: SillyWrapper<'de>,
    data: &'invoke [u8],
    buffer: &'invoke mut [u8],
    mut idx: usize,
) -> Result<&'de str> {
    unsafe {
        use ErrorType::{InvalidEscape, InvalidUnicodeCodepoint};
        let input = input.input;
        // Add 1 to skip the initial "
        idx += 1;

        // we include the terminal '"' so we know where to end
        // This is safe since we check sub's length in the range access above and only
        // create sub sliced form sub to `sub.len()`.

        let src: &[u8] = data.get_kinda_unchecked(idx..);
        let mut src_i: usize = 0;
        let mut len = src_i;
        loop {
            // _mm_loadu_si128 does not require alignment
            #[allow(clippy::cast_ptr_alignment)]
            let v: __m128i = _mm_loadu_si128(src.as_ptr().add(src_i).cast::<arch::__m128i>());

            // store to dest unconditionally - we can overwrite the bits we don't like
            // later
            let bs_bits: u32 = static_cast_u32!(_mm_movemask_epi8(_mm_cmpeq_epi8(
                v,
                _mm_set1_epi8(b'\\' as i8)
            )));
            let quote_mask = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'"' as i8));
            let quote_bits = static_cast_u32!(_mm_movemask_epi8(quote_mask));
            if (bs_bits.wrapping_sub(1) & quote_bits) != 0 {
                // we encountered quotes first. Move dst to point to quotes and exit
                // find out where the quote is...
                let quote_dist: u32 = quote_bits.trailing_zeros();

                ///////////////////////
                // Above, check for overflow in case someone has a crazy string (>=4GB?)
                // But only add the overflow check when the document itself exceeds 4GB
                // Currently unneeded because we refuse to parse docs larger or equal to 4GB.
                ////////////////////////

                // we advance the point, accounting for the fact that we have a NULl termination

                len += quote_dist as usize;
                let v =
                    std::str::from_utf8_unchecked(std::slice::from_raw_parts(input.add(idx), len));
                return Ok(v);

                // we compare the pointers since we care if they are 'at the same spot'
                // not if they are the same value
            }
            if (quote_bits.wrapping_sub(1) & bs_bits) == 0 {
                // they are the same. Since they can't co-occur, it means we encountered
                // neither.
                src_i += 16;
                len += 16;
            } else {
                // Move to the 'bad' character
                let bs_dist: u32 = bs_bits.trailing_zeros();
                len += bs_dist as usize;
                src_i += bs_dist as usize;
                break;
            }
        }

        let mut dst_i: usize = 0;

        // To be more conform with upstream
        loop {
            // _mm_loadu_si128 does not require alignment
            #[allow(clippy::cast_ptr_alignment)]
            let v: __m128i = _mm_loadu_si128(src.as_ptr().add(src_i).cast::<arch::__m128i>());

            // _mm_storeu_si128 does not require alignment
            #[allow(clippy::cast_ptr_alignment)]
            _mm_storeu_si128(buffer.as_mut_ptr().add(dst_i).cast::<arch::__m128i>(), v);

            // store to dest unconditionally - we can overwrite the bits we don't like
            // later
            let bs_bits: u32 = static_cast_u32!(_mm_movemask_epi8(_mm_cmpeq_epi8(
                v,
                _mm_set1_epi8(b'\\' as i8)
            )));
            let quote_mask = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'"' as i8));
            let quote_bits = static_cast_u32!(_mm_movemask_epi8(quote_mask));
            if (bs_bits.wrapping_sub(1) & quote_bits) != 0 {
                // we encountered quotes first. Move dst to point to quotes and exit
                // find out where the quote is...
                let quote_dist: u32 = quote_bits.trailing_zeros();

                ///////////////////////
                // Above, check for overflow in case someone has a crazy string (>=4GB?)
                // But only add the overflow check when the document itself exceeds 4GB
                // Currently unneeded because we refuse to parse docs larger or equal to 4GB.
                ////////////////////////

                // we advance the point, accounting for the fact that we have a NULl termination

                dst_i += quote_dist as usize;
                input
                    .add(idx + len)
                    .copy_from_nonoverlapping(buffer.as_ptr(), dst_i);
                let v = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    input.add(idx),
                    len + dst_i,
                ));
                return Ok(v);

                // we compare the pointers since we care if they are 'at the same spot'
                // not if they are the same value
            }
            if (quote_bits.wrapping_sub(1) & bs_bits) != 0 {
                // find out where the backspace is
                let bs_dist: u32 = bs_bits.trailing_zeros();
                let escape_char: u8 = *src.get_kinda_unchecked(src_i + bs_dist as usize + 1);
                // we encountered backslash first. Handle backslash
                if escape_char == b'u' {
                    // move src/dst up to the start; they will be further adjusted
                    // within the unicode codepoint handling code.
                    src_i += bs_dist as usize;
                    dst_i += bs_dist as usize;
                    let Ok((o, s)) = handle_unicode_codepoint(
                        src.get_kinda_unchecked(src_i..),
                        buffer.get_kinda_unchecked_mut(dst_i..),
                    ) else {
                        return Err(Deserializer::error_c(src_i, 'u', InvalidUnicodeCodepoint));
                    };
                    if o == 0 {
                        return Err(Deserializer::error_c(src_i, 'u', InvalidUnicodeCodepoint));
                    }
                    // We moved o steps forward at the destination and 6 on the source
                    src_i += s;
                    dst_i += o;
                } else {
                    // simple 1:1 conversion. Will eat bs_dist+2 characters in input and
                    // write bs_dist+1 characters to output
                    // note this may reach beyond the part of the buffer we've actually
                    // seen. I think this is ok
                    let escape_result: u8 = *ESCAPE_MAP.get_kinda_unchecked(escape_char as usize);
                    if escape_result == 0 {
                        return Err(Deserializer::error_c(
                            src_i,
                            escape_char as char,
                            InvalidEscape,
                        ));
                    }
                    *buffer.get_kinda_unchecked_mut(dst_i + bs_dist as usize) = escape_result;
                    src_i += bs_dist as usize + 2;
                    dst_i += bs_dist as usize + 1;
                }
            } else {
                // they are the same. Since they can't co-occur, it means we encountered
                // neither.
                src_i += 16;
                dst_i += 16;
            }
        }
    }
}
