#[cfg(target_arch = "x86")]
use std::arch::x86::{
    __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    _mm256_storeu_si256,
};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    _mm256_storeu_si256,
};

use std::mem;

pub use crate::error::{Error, ErrorType};
use crate::stringparse::{handle_unicode_codepoint, ESCAPE_MAP};
use crate::Deserializer;
pub use crate::Result;

impl<'de> Deserializer<'de> {
    #[allow(
        clippy::if_not_else,
        mutable_transmutes,
        clippy::transmute_ptr_to_ptr,
        clippy::too_many_lines,
        clippy::cast_ptr_alignment,
        clippy::cast_possible_wrap,
        clippy::if_not_else,
        clippy::too_many_lines
    )]
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub(crate) fn parse_str_<'invoke>(
        input: &'de [u8],
        data: &'invoke [u8],
        buffer: &'invoke mut [u8],
        mut idx: usize,
    ) -> Result<&'de str> {
        use ErrorType::{InvalidEscape, InvalidUnicodeCodepoint};
        let input: &mut [u8] = unsafe { std::mem::transmute(input) };
        // Add 1 to skip the initial "
        idx += 1;
        //let mut read: usize = 0;

        // we include the terminal '"' so we know where to end
        // This is safe since we check sub's length in the range access above and only
        // create sub sliced form sub to `sub.len()`.

        let src: &[u8] = unsafe { data.get_unchecked(idx..) };
        let mut src_i: usize = 0;
        let mut len = src_i;
        loop {
            let v: __m256i = unsafe {
                _mm256_loadu_si256(src.as_ptr().add(src_i).cast::<std::arch::x86_64::__m256i>())
            };

            // store to dest unconditionally - we can overwrite the bits we don't like
            // later
            let bs_bits: u32 = unsafe {
                static_cast_u32!(_mm256_movemask_epi8(_mm256_cmpeq_epi8(
                    v,
                    _mm256_set1_epi8(b'\\' as i8)
                )))
            };
            let quote_mask = unsafe { _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'"' as i8)) };
            let quote_bits = unsafe { static_cast_u32!(_mm256_movemask_epi8(quote_mask)) };
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
                unsafe {
                    let v = input.get_unchecked(idx..idx + len) as *const [u8] as *const str;
                    return Ok(&*v);
                }

                // we compare the pointers since we care if they are 'at the same spot'
                // not if they are the same value
            }
            if (quote_bits.wrapping_sub(1) & bs_bits) == 0 {
                // they are the same. Since they can't co-occur, it means we encountered
                // neither.
                src_i += 32;
                len += 32;
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
            let v: __m256i = unsafe {
                _mm256_loadu_si256(src.as_ptr().add(src_i).cast::<std::arch::x86_64::__m256i>())
            };

            unsafe {
                _mm256_storeu_si256(
                    buffer
                        .as_mut_ptr()
                        .add(dst_i)
                        .cast::<std::arch::x86_64::__m256i>(),
                    v,
                );
            };

            // store to dest unconditionally - we can overwrite the bits we don't like
            // later
            let bs_bits: u32 = unsafe {
                static_cast_u32!(_mm256_movemask_epi8(_mm256_cmpeq_epi8(
                    v,
                    _mm256_set1_epi8(b'\\' as i8)
                )))
            };
            let quote_mask = unsafe { _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'"' as i8)) };
            let quote_bits = unsafe { static_cast_u32!(_mm256_movemask_epi8(quote_mask)) };
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
                unsafe {
                    input
                        .get_unchecked_mut(idx + len..idx + len + dst_i)
                        .clone_from_slice(buffer.get_unchecked(..dst_i));
                    let v =
                        input.get_unchecked(idx..idx + len + dst_i) as *const [u8] as *const str;
                    return Ok(&*v);
                }

                // we compare the pointers since we care if they are 'at the same spot'
                // not if they are the same value
            }
            if (quote_bits.wrapping_sub(1) & bs_bits) != 0 {
                // find out where the backspace is
                let bs_dist: u32 = bs_bits.trailing_zeros();
                let escape_char: u8 = unsafe { *src.get_unchecked(src_i + bs_dist as usize + 1) };
                // we encountered backslash first. Handle backslash
                if escape_char == b'u' {
                    // move src/dst up to the start; they will be further adjusted
                    // within the unicode codepoint handling code.
                    src_i += bs_dist as usize;
                    dst_i += bs_dist as usize;
                    let (o, s) = if let Ok(r) =
                        handle_unicode_codepoint(unsafe { src.get_unchecked(src_i..) }, unsafe {
                            buffer.get_unchecked_mut(dst_i..)
                        }) {
                        r
                    } else {
                        return Err(Self::raw_error(src_i, 'u', InvalidUnicodeCodepoint));
                    };
                    if o == 0 {
                        return Err(Self::raw_error(src_i, 'u', InvalidUnicodeCodepoint));
                    };
                    // We moved o steps forward at the destination and 6 on the source
                    src_i += s;
                    dst_i += o;
                } else {
                    // simple 1:1 conversion. Will eat bs_dist+2 characters in input and
                    // write bs_dist+1 characters to output
                    // note this may reach beyond the part of the buffer we've actually
                    // seen. I think this is ok
                    let escape_result: u8 =
                        unsafe { *ESCAPE_MAP.get_unchecked(escape_char as usize) };
                    if escape_result == 0 {
                        return Err(Self::raw_error(src_i, escape_char as char, InvalidEscape));
                    }
                    unsafe {
                        *buffer.get_unchecked_mut(dst_i + bs_dist as usize) = escape_result;
                    }
                    src_i += bs_dist as usize + 2;
                    dst_i += bs_dist as usize + 1;
                }
            } else {
                // they are the same. Since they can't co-occur, it means we encountered
                // neither.
                src_i += 32;
                dst_i += 32;
            }
        }
    }
}
