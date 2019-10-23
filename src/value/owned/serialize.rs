// This is mostly taken from json-rust's codegen
// as it seems to perform well and it makes snense to see
// if we can adopt the approach
//
// https://github.com/maciejhirsz/json-rust/blob/master/src/codegen.rs

use super::{Object, Value};
use crate::stry;
use crate::value::generator::*;
use crate::value::ValueTrait;
use std::io;
use std::io::Write;

//use util::print_dec;

impl Value {
    /// Encodes the value into it's JSON representation as a string
    #[deprecated(since = "0.1.21", note = "Please use encode instead")]
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        self.encode()
    }

    /// Encodes the value into it's JSON representation as a string
    pub fn encode(&self) -> String {
        let mut g = DumpGenerator::new();
        let _ = g.write_json(&self);
        g.consume()
    }

    /// Encodes the value into it's JSON representation as a string (pretty printed)
    #[deprecated(since = "0.1.21", note = "Please use encode instead")]
    pub fn to_string_pp(&self) -> String {
        self.encode_pp()
    }

    /// Encodes the value into it's JSON representation as a string (pretty printed)
    pub fn encode_pp(&self) -> String {
        let mut g = PrettyGenerator::new(2);
        let _ = g.write_json(&self);
        g.consume()
    }

    /// Encodes the value into it's JSON representation into a Writer
    pub fn write<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write,
    {
        let mut g = WriterGenerator::new(w);
        g.write_json(self)
    }

    /// Encodes the value into it's JSON representation into a Writer, pretty printed
    pub fn write_pp<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write,
    {
        let mut g = PrettyWriterGenerator::new(w, 2);
        g.write_json(self)
    }
}

trait Generator: BaseGenerator {
    type T: Write;
    type V: ValueTrait;

    #[inline(always)]
    fn write_object(&mut self, object: &Object) -> io::Result<()> {
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

    #[inline(always)]
    fn write_json(&mut self, json: &Value) -> io::Result<()> {
        match *json {
            Value::Null => self.write(b"null"),
            Value::String(ref string) => self.write_string(string),
            Value::I64(number) => self.write_int(number),
            Value::U64(number) => self.write_uint(number),
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
            }
            Value::Object(ref object) => self.write_object(object),
        }
    }
}

impl Generator for DumpGenerator<Value> {
    type T = Vec<u8>;
    type V = Value;
}

impl Generator for PrettyGenerator<Value> {
    type T = Vec<u8>;
    type V = Value;
}

impl<'w, W> Generator for WriterGenerator<'w, W, Value>
where
    W: Write,
{
    type T = W;
    type V = Value;
}

impl<'w, W> Generator for PrettyWriterGenerator<'w, W, Value>
where
    W: Write,
{
    type T = W;
    type V = Value;
}

#[cfg(test)]
mod test {
    use super::Value;
    #[test]
    fn null() {
        assert_eq!(Value::Null.encode(), "null")
    }
    #[test]
    fn bool_true() {
        assert_eq!(Value::Bool(true).encode(), "true")
    }
    #[test]
    fn bool_false() {
        assert_eq!(Value::Bool(false).encode(), "false")
    }
    fn assert_str(from: &str, to: &str) {
        assert_eq!(Value::String(from.into()).encode(), to)
    }
    #[test]
    fn string() {
        assert_str(r#"this is a test"#, r#""this is a test""#);
        assert_str(r#"this is a test ""#, r#""this is a test \"""#);
        assert_str(r#"this is a test """#, r#""this is a test \"\"""#);
        assert_str(
            r#"this is a test a long test that should span the 32 byte boundary"#,
            r#""this is a test a long test that should span the 32 byte boundary""#,
        );
        assert_str(
            r#"this is a test a "long" test that should span the 32 byte boundary"#,
            r#""this is a test a \"long\" test that should span the 32 byte boundary""#,
        );

        assert_str(
            r#"this is a test a \"long\" test that should span the 32 byte boundary"#,
            r#""this is a test a \\\"long\\\" test that should span the 32 byte boundary""#,
        );
    }
}
