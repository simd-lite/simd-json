use crate::value::generator::ESCAPED;
use std::io;
use crate::neon::intrinsics::*;
use crate::neon::stage1::neon_movemask;

#[inline(always)]
pub unsafe fn write_str_simd<W>(writer: &mut W, string: &mut &[u8], len: &mut usize, idx: &mut usize) -> io::Result<()> where W: std::io::Write {
    // The case where we have a 16+ byte block
    // we repeate the same logic as above but with
    // only 16 bytes
    let zero = vdupq_n_u8(0);
    let lower_quote_range = vdupq_n_u8(0x1F);
    let quote = vdupq_n_u8(b'"');
    let backslash = vdupq_n_u8(b'\\');
    while *len - *idx > 16 {
        // Load 16 bytes of data;
        let data: uint8x16_t = vld1q_u8(string.as_ptr().add(*idx));
        // Test the data against being backslash and quote.
        let bs_or_quote =
            vorrq_u8(vceqq_u8(data, backslash), vceqq_u8(data, quote));
        // Now mask the data with the quote range (0x1F).
        let in_quote_range = vandq_u8(data, lower_quote_range);
        // then test of the data is unchanged. aka: xor it with the
        // Any field that was inside the quote range it will be zero
        // now.
        let is_unchanged = vxorrq_u8(data, in_quote_range);
        let in_range = vceqq_u8(is_unchanged, zero);
        let quote_bits = neon_movemask(vorrq_u8(bs_or_quote, in_range));
        if quote_bits != 0 {
            let quote_dist = quote_bits.trailing_zeros() as usize;
            stry!(writer.write_all(&string[0..*idx + quote_dist]));
            let ch = string[*idx + quote_dist];
            match ESCAPED[ch as usize] {
                b'u' => stry!(write!(writer, "\\u{:04x}", ch)),

                escape => stry!(writer.write_all(&[b'\\', escape])),
            };
            *string = &string[*idx + quote_dist + 1..];
            *idx = 0;
            *len = string.len();
        } else {
            *idx += 16;
        }
    }
    stry!(writer.write_all(&string[0..*idx]));
    *string = &string[*idx..];
    Ok(())
}
