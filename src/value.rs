use crate::scalemap::ScaleMap;
use crate::numberparse::Number;

pub type Map<'a> = ScaleMap<&'a str, Value<'a>>;

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Array(Vec<Value<'a>>),
    Bool(bool),
    Map(Map<'a>),
    Null,
    Number(Number),
    String(&'a str),
}
