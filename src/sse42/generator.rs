#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::value::generator::ESCAPED;
use std::io;

#[inline(always)]
pub unsafe fn write_str_simd<W>(
    writer: &mut W,
    string: &mut &[u8],
    len: &mut usize,
    idx: &mut usize,
) -> io::Result<()>
where
    W: std::io::Write,
{
    let zero = _mm_set1_epi8(0);
    let lower_quote_range = _mm_set1_epi8(0x1F as i8);
    let quote = _mm_set1_epi8(b'"' as i8);
    let backslash = _mm_set1_epi8(b'\\' as i8);
    while *len - *idx > 16 {
        // Load 16 bytes of data;
        #[allow(clippy::cast_ptr_alignment)]
        let data: __m128i = _mm_loadu_si128(string.as_ptr().add(*idx) as *const __m128i);
        // Test the data against being backslash and quote.
        let bs_or_quote =
            _mm_or_si128(_mm_cmpeq_epi8(data, backslash), _mm_cmpeq_epi8(data, quote));
        // Now mask the data with the quote range (0x1F).
        let in_quote_range = _mm_and_si128(data, lower_quote_range);
        // then test of the data is unchanged. aka: xor it with the
        // Any field that was inside the quote range it will be zero
        // now.
        let is_unchanged = _mm_xor_si128(data, in_quote_range);
        let in_range = _mm_cmpeq_epi8(is_unchanged, zero);
        let quote_bits = _mm_movemask_epi8(_mm_or_si128(bs_or_quote, in_range));
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
