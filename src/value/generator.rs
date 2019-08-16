// This is mostly taken from json-rust's codegen
// as it seems to perform well and it makes snense to see
// if we can adopt the approach
//
// https://github.com/maciejhirsz/json-rust/blob/master/src/codegen.rs

use crate::value::ValueTrait;
use std::io;
use std::io::Write;
use std::marker::PhantomData;
use std::ptr;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(any(target_feature = "sse4.2", target_feature = "avx2"))]
use std::arch::x86_64::*;
#[cfg(target_feature = "neon")]
use crate::*;

#[cfg(target_feature = "avx2")]
const AVX2_PRESENT : bool = true;
#[cfg(not(target_feature = "avx2"))]
const AVX2_PRESENT : bool = false;
#[cfg(target_feature = "sse4.2")]
const SSE42_PRESENT : bool = true;
#[cfg(not(target_feature = "sse4.2"))]
const SSE42_PRESENT : bool = false;
#[cfg(target_feature = "neon")]
const NEON_PRESENT : bool = true;
#[cfg(not(target_feature = "neon"))]
const NEON_PRESENT : bool = false;

const QU: u8 = b'"';
const BS: u8 = b'\\';
const BB: u8 = b'b';
const TT: u8 = b't';
const NN: u8 = b'n';
const FF: u8 = b'f';
const RR: u8 = b'r';
const UU: u8 = b'u';
const __: u8 = 0;

// Look up table for characters that need escaping in a product string
static ESCAPED: [u8; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

pub trait BaseGenerator {
    type T: Write;
    fn get_writer(&mut self) -> &mut Self::T;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        self.get_writer().write_all(slice)
    }
    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.get_writer().write_all(&[ch])
    }

    fn write_min(&mut self, slice: &[u8], min: u8) -> io::Result<()>;

    #[inline(always)]
    fn new_line(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn indent(&mut self) {}

    #[inline(always)]
    fn dedent(&mut self) {}

    #[inline(never)]
    fn write_string_complex(&mut self, string: &[u8], mut start: usize) -> io::Result<()> {
        stry!(self.write(&string[..start]));

        for (index, ch) in string.iter().enumerate().skip(start) {
            let escape = ESCAPED[*ch as usize];
            if escape > 0 {
                stry!(self.write(&string[start..index]));
                stry!(self.write(&[b'\\', escape]));
                start = index + 1;
            }
            if escape == b'u' {
                stry!(write!(self.get_writer(), "{:04x}", ch));
            }
        }
        self.write(&string[start..])
    }

    #[inline(always)]
    fn write_string(&mut self, string: &str) -> io::Result<()> {
        stry!(self.write_char(b'"'));
        let mut string = string.as_bytes();
        let mut len = string.len();
        let mut idx = 0;

        unsafe {
            // Looking at the table above the lower 5 bits are entirely
            // quote characters that gives us a bitmask of 0x1f for that
            // region, only quote (`"`) and backslash (`\`) are not in
            // this range.
            if AVX2_PRESENT {
                self.process_32_bytes(&mut string, &mut len, &mut idx).unwrap();
            }
            if SSE42_PRESENT || NEON_PRESENT {
                self.process_16_bytes(&mut string, &mut len, &mut idx).unwrap();
            }
        }
        // Legacy code to handle the remainder of the code
        for (index, ch) in string.iter().enumerate() {
            if ESCAPED[*ch as usize] > 0 {
                self.write_string_complex(string, index)?;
                return self.write_char(b'"');
            }
        }
        stry!(self.write(string));
        self.write_char(b'"')
    }

    #[cfg(target_feature = "neon")]
    unsafe fn process_16_bytes(&mut self, string: &mut &[u8], len: &mut usize, idx: &mut usize) -> io::Result<()> {
        macro_rules! bit_mask {
            () => {
                uint8x16_t::new(
                    0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                    0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80
                )
            };
        }

        #[cfg_attr(not(feature = "no-inline"), inline(always))]
        unsafe fn neon_movemask(input: uint8x16_t) -> u16 {
            let minput: uint8x16_t = vandq_u8(input, bit_mask!());
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
            let quote_bits = neon_movemask(vorrq_u8(bs_or_quote, in_range));
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

    #[cfg(target_feature = "sse4.2")]
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

    #[cfg(all(not(target_feature = "sse4.2"), not(target_feature = "neon")))]
    #[inline(always)]
    unsafe fn process_16_bytes(&mut self, _string: &mut &[u8], _len: &mut usize, _idx: &mut usize) -> io::Result<()> {
        Ok(())
    }

    #[cfg(not(target_feature = "avx2"))]
    #[inline(always)]
    fn process_32_bytes(&mut self, _string: &mut &[u8], _len: &mut usize, _idx: &mut usize) -> io::Result<()> {
        Ok(())
    }

    #[cfg(target_feature = "avx2")]
    #[inline(always)]
    unsafe fn process_32_bytes(&mut self, string: &mut &[u8], len: &mut usize, idx: &mut usize) -> io::Result<()> {
        let zero = _mm256_set1_epi8(0);
        let lower_quote_range = _mm256_set1_epi8(0x1F as i8);
        let quote = _mm256_set1_epi8(b'"' as i8);
        let backslash = _mm256_set1_epi8(b'\\' as i8);
        while *len - *idx >= 32 {
            // Load 32 bytes of data;
            #[allow(clippy::cast_ptr_alignment)]
                let data: __m256i = _mm256_loadu_si256(string.as_ptr().add(*idx) as *const __m256i);
            // Test the data against being backslash and quote.
            let bs_or_quote = _mm256_or_si256(
                _mm256_cmpeq_epi8(data, backslash),
                _mm256_cmpeq_epi8(data, quote),
            );
            // Now mask the data with the quote range (0x1F).
            let in_quote_range = _mm256_and_si256(data, lower_quote_range);
            // then test of the data is unchanged. aka: xor it with the
            // Any field that was inside the quote range it will be zero
            // now.
            let is_unchanged = _mm256_xor_si256(data, in_quote_range);
            let in_range = _mm256_cmpeq_epi8(is_unchanged, zero);
            let quote_bits = _mm256_movemask_epi8(_mm256_or_si256(bs_or_quote, in_range));
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
                *idx += 32;
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn write_float(&mut self, num: f64) -> io::Result<()> {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format(num);
        self.get_writer().write_all(s.as_bytes())
    }

    #[inline(always)]
    fn write_int(&mut self, num: i64) -> io::Result<()> {
        itoa::write(self.get_writer(), num).map(|_| ())
        //self.write(num.to_string().as_bytes())
    }
}

/****** Pretty Generator ******/
pub struct DumpGenerator<VT: ValueTrait> {
    _value: PhantomData<VT>,
    code: Vec<u8>,
}

impl<VT: ValueTrait> DumpGenerator<VT> {
    pub fn new() -> Self {
        DumpGenerator {
            _value: PhantomData,
            code: Vec::with_capacity(1024),
        }
    }

    pub fn consume(self) -> String {
        // Original strings were unicode, numbers are all ASCII,
        // therefore this is safe.
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl<VT: ValueTrait> BaseGenerator for DumpGenerator<VT> {
    type T = Vec<u8>;

    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
        Ok(())
    }
    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.code.push(ch);
        Ok(())
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) -> io::Result<()> {
        self.code.push(min);
        Ok(())
    }
}

/****** Pretty Generator ******/

pub struct PrettyGenerator<V: ValueTrait> {
    code: Vec<u8>,
    dent: u16,
    spaces_per_indent: u16,
    _value: PhantomData<V>,
}

impl<V: ValueTrait> PrettyGenerator<V> {
    pub fn new(spaces: u16) -> Self {
        PrettyGenerator {
            code: Vec::with_capacity(1024),
            dent: 0,
            spaces_per_indent: spaces,
            _value: PhantomData,
        }
    }

    pub fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl<V: ValueTrait> BaseGenerator for PrettyGenerator<V> {
    type T = Vec<u8>;
    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
        Ok(())
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.code.push(ch);
        Ok(())
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
        Ok(())
    }

    fn new_line(&mut self) -> io::Result<()> {
        self.code.push(b'\n');
        for _ in 0..(self.dent * self.spaces_per_indent) {
            self.code.push(b' ');
        }
        Ok(())
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }
}

/****** Writer Generator ******/

pub struct WriterGenerator<'w, W: 'w + Write, V: ValueTrait> {
    writer: &'w mut W,
    _value: PhantomData<V>,
}

impl<'w, W, V> WriterGenerator<'w, W, V>
where
    W: 'w + Write,
    V: ValueTrait,
{
    pub fn new(writer: &'w mut W) -> Self {
        WriterGenerator {
            writer,
            _value: PhantomData,
        }
    }
}

impl<'w, W, V> BaseGenerator for WriterGenerator<'w, W, V>
where
    W: Write,
    V: ValueTrait,
{
    type T = W;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut W {
        &mut self.writer
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) -> io::Result<()> {
        self.writer.write_all(&[min])
    }
}

/****** Pretty Writer Generator ******/

pub struct PrettyWriterGenerator<'w, W, V>
where
    W: 'w + Write,
    V: ValueTrait,
{
    writer: &'w mut W,
    dent: u16,
    spaces_per_indent: u16,
    _value: PhantomData<V>,
}

impl<'w, W, V> PrettyWriterGenerator<'w, W, V>
where
    W: 'w + Write,
    V: ValueTrait,
{
    pub fn new(writer: &'w mut W, spaces_per_indent: u16) -> Self {
        PrettyWriterGenerator {
            writer,
            dent: 0,
            spaces_per_indent,
            _value: PhantomData,
        }
    }
}

impl<'w, W, V> BaseGenerator for PrettyWriterGenerator<'w, W, V>
where
    W: Write,
    V: ValueTrait,
{
    type T = W;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut W {
        &mut self.writer
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> io::Result<()> {
        self.writer.write_all(slice)
    }

    fn new_line(&mut self) -> io::Result<()> {
        stry!(self.write_char(b'\n'));
        for _ in 0..(self.dent * self.spaces_per_indent) {
            stry!(self.write_char(b' '));
        }
        Ok(())
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }
}

// From: https://github.com/dtolnay/fastwrite/blob/master/src/lib.rs#L68
//
// LLVM is not able to lower `Vec::extend_from_slice` into a memcpy, so this
// helps eke out that last bit of performance.
#[inline(always)]
pub fn extend_from_slice(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // We would have failed if `reserve` overflowed
        dst.set_len(dst_len + src_len);

        ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr().add(dst_len), src_len);
    }
}
