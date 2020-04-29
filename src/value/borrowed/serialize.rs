// This is mostly taken from json-rust's codegen
// as it seems to perform well and it makes snense to see
// if we can adopt the approach
//
// https://github.com/maciejhirsz/json-rust/blob/master/src/codegen.rs

use super::{Object, Value};
use crate::prelude::*;
use crate::stry;
use crate::StaticNode;
use std::io;
use std::io::Write;
use value_trait::generator::{
    BaseGenerator, DumpGenerator, PrettyGenerator, PrettyWriterGenerator, WriterGenerator,
};

//use util::print_dec;

impl<'value> Writable for Value<'value> {
    #[inline]
    fn encode(&self) -> String {
        let mut g = DumpGenerator::new();
        let _ = g.write_json(&self);
        g.consume()
    }

    #[inline]
    fn encode_pp(&self) -> String {
        let mut g = PrettyGenerator::new(2);
        let _ = g.write_json(&self);
        g.consume()
    }

    #[inline]
    fn write<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write,
    {
        let mut g = WriterGenerator::new(w);
        g.write_json(self)
    }

    #[inline]
    fn write_pp<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write,
    {
        let mut g = PrettyWriterGenerator::new(w, 2);
        g.write_json(self)
    }
}

trait Generator: BaseGenerator {
    type T: Write;

    #[inline(always)]
    fn write_object(&mut self, object: &Object) -> io::Result<()> {
        if object.is_empty() {
            self.write(b"{}")
        } else {
            let mut iter = object.iter();
            self.write(b"{")
                .and_then(|_| {
                    // We know this exists since it's not empty
                    let (key, value) = iter.next().unwrap();
                    self.indent();
                    self.new_line()
                        .and_then(|_| self.write_simple_string(key))
                        .and_then(|_| self.write_min(b": ", b':'))
                        .and_then(|_| self.write_json(value))
                })
                .and_then(|_| {
                    for (key, value) in iter {
                        stry!(self
                            .write(b",")
                            .and_then(|_| self.new_line())
                            .and_then(|_| self.write_simple_string(key))
                            .and_then(|_| self.write_min(b": ", b':'))
                            .and_then(|_| self.write_json(value)));
                    }
                    self.dedent();
                    self.new_line().and_then(|_| self.write(b"}"))
                })
        }
    }

    #[inline(always)]
    fn write_json(&mut self, json: &Value) -> io::Result<()> {
        match *json {
            Value::Static(StaticNode::Null) => self.write(b"null"),
            Value::Static(StaticNode::I64(number)) => self.write_int(number),
            #[cfg(feature = "128bit")]
            Value::Static(StaticNode::I128(number)) => self.write_int(number),
            Value::Static(StaticNode::U64(number)) => self.write_int(number),
            #[cfg(feature = "128bit")]
            Value::Static(StaticNode::U128(number)) => self.write_int(number),
            Value::Static(StaticNode::F64(number)) => self.write_float(number),
            Value::Static(StaticNode::Bool(true)) => self.write(b"true"),
            Value::Static(StaticNode::Bool(false)) => self.write(b"false"),
            Value::String(ref string) => self.write_string(string),
            Value::Array(ref array) => {
                if array.is_empty() {
                    self.write(b"[]")
                } else {
                    let mut iter = <[Value]>::iter(array);
                    // We know we have one item

                    let item = iter.next().unwrap();
                    self.write(b"[")
                        .and_then(|_| {
                            self.indent();
                            self.new_line().and_then(|_| self.write_json(item))
                        })
                        .and_then(|_| {
                            for item in iter {
                                stry!(self
                                    .write(b",")
                                    .and_then(|_| self.new_line())
                                    .and_then(|_| self.write_json(item)));
                            }

                            self.dedent();
                            self.new_line().and_then(|_| self.write(b"]"))
                        })
                }
            }
            Value::Object(ref object) => self.write_object(object),
        }
    }
}

trait FastGenerator: BaseGenerator {
    type T: Write;

    #[inline(always)]
    fn write_object(&mut self, object: &Object) -> io::Result<()> {
        if object.is_empty() {
            self.write(b"{}")
        } else {
            let mut iter = object.iter();
            self.write(b"{")
                .and_then(|_| {
                    // We know this exists since it's not empty
                    let (key, value) = iter.next().unwrap();
                    self.write_simple_string(key)
                        .and_then(|_| self.write(b":"))
                        .and_then(|_| self.write_json(value))
                })
                .and_then(|_| {
                    for (key, value) in iter {
                        stry!(self
                            .write(b",")
                            .and_then(|_| self.write_simple_string(key))
                            .and_then(|_| self.write(b":"))
                            .and_then(|_| self.write_json(value)));
                    }
                    self.write(b"}")
                })
        }
    }

    #[inline(always)]
    fn write_json(&mut self, json: &Value) -> io::Result<()> {
        match *json {
            Value::Static(StaticNode::Null) => self.write(b"null"),
            Value::Static(StaticNode::I64(number)) => self.write_int(number),
            #[cfg(feature = "128bit")]
            Value::Static(StaticNode::I128(number)) => self.write_int(number),
            Value::Static(StaticNode::U64(number)) => self.write_int(number),
            #[cfg(feature = "128bit")]
            Value::Static(StaticNode::U128(number)) => self.write_int(number),
            Value::Static(StaticNode::F64(number)) => self.write_float(number),
            Value::Static(StaticNode::Bool(true)) => self.write(b"true"),
            Value::Static(StaticNode::Bool(false)) => self.write(b"false"),
            Value::String(ref string) => self.write_string(string),
            Value::Array(ref array) => {
                if array.is_empty() {
                    self.write(b"[]")
                } else {
                    let mut iter = <[Value]>::iter(array);
                    // We know we have one item
                    let item = iter.next().unwrap();

                    self.write(b"[")
                        .and_then(|_| self.write_json(item))
                        .and_then(|_| {
                            for item in iter {
                                stry!(self.write(b",").and_then(|_| self.write_json(item)))
                            }
                            self.write(b"]")
                        })
                }
            }
            Value::Object(ref object) => self.write_object(object),
        }
    }
}

impl<'value> FastGenerator for DumpGenerator<Value<'value>> {
    type T = Vec<u8>;
}

impl<'value> Generator for PrettyGenerator<Value<'value>> {
    type T = Vec<u8>;
}

impl<'w, 'value, W> FastGenerator for WriterGenerator<'w, W, Value<'value>>
where
    W: Write,
{
    type T = W;
}

impl<'w, 'value, W> Generator for PrettyWriterGenerator<'w, W, Value<'value>>
where
    W: Write,
{
    type T = W;
}

#[cfg(test)]
mod test {
    use super::Value;
    use crate::prelude::*;
    use crate::StaticNode;

    #[test]
    fn null() {
        assert_eq!(Value::Static(StaticNode::Null).encode(), "null")
    }
    #[test]
    fn bool_true() {
        assert_eq!(Value::Static(StaticNode::Bool(true)).encode(), "true")
    }
    #[test]
    fn bool_false() {
        assert_eq!(Value::Static(StaticNode::Bool(false)).encode(), "false")
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
