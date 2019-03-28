#[cfg(not(feature = "no-borrow"))]
mod borrowed;
mod de;
#[cfg(feature = "no-borrow")]
mod owned;
mod se;

#[cfg(not(feature = "no-borrow"))]
pub use crate::value::borrowed::*;
#[cfg(feature = "no-borrow")]
pub use crate::value::owned::*;
