#![deny(warnings)]
#![cfg_attr(target_feature = "neon", feature(stdsimd,))]
#![cfg_attr(feature = "hints", feature(core_intrinsics))]
#![forbid(warnings)]
#![warn(unused_extern_crates)]
#![deny(
    clippy::all,
    clippy::unwrap_used,
    clippy::unnecessary_unwrap,
    clippy::pedantic
)]
// We might want to revisit inline_always
#![allow(clippy::module_name_repetitions, clippy::inline_always)]
#![deny(missing_docs)]

//! simd-json is a rust port of the simdjson c++ library. It follows
//! most of the design closely with a few exceptions to make it better
//! fit into the rust ecosystem.
//!
//! Note: by default rustc will compile for compatibility, not
//! performance, to take advantage of the simd part of simd json. You
//! have to use a native cpu target on a avx2 capable host system. An
//! example how to do this can be found in the `.cargo` directory on
//! [github](https://github.com/simd-lite/simd-json).
//!
//! ## Goals
//!
//! the goal of the rust port of simdjson is not to create a one to
//! one copy, but to integrate the principles of the c++ library into
//! a rust library that plays well with the rust ecosystem. As such
//! we provide both compatibility with serde as well as parsing to a
//! dom to manipulate data.
//!
//! ## Performance
//!
//! As a rule of thumb this library tries to get as close as possible
//! to the performance of the c++ implementation, but some of the
//! design decisions - such as parsing to a dom or a tape, weigh
//! ergonomics over performance. In other places Rust makes it harder
//! to achieve the same level of performance.
//!
//! ## Safety
//!
//! this library uses unsafe all over the place, and while it leverages
//! quite a few test cases along with property based testing, please use
//! this library with caution.
//!
//!
//! ## Features
//!
//! simd-json.rs comes with a number of features that can be toggled,
//! the following features are intended for 'user' selection. Additional
//! features in the `Cargo.toml` exist to work around cargo limitations.
//!
//! ### `swar-number-parsing` (default)
//!
//! Enables a parsing method that will parse 8 digests at a time for
//! floats - this is a common pattern but comes as a slight perf hit
//! if all the floats have less then 8 digits.
//!
//! ### `serde_impl` (default)
//!
//! Compatibility with [serde](https://serde.rs/). This allows to use
//! [simd-json.rs](https://simd-json.rs) to deserialize serde objects
//! as well as serd compatibility of the different Value types.
//! This can be disabled if serde is not used alongside simd-json.
//!
//! ### `128bit`
//!
//! Support for signed and unsigned 128 bit integer. This feature
//! is disabled by default as 128 bit integers are rare in the wild
//! and parsing them comes as a performance penalty due to extra logic
//! and a changed memory layout.
//!
//! ### `known-key`
//!
//! The known-key feature changes hasher for the objects, from ahash
//! to fxhash, ahash is faster at hashing and provides protection
//! against DOS attacks by forcing multiple keys into a single hashing
//! bucket. fxhash on the other hand allows for repeatable hashing
//! results, that allows memorizing hashes for well know keys and saving
//! time on lookups. In workloads that are heavy at accessing some well
//! known keys this can be a performance advantage.
//!
//! ## Usage
//!
//! simd-json offers two main entry points for usage:
//!
//! ### Values API
//!
//! The values API is a set of optimized DOM objects that allow parsed
//! json to JSON data that has no known variable structure. simd-lite
//! has two versions of this:
//!
//! **Borrowed Values**
//!
//! ```
//! use simd_json;
//! let mut d = br#"{"some": ["key", "value", 2]}"#.to_vec();
//! let v: simd_json::BorrowedValue = simd_json::to_borrowed_value(&mut d).unwrap();
//! ```
//!
//! **Owned Values**
//!
//! ```
//! use simd_json;
//! let mut d = br#"{"some": ["key", "value", 2]}"#.to_vec();
//! let v: simd_json::OwnedValue = simd_json::to_owned_value(&mut d).unwrap();
//! ```
//!
//! ### Serde Compatible API
//!
//! ```ignore
//! use simd_json;
//! use serde_json::Value;
//!
//! let mut d = br#"{"some": ["key", "value", 2]}"#.to_vec();
//! let v: Value = simd_json::serde::from_slice(&mut d).unwrap();
//! ```

#[cfg(feature = "serde_impl")]
extern crate serde as serde_ext;

#[cfg(feature = "serde_impl")]
/// serde related helper functions
pub mod serde;

#[cfg(feature = "serde_impl")]
pub use crate::serde::{
    from_reader, from_slice, from_str, to_string, to_string_pretty, to_vec, to_vec_pretty,
    to_writer, to_writer_pretty,
};

/// Default trait imports;
pub mod prelude;

mod charutils;
#[macro_use]
mod macros;
mod error;
mod numberparse;
mod stringparse;
mod utf8check;

/// Reexport of Cow
pub mod cow;

#[cfg(target_feature = "avx2")]
mod avx2;
#[cfg(target_feature = "avx2")]
pub use crate::avx2::deser::*;
#[cfg(target_feature = "avx2")]
use crate::avx2::stage1::{SimdInput, SIMDINPUT_LENGTH, SIMDJSON_PADDING};

#[cfg(all(target_feature = "sse4.2", not(target_feature = "avx2")))]
mod sse42;
#[cfg(all(target_feature = "sse4.2", not(target_feature = "avx2")))]
pub use crate::sse42::deser::*;
#[cfg(all(target_feature = "sse4.2", not(target_feature = "avx2")))]
use crate::sse42::stage1::{SimdInput, SIMDINPUT_LENGTH, SIMDJSON_PADDING};

#[cfg(all(target_feature = "neon", feature = "neon"))]
mod neon;
#[cfg(all(target_feature = "neon", feature = "neon"))]
pub use crate::neon::deser::*;
#[cfg(all(target_feature = "neon", feature = "neon"))]
use crate::neon::stage1::{SimdInput, SIMDINPUT_LENGTH, SIMDJSON_PADDING};

// We import this as generics
#[cfg(all(not(any(
    target_feature = "sse4.2",
    target_feature = "avx2",
    target_feature = "neon"
))))]
mod sse42;
#[cfg(all(not(any(
    target_feature = "sse4.2",
    target_feature = "avx2",
    target_feature = "neon"
))))]
#[cfg(all(not(any(
    target_feature = "sse4.2",
    target_feature = "avx2",
    target_feature = "neon"
))))]
pub use crate::sse42::deser::*;
#[cfg(all(not(any(
    target_feature = "sse4.2",
    target_feature = "avx2",
    target_feature = "neon"
))))]
use crate::sse42::stage1::{SimdInput, SIMDINPUT_LENGTH, SIMDJSON_PADDING};

#[cfg(all(
    not(feature = "allow-non-simd"),
    not(any(
        target_feature = "sse4.2",
        target_feature = "avx2",
        target_feature = "neon"
    ))
))]
fn please_compile_with_a_simd_compatible_cpu_setting_read_the_simdjonsrs_readme() -> ! {}

use crate::utf8check::ProcessedUtfBytes;

mod stage2;
/// simd-json JSON-DOM value
pub mod value;

use std::mem;
pub use value_trait::StaticNode;

pub use crate::error::{Error, ErrorType};
pub use crate::value::*;
pub use value_trait::ValueType;

/// simd-json Result type
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "known-key")]
mod known_key;
#[cfg(feature = "known-key")]
pub use known_key::{Error as KnownKeyError, KnownKey};

pub use crate::tape::{Node, Tape};
use std::alloc::{alloc, handle_alloc_error, Layout};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// Creates a tape from the input for later consumption
/// # Errors
///
/// Will return `Err` if `s` is invalid JSON.
pub fn to_tape(s: &mut [u8]) -> Result<Vec<Node>> {
    Deserializer::from_slice(s).map(Deserializer::into_tape)
}

pub(crate) type Utf8CheckingState<T> = ProcessedUtfBytes<T>;

pub(crate) trait Stage1Parse<T> {
    fn new_utf8_checking_state() -> Utf8CheckingState<T>;

    fn compute_quote_mask(quote_bits: u64) -> u64;

    fn cmp_mask_against_input(&self, m: u8) -> u64;

    fn check_utf8(&self, state: &mut Utf8CheckingState<T>);

    fn check_utf8_errors(state: &Utf8CheckingState<T>) -> bool;

    fn unsigned_lteq_against_input(&self, maxval: T) -> u64;

    fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64);

    fn flatten_bits(base: &mut Vec<u32>, idx: u32, bits: u64);

    // return both the quote mask (which is a half-open mask that covers the first
    // quote in an unescaped quote pair and everything in the quote pair) and the
    // quote bits, which are the simple unescaped quoted bits.
    //
    // We also update the prev_iter_inside_quote value to tell the next iteration
    // whether we finished the final iteration inside a quote pair; if so, this
    // inverts our behavior of whether we're inside quotes for the next iteration.
    //
    // Note that we don't do any error checking to see if we have backslash
    // sequences outside quotes; these
    // backslash sequences (of any length) will be detected elsewhere.
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(overflowing_literals, clippy::cast_sign_loss)]
    fn find_quote_mask_and_bits(
        &self,
        odd_ends: u64,
        prev_iter_inside_quote: &mut u64,
        quote_bits: &mut u64,
        error_mask: &mut u64,
    ) -> u64 {
        unsafe {
            *quote_bits = self.cmp_mask_against_input(b'"');
            *quote_bits &= !odd_ends;
            // remove from the valid quoted region the unescaped characters.
            let mut quote_mask: u64 = Self::compute_quote_mask(*quote_bits);
            quote_mask ^= *prev_iter_inside_quote;
            // All Unicode characters may be placed within the
            // quotation marks, except for the characters that MUST be escaped:
            // quotation mark, reverse solidus, and the control characters (U+0000
            //through U+001F).
            // https://tools.ietf.org/html/rfc8259
            let unescaped: u64 = self.unsigned_lteq_against_input(Self::fill_s8(0x1F));
            *error_mask |= quote_mask & unescaped;
            // right shift of a signed value expected to be well-defined and standard
            // compliant as of C++20,
            // John Regher from Utah U. says this is fine code
            *prev_iter_inside_quote = static_cast_u64!(static_cast_i64!(quote_mask) >> 63);
            quote_mask
        }
    }

    // return a bitvector indicating where we have characters that end an odd-length
    // sequence of backslashes (and thus change the behavior of the next character
    // to follow). A even-length sequence of backslashes, and, for that matter, the
    // largest even-length prefix of our odd-length sequence of backslashes, simply
    // modify the behavior of the backslashes themselves.
    // We also update the prev_iter_ends_odd_backslash reference parameter to
    // indicate whether we end an iteration on an odd-length sequence of
    // backslashes, which modifies our subsequent search for odd-length
    // sequences of backslashes in an obvious way.
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn find_odd_backslash_sequences(&self, prev_iter_ends_odd_backslash: &mut u64) -> u64 {
        const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
        const ODD_BITS: u64 = !EVEN_BITS;

        let bs_bits: u64 = self.cmp_mask_against_input(b'\\');
        let start_edges: u64 = bs_bits & !(bs_bits << 1);
        // flip lowest if we have an odd-length run at the end of the prior
        // iteration
        let even_start_mask: u64 = EVEN_BITS ^ *prev_iter_ends_odd_backslash;
        let even_starts: u64 = start_edges & even_start_mask;
        let odd_starts: u64 = start_edges & !even_start_mask;
        let even_carries: u64 = bs_bits.wrapping_add(even_starts);

        // must record the carry-out of our odd-carries out of bit 63; this
        // indicates whether the sense of any edge going to the next iteration
        // should be flipped
        let (mut odd_carries, iter_ends_odd_backslash) = bs_bits.overflowing_add(odd_starts);

        odd_carries |= *prev_iter_ends_odd_backslash;
        // push in bit zero as a potential end
        // if we had an odd-numbered run at the
        // end of the previous iteration
        *prev_iter_ends_odd_backslash = if iter_ends_odd_backslash { 0x1 } else { 0x0 };
        let even_carry_ends: u64 = even_carries & !bs_bits;
        let odd_carry_ends: u64 = odd_carries & !bs_bits;
        let even_start_odd_end: u64 = even_carry_ends & ODD_BITS;
        let odd_start_even_end: u64 = odd_carry_ends & EVEN_BITS;
        let odd_ends: u64 = even_start_odd_end | odd_start_even_end;
        odd_ends
    }

    // return a updated structural bit vector with quoted contents cleared out and
    // pseudo-structural characters added to the mask
    // updates prev_iter_ends_pseudo_pred which tells us whether the previous
    // iteration ended on a whitespace or a structural character (which means that
    // the next iteration
    // will have a pseudo-structural character at its start)
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn finalize_structurals(
        mut structurals: u64,
        whitespace: u64,
        quote_mask: u64,
        quote_bits: u64,
        prev_iter_ends_pseudo_pred: &mut u64,
    ) -> u64 {
        // mask off anything inside quotes
        structurals &= !quote_mask;
        // add the real quote bits back into our bitmask as well, so we can
        // quickly traverse the strings we've spent all this trouble gathering
        structurals |= quote_bits;
        // Now, establish "pseudo-structural characters". These are non-whitespace
        // characters that are (a) outside quotes and (b) have a predecessor that's
        // either whitespace or a structural character. This means that subsequent
        // passes will get a chance to encounter the first character of every string
        // of non-whitespace and, if we're parsing an atom like true/false/null or a
        // number we can stop at the first whitespace or structural character
        // following it.

        // a qualified predecessor is something that can happen 1 position before an
        // psuedo-structural character
        let pseudo_pred: u64 = structurals | whitespace;

        let shifted_pseudo_pred: u64 = (pseudo_pred << 1) | *prev_iter_ends_pseudo_pred;
        *prev_iter_ends_pseudo_pred = pseudo_pred >> 63;
        let pseudo_structurals: u64 = shifted_pseudo_pred & (!whitespace) & (!quote_mask);
        structurals |= pseudo_structurals;

        // now, we've used our close quotes all we need to. So let's switch them off
        // they will be off in the quote mask and on in quote bits.
        structurals &= !(quote_bits & !quote_mask);
        structurals
    }

    fn fill_s8(n: i8) -> T;
    fn zero() -> T;
}

/// Deserializer struct to deserialize a JSON
pub struct Deserializer<'de> {
    // Note: we use the 2nd part as both index and length since only one is ever
    // used (array / object use len) everything else uses idx
    pub(crate) tape: Vec<Node<'de>>,
    idx: usize,
}

impl<'de> Deserializer<'de> {
    /// Extracts the tape from the Deserializer
    #[must_use]
    pub fn into_tape(self) -> Vec<Node<'de>> {
        self.tape
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn error(error: ErrorType) -> Error {
        Deserializer::raw_error(0, '?', error)
    }

    fn raw_error(idx: usize, c: char, error: ErrorType) -> Error {
        Error::new(idx, c, error)
    }

    /// Creates a serializer from a mutable slice of bytes
    ///
    /// # Errors
    ///
    /// Will return `Err` if `s` is invalid JSON.
    pub fn from_slice(input: &'de mut [u8]) -> Result<Self> {
        let len = input.len();

        let mut string_buffer: Vec<u8> = Vec::with_capacity(len + SIMDJSON_PADDING);
        unsafe {
            string_buffer.set_len(len + SIMDJSON_PADDING);
        };

        Deserializer::from_slice_with_buffer(input, &mut string_buffer)
    }

    /// Creates a serializer from a mutable slice of bytes using a temporary
    /// buffer for strings for them to be copied in and out if needed
    ///
    /// # Errors
    ///
    /// Will return `Err` if `s` is invalid JSON.
    pub fn from_slice_with_buffer(input: &'de mut [u8], string_buffer: &mut [u8]) -> Result<Self> {
        // We have to pick an initial size of the structural indexes.
        // 6 is a heuristic that seems to work well for the benchmark
        // data and limit re-allocation frequency.

        let len = input.len();

        // let buf_start: usize = input.as_ptr() as *const () as usize;
        // let needs_relocation = (buf_start + input.len()) % page_size::get() < SIMDJSON_PADDING;
        let mut buffer = AlignedBuf::with_capacity(len + SIMDJSON_PADDING * 2);

        Self::from_slice_with_buffers(input, &mut buffer, string_buffer)
    }

    /// Creates a serializer from a mutable slice of bytes using a temporary
    /// buffer for strings for them to be copied in and out if needed
    ///
    /// # Errors
    ///
    /// Will return `Err` if `s` is invalid JSON.
    pub fn from_slice_with_buffers(
        input: &'de mut [u8],
        input_buffer: &mut AlignedBuf,
        string_buffer: &mut [u8],
    ) -> Result<Self> {
        let len = input.len();

        if len > std::u32::MAX as usize {
            return Err(Deserializer::error(ErrorType::InputTooLarge));
        }

        if input_buffer.capacity() < len + SIMDJSON_PADDING * 2 {
            *input_buffer = AlignedBuf::with_capacity(len + SIMDJSON_PADDING * 2);
        }

        unsafe {
            input_buffer
                .as_mut_slice()
                .get_unchecked_mut(..len)
                .clone_from_slice(input);
            *(input_buffer.get_unchecked_mut(len)) = 0;
            input_buffer.set_len(len);
        };

        let s1_result: std::result::Result<Vec<u32>, ErrorType> =
            unsafe { Deserializer::find_structural_bits(&input_buffer) };

        let structural_indexes = match s1_result {
            Ok(i) => i,
            Err(t) => {
                return Err(Error::generic(t));
            }
        };

        let tape =
            Deserializer::build_tape(input, &input_buffer, string_buffer, &structural_indexes)?;

        Ok(Deserializer { tape, idx: 0 })
    }

    #[cfg(feature = "serde_impl")]
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn skip(&mut self) {
        self.idx += 1;
    }

    /// Same as next() but we pull out the check so we don't need to
    /// stry every time.
    ///
    /// # Safety
    /// Use this only if you know the next element exists!
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub unsafe fn next_(&mut self) -> Node<'de> {
        self.idx += 1;
        *self.tape.get_unchecked(self.idx)
    }

    //#[inline(never)]
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) unsafe fn find_structural_bits(
        input: &[u8],
    ) -> std::result::Result<Vec<u32>, ErrorType> {
        let len = input.len();
        // 6 is a heuristic number to estimate it turns out a rate of 1/6 structural characters
        // leads almost never to relocations.
        let mut structural_indexes = Vec::with_capacity(len / 6);
        structural_indexes.push(0); // push extra root element

        let mut state = SimdInput::new_utf8_checking_state();
        // we have padded the input out to 64 byte multiple with the remainder being
        // zeros

        // persistent state across loop
        // does the last iteration end with an odd-length sequence of backslashes?
        // either 0 or 1, but a 64-bit value
        let mut prev_iter_ends_odd_backslash: u64 = 0;
        // does the previous iteration end inside a double-quote pair?
        let mut prev_iter_inside_quote: u64 = 0;
        // either all zeros or all ones
        // does the previous iteration end on something that is a predecessor of a
        // pseudo-structural character - i.e. whitespace or a structural character
        // effectively the very first char is considered to follow "whitespace" for
        // the
        // purposes of pseudo-structural character detection so we initialize to 1
        let mut prev_iter_ends_pseudo_pred: u64 = 1;

        // structurals are persistent state across loop as we flatten them on the
        // subsequent iteration into our array pointed to be base_ptr.
        // This is harmless on the first iteration as structurals==0
        // and is done for performance reasons; we can hide some of the latency of the
        // expensive carryless multiply in the previous step with this work
        let mut structurals: u64 = 0;

        let lenminus64: usize = if len < 64 { 0 } else { len as usize - 64 };
        let mut idx: usize = 0;
        let mut error_mask: u64 = 0; // for unescaped characters within strings (ASCII code points < 0x20)

        while idx < lenminus64 {
            /*
            #ifndef _MSC_VER
              __builtin_prefetch(buf + idx + 128);
            #endif
             */
            let input = SimdInput::new(input.get_unchecked(idx as usize..));
            input.check_utf8(&mut state);
            // detect odd sequences of backslashes
            let odd_ends: u64 =
                input.find_odd_backslash_sequences(&mut prev_iter_ends_odd_backslash);

            // detect insides of quote pairs ("quote_mask") and also our quote_bits
            // themselves
            let mut quote_bits: u64 = 0;
            let quote_mask: u64 = input.find_quote_mask_and_bits(
                odd_ends,
                &mut prev_iter_inside_quote,
                &mut quote_bits,
                &mut error_mask,
            );

            // take the previous iterations structural bits, not our current iteration,
            // and flatten
            #[allow(clippy::cast_possible_truncation)]
            SimdInput::flatten_bits(&mut structural_indexes, idx as u32, structurals);

            let mut whitespace: u64 = 0;
            input.find_whitespace_and_structurals(&mut whitespace, &mut structurals);

            // fixup structurals to reflect quotes and add pseudo-structural characters
            structurals = SimdInput::finalize_structurals(
                structurals,
                whitespace,
                quote_mask,
                quote_bits,
                &mut prev_iter_ends_pseudo_pred,
            );
            idx += SIMDINPUT_LENGTH;
        }

        // we use a giant copy-paste which is ugly.
        // but otherwise the string needs to be properly padded or else we
        // risk invalidating the UTF-8 checks.
        if idx < len {
            let mut tmpbuf: [u8; SIMDINPUT_LENGTH] = [0x20; SIMDINPUT_LENGTH];
            tmpbuf
                .as_mut_ptr()
                .copy_from(input.as_ptr().add(idx), len as usize - idx);
            let input = SimdInput::new(&tmpbuf);

            input.check_utf8(&mut state);

            // detect odd sequences of backslashes
            let odd_ends: u64 =
                input.find_odd_backslash_sequences(&mut prev_iter_ends_odd_backslash);

            // detect insides of quote pairs ("quote_mask") and also our quote_bits
            // themselves
            let mut quote_bits: u64 = 0;
            let quote_mask: u64 = input.find_quote_mask_and_bits(
                odd_ends,
                &mut prev_iter_inside_quote,
                &mut quote_bits,
                &mut error_mask,
            );

            // take the previous iterations structural bits, not our current iteration,
            // and flatten
            SimdInput::flatten_bits(&mut structural_indexes, idx as u32, structurals);

            let mut whitespace: u64 = 0;
            input.find_whitespace_and_structurals(&mut whitespace, &mut structurals);

            // fixup structurals to reflect quotes and add pseudo-structural characters
            structurals = SimdInput::finalize_structurals(
                structurals,
                whitespace,
                quote_mask,
                quote_bits,
                &mut prev_iter_ends_pseudo_pred,
            );
            idx += SIMDINPUT_LENGTH;
        }
        // This test isn't in upstream, for some reason the error mask is et for then.
        if prev_iter_inside_quote != 0 {
            return Err(ErrorType::Syntax);
        }
        // finally, flatten out the remaining structurals from the last iteration
        SimdInput::flatten_bits(&mut structural_indexes, idx as u32, structurals);

        // a valid JSON file cannot have zero structural indexes - we should have
        // found something (note that we compare to 1 as we always add the root!)
        if structural_indexes.len() == 1 {
            return Err(ErrorType::EOF);
        }

        if structural_indexes.last() > Some(&(len as u32)) {
            return Err(ErrorType::InternalError);
        }

        if error_mask != 0 {
            return Err(ErrorType::Syntax);
        }

        if SimdInput::check_utf8_errors(&state) {
            Err(ErrorType::InvalidUTF8)
        } else {
            Ok(structural_indexes)
        }
    }
}

/// SIMD aligned buffer
pub struct AlignedBuf {
    inner: Vec<u8>,
}

impl AlignedBuf {
    /// Creates a new buffer that is  aligned with the simd register size
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let layout = match Layout::from_size_align(capacity, SIMDJSON_PADDING / 2) {
            Ok(layout) => layout,
            Err(_) => Self::capacity_overflow(),
        };
        if mem::size_of::<usize>() < 8 && capacity > isize::MAX as usize {
            Self::capacity_overflow()
        }
        let ptr = match unsafe { NonNull::new(alloc(layout)) } {
            Some(ptr) => ptr,
            None => handle_alloc_error(layout),
        };
        Self {
            inner: unsafe { Vec::from_raw_parts(ptr.as_ptr(), 0, capacity) },
        }
    }

    fn capacity_overflow() -> ! {
        panic!("capacity overflow");
    }
}

impl Deref for AlignedBuf {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AlignedBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unnecessary_operation, clippy::non_ascii_literal)]
    use super::{owned::Value, to_borrowed_value, to_owned_value, Deserializer};
    use crate::tape::*;
    use proptest::prelude::*;
    use value_trait::{StaticNode, Writable};

    #[test]
    fn count1() {
        let mut d = String::from("[]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        assert_eq!(simd.tape[1], Node::Array(0, 2));
    }

    #[test]
    fn count2() {
        let mut d = String::from("[1]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        assert_eq!(simd.tape[1], Node::Array(1, 3));
    }

    #[test]
    fn count3() {
        let mut d = String::from("[1,2]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        assert_eq!(simd.tape[1], Node::Array(2, 4));
    }

    #[test]
    fn count4() {
        let mut d = String::from(" [ 1 , [ 3 ] , 2 ]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        assert_eq!(simd.tape[1], Node::Array(3, 6));
        assert_eq!(simd.tape[3], Node::Array(1, 5));
    }

    #[test]
    fn count5() {
        let mut d = String::from("[[],null,null]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        assert_eq!(simd.tape[1], Node::Array(3, 5));
        assert_eq!(simd.tape[2], Node::Array(0, 3));
    }

    #[cfg(feature = "128bit")]
    #[test]
    fn odd_nuber() {
        use super::value::owned::to_value;
        use super::value::{Builder, Mutable};

        let mut d = String::from(
            r#"{"name": "max_unsafe_auto_id_timestamp", "value": -9223372036854776000}"#,
        );

        let mut d = unsafe { d.as_bytes_mut() };
        let mut o = Value::object();
        o.insert("name", "max_unsafe_auto_id_timestamp")
            .expect("failed to set key");
        o.insert("value", -9_223_372_036_854_776_000_i128)
            .expect("failed to set key");
        assert_eq!(to_value(&mut d), Ok(o));
    }

    #[cfg(feature = "128bit")]
    #[test]
    fn odd_nuber2() {
        use super::value::owned::to_value;
        use super::value::{Builder, Mutable};

        let mut d = String::from(
            r#"{"name": "max_unsafe_auto_id_timestamp", "value": 9223372036854776000}"#,
        );

        let mut d = unsafe { d.as_bytes_mut() };
        let mut o = Value::object();
        o.insert("name", "max_unsafe_auto_id_timestamp")
            .expect("failed to set key");
        o.insert("value", 9_223_372_036_854_776_000_u128)
            .expect("failed to set key");
        assert_eq!(to_value(&mut d), Ok(o));
    }
    // How much do we care about this, it's within the same range and
    // based on floating point math imprecisions during parsing.
    // Is this a real issue worth improving?
    #[test]
    fn silly_float1() {
        let v = Value::from(3.090_144_804_232_201_7e305);
        let s = v.encode();
        let mut bytes = s.as_bytes().to_vec();
        let parsed = to_owned_value(&mut bytes).expect("failed to parse generated float");
        assert_eq!(v, parsed);
    }

    #[test]
    #[ignore]
    fn silly_float2() {
        let v = Value::from(-6.990_585_694_841_803e305);
        let s = v.encode();
        let mut bytes = s.as_bytes().to_vec();
        let parsed = to_owned_value(&mut bytes).expect("failed to parse gernated float");
        assert_eq!(v, parsed);
    }
    #[cfg(not(feature = "128bit"))]
    fn arb_json_value() -> BoxedStrategy<Value> {
        let leaf = prop_oneof![
            Just(Value::Static(StaticNode::Null)),
            any::<bool>().prop_map(Value::from),
            //(-1.0e306f64..1.0e306f64).prop_map(Value::from), // damn you float!
            any::<i64>().prop_map(Value::from),
            any::<u64>().prop_map(Value::from),
            ".*".prop_map(Value::from),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    // Take the inner strategy and make the two recursive cases.
                    prop::collection::vec(inner.clone(), 0..10).prop_map(Value::from),
                    prop::collection::hash_map(".*", inner, 0..10).prop_map(Value::from),
                ]
            },
        )
        .boxed()
    }

    #[cfg(feature = "128bit")]
    fn arb_json_value() -> BoxedStrategy<Value> {
        let leaf = prop_oneof![
            Just(Value::Static(StaticNode::Null)),
            any::<bool>().prop_map(Value::from),
            //(-1.0e306f64..1.0e306f64).prop_map(Value::from), // damn you float!
            any::<i64>().prop_map(Value::from),
            any::<u64>().prop_map(Value::from),
            any::<i128>().prop_map(Value::from),
            any::<u128>().prop_map(Value::from),
            ".*".prop_map(Value::from),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    // Take the inner strategy and make the two recursive cases.
                    prop::collection::vec(inner.clone(), 0..10).prop_map(Value::from),
                    prop::collection::hash_map(".*", inner, 0..10).prop_map(Value::from),
                ]
            },
        )
        .boxed()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            // Setting both fork and timeout is redundant since timeout implies
            // fork, but both are shown for clarity.
            // Disabled for code coverage, enable to track bugs
            // fork: true,
            .. ProptestConfig::default()
        })]

        #[test]
        fn prop_json_encode_decode(val in arb_json_value()) {
            let mut encoded: Vec<u8> = Vec::new();
            let _ = val.write(&mut encoded);
            println!("{}", String::from_utf8_lossy(&encoded.clone()));
            let mut e = encoded.clone();
            let res = to_owned_value(&mut e).expect("can't convert");
            assert_eq!(val, res);
            let mut e = encoded.clone();
            let res = to_borrowed_value(&mut e).expect("can't convert");
            assert_eq!(val, res);
            #[cfg(not(feature = "128bit"))]
            { // we can't dop 128 bit w/ serde
                use crate::{deserialize, BorrowedValue, OwnedValue};
                let mut e = encoded.clone();
                let res: OwnedValue = deserialize(&mut e).expect("can't convert");
                assert_eq!(val, res);
                let mut e = encoded.clone();
                let res: BorrowedValue = deserialize(&mut e).expect("can't convert");
                assert_eq!(val, res);
            }
        }

    }
}

#[cfg(feature = "serde_impl")]
#[cfg(test)]
mod tests_serde {
    #![allow(clippy::unnecessary_operation, clippy::non_ascii_literal)]
    use super::serde::from_slice;
    use super::{owned::to_value, owned::Object, owned::Value, to_borrowed_value, to_owned_value};
    use halfbrown::HashMap;
    use proptest::prelude::*;
    use serde::Deserialize;
    use serde_json;
    use value_trait::{Builder, Mutable, StaticNode};

    #[test]
    fn empty() {
        let mut d = String::from("");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_simd = from_slice::<Value>(&mut d);
        let v_serde = serde_json::from_slice::<Value>(d);
        assert!(v_simd.is_err());
        assert!(v_serde.is_err());
    }

    #[test]
    fn bool_true() {
        let mut d = String::from("true");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };

        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(true)));
    }

    #[test]
    fn bool_false() {
        let mut d = String::from("false");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(false)));
        //assert!(false)
    }

    #[test]
    fn union() {
        let mut d = String::from("null");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::Static(StaticNode::Null)));
    }

    #[test]
    fn int() {
        let mut d = String::from("42");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(42)));
    }

    #[test]
    fn zero() {
        let mut d = String::from("0");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(0)));
    }

    #[test]
    fn one() {
        let mut d = String::from("1");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(1)));
    }

    #[test]
    fn minus_one() {
        let mut d = String::from("-1");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(-1)));
    }

    #[test]
    fn float() {
        let mut d = String::from("23.0");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(23.0)));
    }

    #[test]
    fn string() {
        let mut d = String::from(r#""snot""#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(to_value(&mut d1), Ok(Value::from("snot")));
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn lonely_quote() {
        let mut d = String::from(r#"""#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde = serde_json::from_slice::<serde_json::Value>(d).is_err();
        let v_simd = from_slice::<serde_json::Value>(&mut d).is_err();
        assert!(v_simd);
        assert!(v_serde);
    }

    #[test]
    fn lonely_quote1() {
        let mut d = String::from(r#"["]"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde = serde_json::from_slice::<serde_json::Value>(d).is_err();
        let v_simd = from_slice::<serde_json::Value>(&mut d).is_err();
        assert!(v_simd);
        assert!(v_serde);
    }
    #[test]
    fn lonely_quote2() {
        let mut d = String::from(r#"[1, "]"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde = serde_json::from_slice::<serde_json::Value>(d).is_err();
        let v_simd = from_slice::<serde_json::Value>(&mut d).is_err();
        assert!(v_simd);
        assert!(v_serde);
    }

    #[test]
    fn lonely_quote3() {
        let mut d = String::from(r#"{": 1}"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde = serde_json::from_slice::<serde_json::Value>(d).is_err();
        let v_simd = from_slice::<serde_json::Value>(&mut d).is_err();
        assert!(v_simd);
        assert!(v_serde);
    }

    #[test]
    fn empty_string() {
        let mut d = String::from(r#""""#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(to_value(&mut d1), Ok(Value::from("")));
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn empty_array() {
        let mut d = String::from(r#"[]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("parse_serde");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("parse_simd");
        assert_eq!(to_value(&mut d1), Ok(Value::Array(vec![])));
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn malformed_array() {
        let mut d = String::from(r#"[["#);
        let mut d1 = d.clone();
        let mut d2 = d.clone();
        let mut d = unsafe { d.as_bytes_mut() };
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d2 = unsafe { d2.as_bytes_mut() };
        let v_serde: Result<serde_json::Value, _> = serde_json::from_slice(d);
        let v_simd_ov = to_owned_value(&mut d);
        let v_simd_bv = to_borrowed_value(&mut d1);
        let v_simd: Result<serde_json::Value, _> = from_slice(&mut d2);
        assert!(v_simd_ov.is_err());
        assert!(v_simd_bv.is_err());
        assert!(v_simd.is_err());
        assert!(v_serde.is_err());
    }

    #[test]
    fn double_array() {
        let mut d = String::from(r#"[[]]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("parse_serde");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("parse_simd");
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![Value::Array(vec![])]))
        );
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn null_null_array() {
        let mut d = String::from(r#"[[],null,null]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("parse_serde");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("parse_simd");
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![
                Value::Array(vec![]),
                Value::Static(StaticNode::Null),
                Value::Static(StaticNode::Null),
            ]))
        );
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn one_element_array() {
        let mut d = String::from(r#"["snot"]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![Value::from("snot")]))
        );
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn two_element_array() {
        let mut d = String::from(r#"["snot", "badger"]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![
                Value::from("snot"),
                Value::from("badger")
            ]))
        );
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn list() {
        let mut d = String::from(r#"[42, 23.0, "snot badger"]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![
                Value::from(42),
                Value::from(23.0),
                Value::from("snot badger")
            ]))
        );
    }

    #[test]
    fn nested_list1() {
        let mut d = String::from(r#"[42, [23.0, "snot"], "bad", "ger"]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![
                Value::from(42),
                Value::Array(vec![Value::from(23.0), Value::from("snot")]),
                Value::from("bad"),
                Value::from("ger")
            ]))
        );

        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn nested_list2() {
        let mut d = String::from(r#"[42, [23.0, "snot"], {"bad": "ger"}]"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn utf8() {
        let mut d = String::from(r#""\u000e""#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, "\u{e}");
        // NOTE: serde is broken for this
        //assert_eq!(v_serde, "\u{e}");
        //assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn unicode() {
        let mut d = String::from(r#""¡\"""#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn odd_array() {
        let mut d = String::from("[{},null]");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![
                Value::from(Object::new()),
                Value::Static(StaticNode::Null)
            ]))
        );
    }

    #[test]
    fn min_i64() {
        let mut d = String::from(
            r#"{"name": "max_unsafe_auto_id_timestamp", "value": -9223372036854775808}"#,
        );

        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        let mut o = Value::object();
        o.insert("name", "max_unsafe_auto_id_timestamp")
            .expect("failed to set key");
        o.insert("value", -9_223_372_036_854_775_808_i64)
            .expect("failed to set key");
        assert_eq!(to_value(&mut d1), Ok(o));
    }

    #[test]
    fn map2() {
        let mut d = String::from(r#"[{"\u0000":null}]"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn null() {
        let mut d = String::from(r#"null"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(to_value(&mut d1), Ok(Value::Static(StaticNode::Null)));
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }
    #[test]
    fn null_null() {
        let mut d = String::from(r#"[null, null]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![
                Value::Static(StaticNode::Null),
                Value::Static(StaticNode::Null),
            ]))
        );
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn nested_null() {
        let mut d = String::from(r#"[[null, null]]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![Value::Array(vec![
                Value::Static(StaticNode::Null),
                Value::Static(StaticNode::Null),
            ])]))
        );

        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn nestednested_null() {
        let mut d = String::from(r#"[[[null, null]]]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![Value::Array(vec![Value::Array(vec![
                Value::Static(StaticNode::Null),
                Value::Static(StaticNode::Null),
            ])])]))
        );
    }

    #[test]
    fn odd_array2() {
        let mut d = String::from("[[\"\\u0000\\\"\"]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn odd_array3() {
        let mut d = String::from("[{\"\\u0000\\u0000\":null}]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn odd_array4() {
        let mut d = String::from("[{\"\\u0000𐀀a\":null}]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn float1() {
        let mut d = String::from("2.3250706903316115e307");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("serde_json");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    // We ignore this since serde is less percise on this test
    #[ignore]
    #[test]
    fn float2() {
        let mut d = String::from("-4.5512678569607477e306");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("serde_json");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn map0() {
        let mut d = String::from(r#"{"snot": "badger"}"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        let mut h = Object::new();
        h.insert("snot".into(), Value::from("badger"));
        assert_eq!(to_value(&mut d1), Ok(Value::from(h)));
    }

    #[test]
    fn map1() {
        let mut d = String::from(r#"{"snot": "badger", "badger": "snot"}"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        let mut h = Object::new();
        h.insert("snot".into(), Value::from("badger"));
        h.insert("badger".into(), Value::from("snot"));
        assert_eq!(to_value(&mut d1), Ok(Value::from(h)));
    }

    #[cfg(feature = "serde_impl")]
    #[test]
    fn tpl1() {
        let mut d = String::from("[-65.613616999999977, 43.420273000000009]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: (f32, f32) = serde_json::from_slice(d).expect("serde_json");
        let v_simd: (f32, f32) = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl2() {
        let mut d = String::from("[[-65.613616999999977, 43.420273000000009]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<(f32, f32)> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<(f32, f32)> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl3() {
        let mut d = String::from(
            "[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]",
        );
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<(f32, f32)> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<(f32, f32)> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }
    #[test]
    fn tpl4() {
        let mut d = String::from("[[[-65.613616999999977,43.420273000000009]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<(f32, f32)>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<(f32, f32)>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }
    #[test]
    fn tpl5() {
        let mut d = String::from("[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<(f32, f32)>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<(f32, f32)>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl6() {
        let mut d = String::from("[[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<Vec<(f32, f32)>>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<Vec<(f32, f32)>>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl7() {
        let mut d = String::from("[[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<Vec<[f32; 2]>>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<Vec<[f32; 2]>>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Obj {
        a: u64,
        b: u64,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Obj1 {
        a: Obj,
    }

    #[test]
    fn obj1() {
        let mut d = String::from(r#"{"a": 1, "b":1}"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Obj = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Obj = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn obj2() {
        let mut d =
            String::from(r#"{"a": {"a": 1, "b":1}, "b": {"a": 1, "b":1}, "c": {"a": 1, "b": 1}}"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: HashMap<String, Obj> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: HashMap<String, Obj> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn obj3() {
        let mut d = String::from(
            r#"{"c": {"a": {"a": 1, "b":1}, "b": {"a": 1, "b":1}, "c": {"a": 1, "b": 1}}}"#,
        );
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: HashMap<String, HashMap<String, Obj>> =
            serde_json::from_slice(d).expect("serde_json");
        let v_simd: HashMap<String, HashMap<String, Obj>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn obj4() {
        let mut d = String::from(r#"{"c": {"a": {"a": 1, "b":1}}}"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: HashMap<String, Obj1> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: HashMap<String, Obj1> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn vecvec() {
        let mut d = String::from("[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]], [[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<(f32, f32)>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<(f32, f32)>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn crazy_string() {
        // there is unicode in here!
        let d = "\"𐀀𐀀  𐀀𐀀0 𐀀A\\u00000A0 A \\u000b\"";
        let mut d = String::from(d);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("serde_json");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[cfg(feature = "serde_impl")]
    #[test]
    fn event() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        pub struct CitmCatalog {
            pub area_names: HashMap<String, String>,
            pub audience_sub_category_names: HashMap<String, String>,
            pub block_names: HashMap<String, String>,
            pub events: HashMap<String, Event>,
        }
        pub type Id = u32;
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        pub struct Event {
            pub description: (),
            pub id: Id,
            pub logo: Option<String>,
            pub name: String,
            pub sub_topic_ids: Vec<Id>,
            pub subject_code: (),
            pub subtitle: (),
            pub topic_ids: Vec<Id>,
        }

        let mut d = String::from(
            r#"
{
    "areaNames": {
        "205705993": "Arrière-scène central",
        "205705994": "1er balcon central",
        "205705995": "2ème balcon bergerie cour",
        "205705996": "2ème balcon bergerie jardin",
        "205705998": "1er balcon bergerie jardin",
        "205705999": "1er balcon bergerie cour",
        "205706000": "Arrière-scène jardin",
        "205706001": "Arrière-scène cour",
        "205706002": "2ème balcon jardin",
        "205706003": "2ème balcon cour",
        "205706004": "2ème Balcon central",
        "205706005": "1er balcon jardin",
        "205706006": "1er balcon cour",
        "205706007": "Orchestre central",
        "205706008": "Orchestre jardin",
        "205706009": "Orchestre cour",
        "342752287": "Zone physique secrète"
    },
    "audienceSubCategoryNames": {
        "337100890": "Abonné"
    },
    "blockNames": {},
  "events": {
    "138586341": {
      "description": null,
      "id": 138586341,
      "logo": null,
      "name": "30th Anniversary Tour",
      "subTopicIds": [
        337184269,
        337184283
      ],
      "subjectCode": null,
      "subtitle": null,
      "topicIds": [
        324846099,
        107888604
      ]
    },
    "138586345": {
      "description": null,
      "id": 138586345,
      "logo": "/images/UE0AAAAACEKo6QAAAAZDSVRN",
      "name": "Berliner Philharmoniker",
      "subTopicIds": [
        337184268,
        337184283,
        337184275
      ],
      "subjectCode": null,
      "subtitle": null,
      "topicIds": [
        324846099,
        107888604,
        324846100
      ]
    }
  }
}
"#,
        );
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: CitmCatalog = serde_json::from_slice(d).expect("serde_json");
        let v_simd: CitmCatalog = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    //6.576692109929364e305
    fn arb_json() -> BoxedStrategy<String> {
        let leaf = prop_oneof![
            Just(Value::Static(StaticNode::Null)),
            any::<bool>()
                .prop_map(StaticNode::Bool)
                .prop_map(Value::Static),
            // (-1.0e306f64..1.0e306f64).prop_map(Value::from), // The float parsing of simd and serde are too different
            any::<i64>().prop_map(Value::from),
            ".*".prop_map(Value::from),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    // Take the inner strategy and make the two recursive cases.
                    prop::collection::vec(inner.clone(), 0..10).prop_map(Value::from),
                    prop::collection::hash_map(".*", inner, 0..10).prop_map(Value::from),
                ]
            },
        )
        .prop_map(|v| serde_json::to_string(&v).expect("").to_string())
        .boxed()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            // Setting both fork and timeout is redundant since timeout implies
            // fork, but both are shown for clarity.
            // Disabled for code coverage, enable to track bugs
            // fork: true,
            .. ProptestConfig::default()
        })]

        #[test]
        fn prop_json(d in arb_json()) {
            use super::{OwnedValue, deserialize};
            if let Ok(v_serde) = serde_json::from_slice::<serde_json::Value>(&d.as_bytes()) {
                let mut d1 = d.clone();
                let d1 = unsafe{ d1.as_bytes_mut()};
                let v_simd_serde: serde_json::Value = from_slice(d1).expect("");
                // We add our own encoder in here.
                let mut d2 = v_simd_serde.to_string();
                let d2 = unsafe{ d2.as_bytes_mut()};
                let mut d3 = d.clone();
                let d3 = unsafe{ d3.as_bytes_mut()};
                let mut d4 = d.clone();
                let d4 = unsafe{ d4.as_bytes_mut()};
                assert_eq!(v_simd_serde, v_serde);
                let v_simd_owned = to_owned_value(d2).expect("to_owned_value failed");
                let v_simd_borrowed = to_borrowed_value(d3).expect("to_borrowed_value failed");
                assert_eq!(v_simd_borrowed, v_simd_owned);
                let v_deserialize: OwnedValue = deserialize(d4).expect("deserialize failed");
                assert_eq!(v_deserialize, v_simd_owned);
            }

        }

    }

    fn arb_junk() -> BoxedStrategy<Vec<u8>> {
        prop::collection::vec(any::<u8>(), 0..(1024 * 8)).boxed()
    }
    proptest! {
        #![proptest_config(ProptestConfig {
            // Setting both fork and timeout is redundant since timeout implies
            // fork, but both are shown for clarity.
            // Disabled for code coverage, enable to track bugs
            // fork: true,
            .. ProptestConfig::default()
        })]
        #[test]
        fn prop_junk(d in arb_junk()) {
            let mut d1 = d.clone();
            let mut d2 = d.clone();
            let mut d3 = d.clone();

            let _ = from_slice::<serde_json::Value>(&mut d1);
            let _ = to_borrowed_value(&mut d2);
            let _ = to_owned_value(&mut d3);

        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            // Setting both fork and timeout is redundant since timeout implies
            // fork, but both are shown for clarity.
            // Disabled for code coverage, enable to track bugs
            // fork: true,
            .. ProptestConfig::default()
        })]

        #[test]
        fn prop_string(d in "\\PC*") {
            let mut d1 = d.clone();
            let mut d1 = unsafe{ d1.as_bytes_mut()};
            let mut d2 = d.clone();
            let mut d2 = unsafe{ d2.as_bytes_mut()};
            let mut d3 = d.clone();
            let mut d3 = unsafe{ d3.as_bytes_mut()};
            let _ = from_slice::<serde_json::Value>(&mut d1);
            let _ = to_borrowed_value(&mut d2);
            let _ = to_owned_value(&mut d3);

        }
    }
}
