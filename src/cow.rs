//! Reexport of Cow

#[cfg(not(feature = "beef"))]
pub use std::borrow::Cow;

#[cfg(feature = "beef")]
pub use beef::lean::Cow;
