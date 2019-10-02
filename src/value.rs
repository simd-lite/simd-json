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
use std::convert::TryInto;

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
    + From<&'v str>
    + From<Cow<'v, str>>
    */
    + From<String>
    + From<bool>
    + From<()>
{
    type Map;
    type Array;

    fn get(&self, k: &str) -> Option<&Self>;
    fn get_mut(&mut self, k: &str) -> Option<&mut Self>;

    fn get_idx(&self, i: usize) -> Option<&Self>;
    fn get_idx_mut(&mut self, i: usize) -> Option<&mut Self>;

    fn kind(&self) -> ValueType;

    fn is_null(&self) -> bool;

    fn as_bool(&self) -> Option<bool>;
    fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    fn as_i128(&self) -> Option<i128> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    fn is_i128(&self) -> bool {
        self.as_i128().is_some()
    }

    fn as_i64(&self) -> Option<i64>;
    fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    fn as_i32(&self) -> Option<i32> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    fn is_i32(&self) -> bool {
        self.as_i32().is_some()
    }

    fn as_i16(&self) -> Option<i16> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    fn is_i16(&self) -> bool {
        self.as_i16().is_some()
    }

    fn as_i8(&self) -> Option<i8> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    fn is_i8(&self) -> bool {
        self.as_i8().is_some()
    }

    fn as_u128(&self) -> Option<u128> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    fn is_u128(&self) -> bool {
        self.as_u128().is_some()
    }

    fn as_u64(&self) -> Option<u64> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    fn as_usize(&self) -> Option<usize> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    fn is_usize(&self) -> bool {
        self.as_usize().is_some()
    }

    fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    fn is_u32(&self) -> bool {
        self.as_u32().is_some()
    }

    fn as_u16(&self) -> Option<u16> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    fn is_u16(&self) -> bool {
        self.as_u16().is_some()
    }

    fn as_u8(&self) -> Option<u8> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    fn is_u8(&self) -> bool {
        self.as_u8().is_some()
    }

    fn as_f64(&self) -> Option<f64>;
    fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }
    fn cast_f64(&self) -> Option<f64>;

    fn as_f32(&self) -> Option<f32> {
        self.as_f64().and_then(|u| if u <= f64::from(std::f32::MAX) && u >= f64::from(std::f32::MIN) {
            Some(u as f32)
        } else {
            None
        })
    }
    fn is_f32(&self) -> bool {
        self.as_f32().is_some()
    }

    #[deprecated(
        since = "0.1.20",
        note = "This will allocate a new string please use as_str instead"
    )]
    fn as_string(&self) -> Option<String>;
    #[deprecated(
        since = "0.1.20",
        note = "Please use is_str instead"
    )]
    fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    fn as_str(&self) -> Option<&str>;
    fn is_str(&self) -> bool {
        self.as_str().is_some()
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
