mod de;
mod se;
use crate::scalemap::ScaleMap;
use crate::numberparse::Number;

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
