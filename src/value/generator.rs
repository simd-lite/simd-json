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

use crate::*;

#[cfg(target_feature = "avx2")]
pub use crate::avx2::generator::*;

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_feature = "avx2")))]
pub use crate::sse42::generator::*;

#[cfg(target_feature = "neon")]
pub use crate::neon::generator::*;

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
            stry!(self.process_32_bytes(&mut string, &mut len, &mut idx));
            stry!(self.process_32_bytes(&mut string, &mut len, &mut idx));
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

    // 32-byte generation implementation
    process_32_bytes!();

    // 16-byte generation implementation
    process_16_bytes!();

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
