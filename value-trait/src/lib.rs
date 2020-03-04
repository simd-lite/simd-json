//! A crate providing generalised value traits for working with
//! `JSONesque` values.

#![forbid(warnings)]
#![warn(unused_extern_crates)]
#![deny(
    clippy::all,
    clippy::result_unwrap_used,
    clippy::unnecessary_unwrap,
    clippy::pedantic
)]
// We might want to revisit inline_always
#![allow(clippy::module_name_repetitions, clippy::inline_always)]
#![deny(missing_docs)]

use std::borrow::{Borrow, Cow};
use std::convert::TryInto;
use std::fmt;
use std::hash::Hash;
use std::io::{self, Write};
use std::ops::{Index, IndexMut};

mod array;
/// Traits for serializing JSON
pub mod generator;
mod node;
mod object;

pub use array::Array;
pub use node::StaticNode;
pub use object::Object;

#[derive(Debug, Clone, Copy, PartialEq)]
/// An access error for `ValueType`
pub enum AccessError {
    /// An access attempt to a Value was made under the
    /// assumption that it is an Object - the Value however
    /// wasn't.
    NotAnObject,
    /// An access attempt to a Value was made under the
    /// assumption that it is an Array - the Value however
    /// wasn't.
    NotAnArray,
}

#[cfg_attr(tarpaulin, skip)]
impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnArray => write!(f, "The value is not an array"),
            Self::NotAnObject => write!(f, "The value is not an object"),
        }
    }
}
impl std::error::Error for AccessError {}

/// Types of JSON values
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ValueType {
    /// null
    Null,
    /// a boolean
    Bool,
    /// a signed integer type
    I64,
    #[cfg(feature = "128bit")]
    /// a 128 bit signed integer
    I128,
    /// a unsigned integer type
    U64,
    #[cfg(feature = "128bit")]
    /// a 128 bit unsiged integer
    U128,
    /// a float type
    F64,
    /// a string type
    String,
    /// an array
    Array,
    /// an object
    Object,
}

/// A Value that can be serialized and written
pub trait Writable {
    /// Encodes the value into it's JSON representation as a string
    #[must_use]
    fn encode(&self) -> String;

    /// Encodes the value into it's JSON representation as a string (pretty printed)
    #[must_use]
    fn encode_pp(&self) -> String;

    /// Encodes the value into it's JSON representation into a Writer
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error is encountered
    fn write<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write;

    /// Encodes the value into it's JSON representation into a Writer, pretty printed
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error is encountered.
    fn write_pp<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write;
}

/// Support of builder methods for traits.
pub trait Builder<'input>:
    Default
    + From<StaticNode>
    + From<i8>
    + From<i16>
    + From<i32>
    + From<i64>
    + From<u8>
    + From<u16>
    + From<u32>
    + From<u64>
    + From<f32>
    + From<f64>
    + From<bool>
    + From<()>
    + From<String>
    + From<&'input str>
    + From<Cow<'input, str>>
{
    /// Returns an empty array with a given capacity
    fn array_with_capacity(capacity: usize) -> Self;
    /// Returns an empty object with a given capacity
    fn object_with_capacity(capacity: usize) -> Self;
    /// Returns an empty array
    #[must_use]
    fn array() -> Self {
        Self::array_with_capacity(0)
    }
    /// Returns an empty object
    #[must_use]
    fn object() -> Self {
        Self::object_with_capacity(0)
    }
    /// Returns anull value
    fn null() -> Self;
}

/// The `Value` exposes common interface for values, this allows using both
/// `BorrowedValue` and `OwnedValue` nearly interchangable
pub trait Value:
    Sized
    + Index<usize>
    + PartialEq<i8>
    + PartialEq<i16>
    + PartialEq<i32>
    + PartialEq<i64>
    + PartialEq<i128>
    + PartialEq<u8>
    + PartialEq<u16>
    + PartialEq<u32>
    + PartialEq<u64>
    + PartialEq<u128>
    + PartialEq<f32>
    + PartialEq<f64>
    + PartialEq<String>
    + PartialEq<bool>
    + PartialEq<()>
{
    /// The type for Objects
    type Key: Hash + Eq;
    /// The array structure
    type Array: Array<Element = Self>;
    /// The object structure
    type Object: Object<Key = Self::Key, Element = Self>;

    /// Gets a ref to a value based on a key, returns `None` if the
    /// current Value isn't an Object or doesn't contain the key
    /// it was asked for.
    #[inline]
    #[must_use]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        self.as_object().and_then(|a| a.get(k))
    }

    /// Checks if a Value contains a given key. This will return
    /// flase if Value isn't an object  
    #[inline]
    #[must_use]
    fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        self.as_object().and_then(|a| a.get(k)).is_some()
    }

    /// Gets a ref to a value based on n index, returns `None` if the
    /// current Value isn't an Array or doesn't contain the index
    /// it was asked for.
    #[inline]
    #[must_use]
    fn get_idx(&self, i: usize) -> Option<&Self> {
        self.as_array().and_then(|a| a.get(i))
    }

    /// Returns the type of the current Valye
    #[must_use]
    fn value_type(&self) -> ValueType;

    /// returns true if the current value is null
    #[must_use]
    fn is_null(&self) -> bool;

    /// Tries to represent the value as a bool
    #[must_use]
    fn as_bool(&self) -> Option<bool>;
    /// returns true if the current value a bool
    #[inline]
    #[must_use]
    fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    /// Tries to represent the value as an i128
    #[inline]
    #[must_use]
    fn as_i128(&self) -> Option<i128> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a i128
    #[inline]
    #[must_use]
    fn is_i128(&self) -> bool {
        self.as_i128().is_some()
    }

    /// Tries to represent the value as an i64
    #[must_use]
    fn as_i64(&self) -> Option<i64>;
    /// returns true if the current value can be represented as a i64
    #[inline]
    #[must_use]
    fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    /// Tries to represent the value as an i32
    #[inline]
    #[must_use]
    fn as_i32(&self) -> Option<i32> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a i32
    #[inline]
    #[must_use]
    fn is_i32(&self) -> bool {
        self.as_i32().is_some()
    }

    /// Tries to represent the value as an i16
    #[inline]
    #[must_use]
    fn as_i16(&self) -> Option<i16> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a i16
    #[inline]
    #[must_use]
    fn is_i16(&self) -> bool {
        self.as_i16().is_some()
    }

    /// Tries to represent the value as an i8
    #[inline]
    #[must_use]
    fn as_i8(&self) -> Option<i8> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a i8
    #[inline]
    #[must_use]
    fn is_i8(&self) -> bool {
        self.as_i8().is_some()
    }

    /// Tries to represent the value as an u128
    #[inline]
    #[must_use]
    fn as_u128(&self) -> Option<u128> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a u128
    #[inline]
    #[must_use]
    fn is_u128(&self) -> bool {
        self.as_u128().is_some()
    }

    /// Tries to represent the value as an u64
    #[must_use]
    fn as_u64(&self) -> Option<u64>;

    /// returns true if the current value can be represented as a u64
    #[inline]
    #[must_use]
    fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    /// Tries to represent the value as an usize
    #[inline]
    #[must_use]
    fn as_usize(&self) -> Option<usize> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a usize
    #[inline]
    #[must_use]
    fn is_usize(&self) -> bool {
        self.as_usize().is_some()
    }

    /// Tries to represent the value as an u32
    #[inline]
    #[must_use]
    fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a u32
    #[inline]
    #[must_use]
    fn is_u32(&self) -> bool {
        self.as_u32().is_some()
    }

    /// Tries to represent the value as an u16
    #[inline]
    #[must_use]
    fn as_u16(&self) -> Option<u16> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a u16
    #[inline]
    #[must_use]
    fn is_u16(&self) -> bool {
        self.as_u16().is_some()
    }

    /// Tries to represent the value as an u8
    #[inline]
    #[must_use]
    fn as_u8(&self) -> Option<u8> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }
    /// returns true if the current value can be represented as a u8
    #[inline]
    #[must_use]
    fn is_u8(&self) -> bool {
        self.as_u8().is_some()
    }

    /// Tries to represent the value as a f64
    #[must_use]
    fn as_f64(&self) -> Option<f64>;
    /// returns true if the current value can be represented as a f64
    #[inline]
    #[must_use]
    fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }
    /// Casts the current value to a f64 if possible, this will turn integer
    /// values into floats.
    #[must_use]
    fn cast_f64(&self) -> Option<f64>;
    /// returns true if the current value can be cast into a f64
    #[inline]
    #[must_use]
    fn is_f64_castable(&self) -> bool {
        self.cast_f64().is_some()
    }

    /// Tries to represent the value as a f32
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    #[must_use]
    fn as_f32(&self) -> Option<f32> {
        self.as_f64().and_then(|u| {
            if u <= f64::from(std::f32::MAX) && u >= f64::from(std::f32::MIN) {
                // Since we check above
                Some(u as f32)
            } else {
                None
            }
        })
    }
    /// returns true if the current value can be represented as a f64
    #[inline]
    #[must_use]
    fn is_f32(&self) -> bool {
        self.as_f32().is_some()
    }

    /// Tries to represent the value as a &str
    #[must_use]
    fn as_str(&self) -> Option<&str>;
    /// returns true if the current value can be represented as a str
    #[inline]
    #[must_use]
    fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    /// Tries to represent the value as an array and returns a refference to it
    #[must_use]
    fn as_array(&self) -> Option<&Self::Array>;

    /// returns true if the current value can be represented as an array
    #[inline]
    #[must_use]
    fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Tries to represent the value as an object and returns a refference to it
    #[must_use]
    fn as_object(&self) -> Option<&Self::Object>;

    /// returns true if the current value can be represented as an object
    #[inline]
    #[must_use]
    fn is_object(&self) -> bool {
        self.as_object().is_some()
    }
}

/// Mutatability for values
pub trait Mutable: IndexMut<usize> + Value + Sized {
    /// Tries to insert into this `Value` as an `Object`.
    /// Will return an `AccessError::NotAnObject` if called
    /// on a `Value` that isn't an object - otherwise will
    /// behave the same as `HashMap::insert`
    /// # Errors
    ///
    /// Will return `Err` if `self` is not an object.
    #[inline]
    fn insert<K, V>(&mut self, k: K, v: V) -> std::result::Result<Option<Self>, AccessError>
    where
        K: Into<<Self as Value>::Key>,
        V: Into<Self>,
        <Self as Value>::Key: Hash + Eq,
    {
        self.as_object_mut()
            .ok_or(AccessError::NotAnObject)
            .map(|o| o.insert(k.into(), v.into()))
    }

    /// Tries to remove from this `Value` as an `Object`.
    /// Will return an `AccessError::NotAnObject` if called
    /// on a `Value` that isn't an object - otherwise will
    /// behave the same as `HashMap::remove`
    /// # Errors
    ///
    /// Will return `Err` if `self` is not an Object.
    #[inline]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> std::result::Result<Option<Self>, AccessError>
    where
        <Self as Value>::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        self.as_object_mut()
            .ok_or(AccessError::NotAnObject)
            .map(|o| o.remove(k))
    }

    /// Tries to push to this `Value` as an `Array`.
    /// Will return an `AccessError::NotAnArray` if called
    /// on a `Value` that isn't an `Array` - otherwise will
    /// behave the same as `Vec::push`
    /// # Errors
    ///
    /// Will return `Err` if `self` is not an array.
    #[inline]
    fn push<V>(&mut self, v: V) -> std::result::Result<(), AccessError>
    where
        V: Into<Self>,
    {
        self.as_array_mut()
            .ok_or(AccessError::NotAnArray)
            .map(|o| o.push(v.into()))
    }

    /// Tries to pop from this `Value` as an `Array`.
    /// Will return an `AccessError::NotAnArray` if called
    /// on a `Value` that isn't an `Array` - otherwise will
    /// behave the same as `Vec::pop`
    /// # Errors
    ///
    /// Will return `Err` if `self` is not an array.
    #[inline]
    fn pop(&mut self) -> std::result::Result<Option<Self>, AccessError> {
        self.as_array_mut()
            .ok_or(AccessError::NotAnArray)
            .map(Array::pop)
    }
    /// Same as `get` but returns a mutable ref instead
    //    fn get_amut(&mut self, k: &str) -> Option<&mut Self>;
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self>
    where
        <Self as Value>::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        self.as_object_mut().and_then(|m| m.get_mut(&k))
    }

    /// Same as `get_idx` but returns a mutable ref instead
    #[inline]
    fn get_idx_mut(&mut self, i: usize) -> Option<&mut Self> {
        self.as_array_mut().and_then(|a| a.get_mut(i))
    }
    /// Tries to represent the value as an array and returns a mutable refference to it
    fn as_array_mut(&mut self) -> Option<&mut <Self as Value>::Array>;
    /// Tries to represent the value as an object and returns a mutable refference to it
    fn as_object_mut(&mut self) -> Option<&mut Self::Object>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
