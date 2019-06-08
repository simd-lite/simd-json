// This is mostly taken from json-rust's codegen
// as it seems to perform well and it makes snense to see
// if we can adopt the approach
//
// https://github.com/maciejhirsz/json-rust/blob/master/src/codegen.rs

use super::{Value, Map};
use crate::stry;
use std::ptr;
use std::io::Write;
use std::io;

//use util::print_dec;

impl Value {
    pub fn to_string(&self) -> String {
        let mut g = DumpGenerator::new();
        let _ = g.write_json(&self);
        g.consume()
    }
    pub fn to_string_pp(&self) -> String {
        let mut g = PrettyGenerator::new(2);
        let _ = g.write_json(&self);
        g.consume()
    }

    pub fn write<'writer, W>(&self, w: &mut W) -> io::Result<()> where W: 'writer + Write {
        let mut g = WriterGenerator::new(w);
        g.write_json(self)

    }
}


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

pub trait Generator {
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
    fn new_line(&mut self) -> io::Result<()> { Ok(()) }

    #[inline(always)]
    fn indent(&mut self) {}

    #[inline(always)]
    fn dedent(&mut self) {}

    #[inline(never)]
    fn write_string_complex(&mut self, string: &str, mut start: usize) -> io::Result<()> {
        stry!(self.write(string[ .. start].as_bytes()));

        for (index, ch) in string.bytes().enumerate().skip(start) {
            let escape = ESCAPED[ch as usize];
            if escape > 0 {
                stry!(self.write(string[start .. index].as_bytes()));
                stry!(self.write(&[b'\\', escape]));
                start = index + 1;
            }
            if escape == b'u' {
                stry!(write!(self.get_writer(), "{:04x}", ch));
            }
        }
        stry!(self.write(string[start ..].as_bytes()));

        self.write_char(b'"')
    }

    #[inline(always)]
    fn write_string(&mut self, string: &str) -> io::Result<()> {
        stry!(self.write_char(b'"'));

        for (index, ch) in string.bytes().enumerate() {
            if ESCAPED[ch as usize] > 0 {
                return self.write_string_complex(string, index)
            }
        }

        stry!(self.write(string.as_bytes()));
        self.write_char(b'"')
    }

    #[inline(always)]
    fn write_float(&mut self, num: f64) -> io::Result<()> {
        float_fast_print::write_f64_shortest(self.get_writer(), num).map(|_| ())
    }

    #[inline(always)]
    fn write_int(&mut self, num: i64) -> io::Result<()> {
        self.write(num.to_string().as_bytes())
    }

    #[inline(always)]
    fn write_object(&mut self, object: &Map) -> io::Result<()> {
        stry!(self.write_char(b'{'));
        let mut iter = object.iter();

        if let Some((key, value)) = iter.next() {
            self.indent();
            stry!(self.new_line());
            stry!(self.write_string(key));
            stry!(self.write_min(b": ", b':'));
            stry!(self.write_json(value));
        } else {
            stry!(self.write_char(b'}'));
            return Ok(());
        }

        for (key, value) in iter {
            stry!(self.write_char(b','));
            stry!(self.new_line());
            stry!(self.write_string(key));
            stry!(self.write_min(b": ", b':'));
            stry!(self.write_json(value));
        }

        self.dedent();
        stry!(self.new_line());
        self.write_char(b'}')
    }

    fn write_json(&mut self, json: &Value) -> io::Result<()> {
        match *json {
            Value::Null => self.write(b"null"),
            Value::String(ref string) => self.write_string(string),
            Value::I64(number) => self.write_int(number),
            Value::F64(number) => self.write_float(number),
            Value::Bool(true) => self.write(b"true"),
            Value::Bool(false) => self.write(b"false"),
            Value::Array(ref array) => {
                stry!(self.write_char(b'['));
                let mut iter = array.iter();

                if let Some(item) = iter.next() {
                    self.indent();
                    stry!(self.new_line());
                    stry!(self.write_json(item));
                } else {
                    stry!(self.write_char(b']'));
                    return Ok(());
                }

                for item in iter {
                    stry!(self.write_char(b','));
                    stry!(self.new_line());
                    stry!(self.write_json(item));
                }

                self.dedent();
                stry!(self.new_line());
                self.write_char(b']')
            },
            Value::Object(ref object) => {
                self.write_object(object)
            }
        }
    }
}

pub struct DumpGenerator {
    code: Vec<u8>,
}

impl DumpGenerator {
    pub fn new() -> Self {
        DumpGenerator {
            code: Vec::with_capacity(1024),
        }
    }

    pub fn consume(self) -> String {
        // Original strings were unicode, numbers are all ASCII,
        // therefore this is safe.
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl Generator for DumpGenerator {
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

pub struct PrettyGenerator {
    code: Vec<u8>,
    dent: u16,
    spaces_per_indent: u16,
}

impl PrettyGenerator {
    pub fn new(spaces: u16) -> Self {
        PrettyGenerator {
            code: Vec::with_capacity(1024),
            dent: 0,
            spaces_per_indent: spaces
        }
    }

    pub fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl Generator for PrettyGenerator {
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

pub struct WriterGenerator<'a, W: 'a + Write> {
    writer: &'a mut W
}

impl<'a, W> WriterGenerator<'a, W> where W: 'a + Write {
    pub fn new(writer: &'a mut W) -> Self {
        WriterGenerator {
            writer: writer
        }
    }
}

impl<'a, W> Generator for WriterGenerator<'a, W> where W: Write {
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


pub struct PrettyWriterGenerator<'a, W: 'a + Write> {
    writer: &'a mut W,
    dent: u16,
    spaces_per_indent: u16,
}

impl<'a, W> PrettyWriterGenerator<'a, W> where W: 'a + Write {
    pub fn new(writer: &'a mut W, spaces: u16) -> Self {
        PrettyWriterGenerator {
            writer: writer,
            dent: 0,
            spaces_per_indent: spaces,
        }
    }
}

impl<'a, W> Generator for PrettyWriterGenerator<'a, W> where W: Write {
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
#[inline]
fn extend_from_slice(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // We would have failed if `reserve` overflowed
        dst.set_len(dst_len + src_len);

        ptr::copy_nonoverlapping(
            src.as_ptr(),
            dst.as_mut_ptr().offset(dst_len as isize),
            src_len);
    }
}

#[cfg(test)]
mod test {
    use super::Value;
    #[test]
    fn null () {
        assert_eq!(Value::Null.to_string(), "null")
    }
    #[test]
    fn bool_true () {
        assert_eq!(Value::Bool(true).to_string(), "true")
    }
    #[test]
    fn bool_false () {
        assert_eq!(Value::Bool(false).to_string(), "false")
    }
}
