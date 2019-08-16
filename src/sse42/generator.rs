
#[macro_export]
macro_rules! process_32_bytes {
    () => {
        #[inline(always)]
        unsafe fn process_32_bytes(&mut self, _string: &mut &[u8], _len: &mut usize, _idx: &mut usize) -> io::Result<()> {
            Ok(())
        }
    }
}

#[macro_export]
#[cfg(target_feature = "sse4.2")]
macro_rules! process_16_bytes {
    () => {
        #[inline(always)]
        unsafe fn process_16_bytes(&mut self, string: &mut &[u8], len: &mut usize, idx: &mut usize) -> io::Result<()> {
            // The case where we have a 16+ byte block
            // we repeate the same logic as above but with
            // only 16 bytes
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
                    stry!(self.get_writer().write_all(&string[0..*idx + quote_dist]));
                    let ch = string[*idx + quote_dist];
                    match ESCAPED[ch as usize] {
                        b'u' => stry!(write!(self.get_writer(), "\\u{:04x}", ch)),

                        escape => stry!(self.write(&[b'\\', escape])),
                    };
                    *string = &string[*idx + quote_dist + 1..];
                    *idx = 0;
                    *len = string.len();
                } else {
                    *idx += 16;
                }
            }
            stry!(self.get_writer().write_all(&string[0..*idx]));
            *string = &string[*idx..];
            Ok(())
        }
    }
}

#[macro_export]
#[cfg(not(target_feature = "sse4.2"))]
macro_rules! process_16_bytes {
    () => {
        #[inline(always)]
        unsafe fn process_16_bytes(&mut self, _string: &mut &[u8], _len: &mut usize, _idx: &mut usize) -> io::Result<()> {
            Ok(())
        }
    }
}