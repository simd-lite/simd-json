/// A tape of a parsed json, all values are extracted and validated and
/// can be used without further computation.
use std::marker::PhantomData;

/// `Tape`
pub struct Tape<'input>(Vec<Node<'input>>);

/// Tape `Node`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Node<'input> {
    /// A string, located inside the input slice
    String(u32, *const u8, PhantomData<&'input [u8]>),
    /// An `Object` with the given `size` starts here.
    /// the following values are keys and values, alternating
    /// however values can be nested and have a length thsemlfs.
    Object(usize),
    /// An array with a given size starts here. The next `size`
    /// elements belong to it - values can be nested and have a
    /// `size` of their own.
    Array(usize),
    /// A signed integer.
    I64(i64),
    /// An unsigned integer.
    U64(u64),
    /// A floating point value
    F64(f64),
    /// A boolean value
    Bool(bool),
    /// The null value
    Null,
}
