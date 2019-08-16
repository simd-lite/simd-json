
#[macro_export]
macro_rules! process_32_bytes {
    () => {
        #[inline(always)]
        unsafe fn process_32_bytes(&mut self, _string: &mut &[u8], _len: &mut usize, _idx: &mut usize) -> io::Result < () > {
            Ok(())
        }
    };
}

#[macro_export]
macro_rules! process_16_bytes {
    () => {
        #[inline(always)]
        unsafe fn process_16_bytes(&mut self, string: &mut &[u8], len: &mut usize, idx: &mut usize) -> io::Result<()> {
            #[cfg_attr(not(feature = "no-inline"), inline(always))]
            unsafe fn __neon_movemask(input: uint8x16_t) -> u16 {
                let bit_mask = uint8x16_t::new(
                    0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                    0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80
                );
                let minput: uint8x16_t = vandq_u8(input, bit_mask);
                let tmp: uint8x16_t = vpaddq_u8(minput, minput);
                let tmp = vpaddq_u8(tmp, tmp);
                let tmp = vpaddq_u8(tmp, tmp);

                vgetq_lane_u16(vreinterpretq_u16_u8(tmp), 0)
            }

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
                let quote_bits = __neon_movemask(vorrq_u8(bs_or_quote, in_range));
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
    };
}