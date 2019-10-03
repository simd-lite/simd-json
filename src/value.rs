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

/// Borrowed values, using Cow's for strings using in situ parsing strategies wherever possible
pub mod borrowed;
pub(crate) mod generator;
/// Owned, lifetimeless version of the value for times when lifetimes are to be avoided
pub mod owned;
use std::convert::TryInto;

pub use self::borrowed::{to_value as to_borrowed_value, Value as BorrowedValue};
pub use self::owned::{to_value as to_owned_value, Value as OwnedValue};

/// Types of JSON values
#[derive(Copy, Clone, Debug)]
pub enum ValueType {
    /// null
    Null,
    /// a boolean
    Bool,
    /// an integer type
    I64,
    /// a float type
    F64,
    /// a string type
    String,
    /// an array
    Array,
    /// an object
    Object,
}

/// The `ValueTrait` exposes common interface for values, this allows using both
/// `BorrowedValue` and `OwnedValue` nearly interchangable
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
    /// The type for Objects
    type Object;
    /// Tye type for Arrays
    type Array;

    /// Gets a ref to a value based on a key, returns `None` if the
    /// current Value isn't an Object or doesn't contain the key
    /// it was asked for.
    fn get(&self, k: &str) -> Option<&Self>;
    /// Same as `get` but returns a mutable ref instead
    fn get_mut(&mut self, k: &str) -> Option<&mut Self>;

    /// Gets a ref to a value based on n index, returns `None` if the
    /// current Value isn't an Array or doesn't contain the index
    /// it was asked for.
    fn get_idx(&self, i: usize) -> Option<&Self>;
    /// Same as `get_idx` but returns a mutable ref instead
    fn get_idx_mut(&mut self, i: usize) -> Option<&mut Self>;

    /// Returns the type of the current Valye
    #[deprecated(
        since = "0.1.21",
        note = "please use value_type instead"
    )]
    fn kind(&self) -> ValueType {
        self.value_type()
    }

    /// Returns the type of the current Valye
    fn value_type(&self) -> ValueType;

    /// returns true if the current value is null
    fn is_null(&self) -> bool;


    /// Tries to represent the value as a bool
    fn as_bool(&self) -> Option<bool>;
    /// returns true if the current value a bool
    fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    /// Tries to represent the value as an i128
    fn as_i128(&self) -> Option<i128> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a i128
    fn is_i128(&self) -> bool {
        self.as_i128().is_some()
    }

    /// Tries to represent the value as an i64
    fn as_i64(&self) -> Option<i64>;
    /// returns true if the current value can be represented as a i64
    fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    /// Tries to represent the value as an i32
    fn as_i32(&self) -> Option<i32> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a i32
    fn is_i32(&self) -> bool {
        self.as_i32().is_some()
    }

    /// Tries to represent the value as an i16
    fn as_i16(&self) -> Option<i16> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a i16
    fn is_i16(&self) -> bool {
        self.as_i16().is_some()
    }

    /// Tries to represent the value as an i8
    fn as_i8(&self) -> Option<i8> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a i8
    fn is_i8(&self) -> bool {
        self.as_i8().is_some()
    }

    /// Tries to represent the value as an u128
    fn as_u128(&self) -> Option<u128> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a u128
    fn is_u128(&self) -> bool {
        self.as_u128().is_some()
    }

    /// Tries to represent the value as an u64
    fn as_u64(&self) -> Option<u64> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a u64
    fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    /// Tries to represent the value as an usize
    fn as_usize(&self) -> Option<usize> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a usize
    fn is_usize(&self) -> bool {
        self.as_usize().is_some()
    }

    /// Tries to represent the value as an u32
    fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a u32
    fn is_u32(&self) -> bool {
        self.as_u32().is_some()
    }

    /// Tries to represent the value as an u16
    fn as_u16(&self) -> Option<u16> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a u16
    fn is_u16(&self) -> bool {
        self.as_u16().is_some()
    }

    /// Tries to represent the value as an u8
    fn as_u8(&self) -> Option<u8> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a u8
    fn is_u8(&self) -> bool {
        self.as_u8().is_some()
    }

    /// Tries to represent the value as a f64
    fn as_f64(&self) -> Option<f64>;
    /// returns true if the current value can be represented as a f64
    fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }
    /// Casts the current value to a f64 if possible, this will turn integer
    /// values into floats.
    fn cast_f64(&self) -> Option<f64>;
    /// returns true if the current value can be cast into a f64
    fn is_f64_castable(&self) -> bool {
        self.cast_f64().is_some()
    }

    /// Tries to represent the value as a f32
    fn as_f32(&self) -> Option<f32> {
        self.as_f64().and_then(|u| if u <= f64::from(std::f32::MAX) && u >= f64::from(std::f32::MIN) {
            // Since we check above
            #[allow(clippy::cast_possible_truncation)]
            Some(u as f32)
        } else {
            None
        })
    }
    /// returns true if the current value can be represented as a f64
    fn is_f32(&self) -> bool {
        self.as_f32().is_some()
    }

    /// Tries to represent the value as a String
    #[deprecated(
        since = "0.1.20",
        note = "This will allocate a new string please use as_str instead"
    )]
    fn as_string(&self) -> Option<String>;
    /// returns true if the current value can be represented as a String
    #[deprecated(
        since = "0.1.20",
        note = "Please use is_str instead"
    )]
    fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    /// Tries to represent the value as a &str
    fn as_str(&self) -> Option<&str>;
    /// returns true if the current value can be represented as a str
    fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    /// Tries to represent the value as an array and returns a refference to it
    fn as_array(&self) -> Option<&Self::Array>;
    /// Tries to represent the value as an array and returns a mutable refference to it
    fn as_array_mut(&mut self) -> Option<&mut Self::Array>;
    /// returns true if the current value can be represented as an array
    fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Tries to represent the value as an object and returns a refference to it
    fn as_object(&self) -> Option<&Self::Object>;
    /// Tries to represent the value as an object and returns a mutable refference to it
    fn as_object_mut(&mut self) -> Option<&mut Self::Object>;
    /// returns true if the current value can be represented as an object
    fn is_object(&self) -> bool {
        self.as_object().is_some()
    }
}
