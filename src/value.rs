mod de;
mod se;
#[cfg(not(feature = "no-borrow"))]
mod borrowed;
#[cfg(feature = "no-borrow")]
mod owned;

#[cfg(not(feature = "no-borrow"))]
pub use crate::value::borrowed::*;
#[cfg(feature = "no-borrow")]
pub use crate::value::owned::*;
