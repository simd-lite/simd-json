//use std::mem;

pub use crate::error::{Error, ErrorType};
pub use crate::Deserializer;
pub use crate::Result;
pub use crate::neon::stage1::*;
pub use crate::neon::intrinsics::*;
pub use crate::neon::utf8check::*;
pub use crate::stringparse::*;

pub use crate::neon::intrinsics::*;

unsafe fn find_bs_bits_and_quote_bits(src: &[u8]) -> ParseStringHelper {
    // this can read up to 31 bytes beyond the buffer size, but we require
    // SIMDJSON_PADDING of padding
    let v0 : uint8x16_t = vld1q_u8(src.as_ptr());
    let v1 : uint8x16_t = vld1q_u8(src.as_ptr().add(16));

    let bs_mask : uint8x16_t = vmovq_n_u8('\\' as u8);
    let qt_mask : uint8x16_t = vmovq_n_u8('"' as u8);

    let bit_mask = uint8x16_t::new(0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                                   0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80);

    let cmp_bs_0 : uint8x16_t = vceqq_u8(v0, bs_mask);
    let cmp_bs_1 : uint8x16_t = vceqq_u8(v1, bs_mask);
    let cmp_qt_0 : uint8x16_t = vceqq_u8(v0, qt_mask);
    let cmp_qt_1 : uint8x16_t = vceqq_u8(v1, qt_mask);

    let cmp_bs_0 = vandq_u8(cmp_bs_0, bit_mask);
    let cmp_bs_1 = vandq_u8(cmp_bs_1, bit_mask);
    let cmp_qt_0 = vandq_u8(cmp_qt_0, bit_mask);
    let cmp_qt_1 = vandq_u8(cmp_qt_1, bit_mask);

    let sum0 : uint8x16_t = vpaddq_u8(cmp_bs_0, cmp_bs_1);
    let sum1 : uint8x16_t = vpaddq_u8(cmp_qt_0, cmp_qt_1);
    let sum0 = vpaddq_u8(sum0, sum1);
    let sum0 = vpaddq_u8(sum0, sum0);

    ParseStringHelper {
        bs_bits: vgetq_lane_u32(vreinterpretq_u32_u8(sum0), 0), // bs_bits
        quote_bits: vgetq_lane_u32(vreinterpretq_u32_u8(sum0), 1)  // quote_bits
    }
}

impl<'de> Deserializer<'de> {
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub fn parse_str_(&mut self) -> Result<&'de str> {
        // Add 1 to skip the initial "
        let idx = self.iidx + 1;
        let mut padding = [0u8; 32];
        //let mut read: usize = 0;

        // we include the terminal '"' so we know where to end
        // This is safe since we check sub's lenght in the range access above and only
        // create sub sliced form sub to `sub.len()`.

        let src: &[u8] = unsafe { &self.input.get_unchecked(idx..) };
        let mut src_i: usize = 0;
        let mut len = src_i;
        loop {
            // store to dest unconditionally - we can overwrite the bits we don't like
            // later

            let srcx = if src.len() >= src_i + 32 {
                &src[src_i..]
            } else {
                unsafe {
                    padding
                        .get_unchecked_mut(..src.len() - src_i)
                        .clone_from_slice(src.get_unchecked(src_i..));
                    &padding
                }
            };

            let ParseStringHelper { bs_bits, quote_bits } = unsafe { find_bs_bits_and_quote_bits(&srcx) };

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
                    let v = self.input.get_unchecked(idx..idx + len) as *const [u8] as *const str;
                    return Ok(&*v);
                }

                // we compare the pointers since we care if they are 'at the same spot'
                // not if they are the same value
            }
            if (quote_bits.wrapping_sub(1) & bs_bits) != 0 {
                // Move to the 'bad' character
                let bs_dist: u32 = bs_bits.trailing_zeros();
                len += bs_dist as usize;
                src_i += bs_dist as usize;
                break;
            } else {
                // they are the same. Since they can't co-occur, it means we encountered
                // neither.
                src_i += 32;
                len += 32;
            }
        }

        let mut dst_i: usize = 0;
        let dst: &mut [u8] = self.strings.as_mut_slice();

        loop {
            let srcx = if src.len() >= src_i + 32 {
                &src[src_i..]
            } else {
                unsafe {
                    padding
                        .get_unchecked_mut(..src.len() - src_i)
                        .clone_from_slice(src.get_unchecked(src_i..));
                    &padding
                }
            };

            dst[dst_i..dst_i + 32].copy_from_slice(&srcx[..32]);

            // store to dest unconditionally - we can overwrite the bits we don't like
            // later
            let ParseStringHelper { bs_bits, quote_bits } = unsafe { find_bs_bits_and_quote_bits(&srcx) };

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
                    self.input
                        .get_unchecked_mut(idx + len..idx + len + dst_i)
                        .clone_from_slice(&self.strings.get_unchecked(..dst_i));
                    let v = self.input.get_unchecked(idx..idx + len + dst_i) as *const [u8]
                        as *const str;
                    self.str_offset += dst_i as usize;
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
                        dst.get_unchecked_mut(dst_i..)
                    }) {
                        r
                    } else {
                        return Err(self.error(ErrorType::InvlaidUnicodeCodepoint));
                    };
                    if o == 0 {
                        return Err(self.error(ErrorType::InvlaidUnicodeCodepoint));
                    };
                    // We moved o steps forword at the destiation and 6 on the source
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
                        return Err(self.error(ErrorType::InvalidEscape));
                    }
                    unsafe {
                        *dst.get_unchecked_mut(dst_i + bs_dist as usize) = escape_result;
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