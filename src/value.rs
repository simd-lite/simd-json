/// This module holds the two dom implementations we use. We distingush between
/// owned and borrowed. The difference being is that the borrowed value will
/// use `&str` as its string type, refferencing the input, while owned will
/// allocate a new String for each value.
///
/// Note that since json strings allow for for escape sequences the borrowed
/// value does not impement zero copy parsing, it does however not allocate
/// new memory for strings.
///
/// This differs notably from serds zero copy implementation as, unlike serde,
/// we do not require prior knowledge sbout string comtent to to take advantage
/// of it.
pub mod borrowed;
pub(crate) mod generator;
pub mod owned;

pub use self::borrowed::{to_value as to_borrowed_value, Value as BorrowedValue};
pub use self::owned::{to_value as to_owned_value, Value as OwnedValue};

#[derive(Copy, Clone, Debug)]
pub enum ValueType {
    Null,
    Bool,
    I64,
    F64,
    String,
    Array,
    Object,
}

pub trait ValueTrait:
    From<i8>
    + From<i16>
    + From<i32>
    + From<i64>
    + From<u8>
    + From<u16>
    + From<u32>
    + From<u64>
    + From<f32>
    + From<f64>
    /* Silly rust ... I got no idea how to do those :/
    + From<&'v i8>
    + From<&'v i16>
    + From<&'v i32>
    + From<&'v i64>
    + From<&'v u8>
    + From<&'v u16>
    + From<&'v u32>
    + From<&'v u64>
    + From<&'v f32>
    + From<&'v f64>
    */
    + From<String>
    + From<bool>
    + From<()>
{
    type Map;
    type Array;

    fn get(&self, k: &str) -> Option<&Self>;
    fn get_mut(&mut self, k: &str) -> Option<&mut Self>;

    fn kind(&self) -> ValueType;

    fn is_null(&self) -> bool;

    fn as_bool(&self) -> Option<bool>;
    fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    fn as_i64(&self) -> Option<i64>;
    fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    fn as_u64(&self) -> Option<u64> {
        self.as_i64().map(|u| u as u64)
    }
    fn is_u64(&self) -> bool {
        self.as_i64().is_some()
    }

    fn as_f64(&self) -> Option<f64>;
    fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }
    fn cast_f64(&self) -> Option<f64>;

    fn as_string(&self) -> Option<String>;
    fn is_string(&self) -> bool {
        self.as_string().is_some()
    }

    fn is_array(&self) -> bool {
        self.as_array().is_some()
    }
    fn as_array(&self) -> Option<&Self::Array>;
    fn as_array_mut(&mut self) -> Option<&mut Self::Array>;

    fn is_object(&self) -> bool {
        self.as_object().is_some()
    }
    fn as_object(&self) -> Option<&Self::Map>;
    fn as_object_mut(&mut self) -> Option<&mut Self::Map>;
}
