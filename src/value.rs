mod de;
mod se;
use crate::numberparse::Number;
use crate::scalemap::ScaleMap;

pub type Map<'a> = ScaleMap<&'a str, Value<'a>>;

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Number(Number),
    String(&'a str),
    Array(Vec<Value<'a>>),
    Map(Map<'a>),
}
