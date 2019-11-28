#![deny(warnings)]
#![cfg_attr(
    target_feature = "neon",
    feature(
        asm,
        stdsimd,
        repr_simd,
        custom_inner_attributes,
        aarch64_target_feature,
        platform_intrinsics,
        stmt_expr_attributes,
        simd_ffi,
        link_llvm_intrinsics,
        rustc_attrs,
    )
)]
#![cfg_attr(feature = "hints", feature(core_intrinsics))]
#![forbid(warnings)]
#![warn(unused_extern_crates)]
#![cfg_attr(
    feature = "cargo-clippy",
    deny(
        clippy::all,
        clippy::result_unwrap_used,
        clippy::unnecessary_unwrap,
        clippy::pedantic
    ),
    // We might want to revisit inline_always
    allow(clippy::module_name_repetitions, clippy::inline_always)
)]
#![deny(missing_docs)]

//! simdjson-rs is a rust port of the simejson c++ library. It follows
//! most of the design closely with a few exceptions to make it better
//! fit into the rust ecosystem.
//!
//! Note: by default rustc will compile for compatibility, not
//! performance, to take advantage of the simd part of simd json. You
//! have to use a native cpu target on a avx2 capable host system. An
//! example how to do this can be found in the `.cargo` directory on
//! [github](https://github.com/Licenser/simdjson-rs).
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
//! As a rule of thumb this library tries to get as close as posible
//! to the performance of the c++ implementation, but some of the
//! design decisions - such as parsing to a dom or a tape, weigh
//! ergonomics over performance. In other places Rust makes it harder
//! to achive the same level of performance.
//!
//! ## Safety
//!
//! this library uses unsafe all over the place, and while it leverages
//! quite a few test cases along with property based testing, please use
//! this library with caution.
//!
//!
//! ## Usage
//!
//! simdjson-rs offers two main entry points for usage:
//!
//! ### Values API
//!
//! The values API is a set of optimized DOM objects that allow parsed
//! json to JSON data that has no known variable structure. simdjson-rs
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
//! ### Serde Comaptible API
//!
//! ```
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

mod charutils;
#[macro_use]
mod macros;
mod error;
mod numberparse;
mod stringparse;
mod utf8check;

#[cfg(target_feature = "avx2")]
mod avx2;
#[cfg(target_feature = "avx2")]
pub use crate::avx2::deser::*;
#[cfg(target_feature = "avx2")]
use crate::avx2::stage1::{SimdInput, SIMDINPUT_LENGTH, SIMDJSON_PADDING};

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(target_feature = "avx2")
))]
mod sse42;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(target_feature = "avx2")
))]
pub use crate::sse42::deser::*;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(target_feature = "avx2")
))]
use crate::sse42::stage1::{SimdInput, SIMDINPUT_LENGTH, SIMDJSON_PADDING};

#[cfg(all(target_feature = "neon", feature = "neon"))]
mod neon;
#[cfg(all(target_feature = "neon", feature = "neon"))]
pub use crate::neon::deser::*;
#[cfg(all(target_feature = "neon", feature = "neon"))]
use crate::neon::stage1::{SimdInput, SIMDINPUT_LENGTH, SIMDJSON_PADDING};

use crate::utf8check::ProcessedUtfBytes;

mod stage2;
/// simd-json JSON-DOM value
pub mod value;

#[cfg(not(target_feature = "neon"))]
use std::mem;
use std::str;

pub use crate::error::{Error, ErrorType};
pub use crate::value::*;

/// simd-json Result type
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "known-key")]
mod known_key;
#[cfg(feature = "known-key")]
pub use known_key::{Error as KnownKeyError, KnownKey};

pub use crate::tape::{Node, StaticNode, Tape};

/// Creates a tape from the input for later consumption
pub fn to_tape<'input>(s: &'input mut [u8]) -> Result<Vec<Node<'input>>> {
    let de = stry!(Deserializer::from_slice(s));
    Ok(de.tape)
}

pub(crate) trait Stage1Parse<T> {
    fn cmp_mask_against_input(&self, m: u8) -> u64;

    fn check_utf8(&self, has_error: &mut T, previous: &mut ProcessedUtfBytes<T>);

    fn unsigned_lteq_against_input(&self, maxval: T) -> u64;

    fn find_quote_mask_and_bits(
        &self,
        odd_ends: u64,
        prev_iter_inside_quote: &mut u64,
        quote_bits: &mut u64,
        error_mask: &mut u64,
    ) -> u64;

    fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64);

    fn flatten_bits(base: &mut Vec<u32>, idx: u32, bits: u64);

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

    fn is_error_detected(has_error: T) -> bool;

    fn zero() -> T;
}

pub(crate) struct Deserializer<'de> {
    // Note: we use the 2nd part as both index and lenght since only one is ever
    // used (array / object use len) everything else uses idx
    pub(crate) tape: Vec<Node<'de>>,
    idx: usize,
}

impl<'de> Deserializer<'de> {
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn error(&self, error: ErrorType) -> Error {
        Deserializer::raw_error(0, '?', error)
    }

    fn raw_error(idx: usize, c: char, error: ErrorType) -> Error {
        Error::new(idx, c, error)
    }
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_slice(input: &'de mut [u8]) -> Result<Self> {
        // We have to pick an initial size of the structural indexes.
        // 6 is a heuristic that seems to work well for the benchmark
        // data and limit re-allocation frequency.

        let len = input.len();

        let buf_start: usize = input.as_ptr() as *const () as usize;
        let needs_relocation = (buf_start + input.len()) % page_size::get() < SIMDJSON_PADDING;

        let s1_result: std::result::Result<Vec<u32>, ErrorType> = if needs_relocation {
            let mut data: Vec<u8> = Vec::with_capacity(len + SIMDJSON_PADDING);
            unsafe {
                data.set_len(len + 1);
                data.as_mut_slice()
                    .get_unchecked_mut(0..len)
                    .clone_from_slice(input);
                *(data.get_unchecked_mut(len)) = 0;
                data.set_len(len);
                Deserializer::find_structural_bits(&data)
            }
        } else {
            unsafe { Deserializer::find_structural_bits(input) }
        };
        let structural_indexes = match s1_result {
            Ok(i) => i,
            Err(t) => {
                return Err(Error::generic(t));
            }
        };

        let tape = Deserializer::build_tape(input, &structural_indexes)?;

        Ok(Deserializer { tape, idx: 0 })
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn skip(&mut self) {
        self.idx += 1;
    }

    // pull out the check so we don't need to
    // stry every time
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn next_(&mut self) -> Node<'de> {
        unsafe {
            self.idx += 1;
            *self.tape.get_unchecked(self.idx)
        }
    }

    //#[inline(never)]
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) unsafe fn find_structural_bits(
        input: &[u8],
    ) -> std::result::Result<Vec<u32>, ErrorType> {
        let len = input.len();
        // 6 is a heuristic number to estimate it turns out a rate of 1/6 structural caracters lears
        // almost never to relocations.
        let mut structural_indexes = Vec::with_capacity(len / 6);
        structural_indexes.push(0); // push extra root element

        let mut has_error = SimdInput::zero();
        let mut previous = ProcessedUtfBytes::default();
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
            input.check_utf8(&mut has_error, &mut previous);
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

            input.check_utf8(&mut has_error, &mut previous);

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

        if SimdInput::is_error_detected(has_error) {
            Err(ErrorType::InvalidUTF8)
        } else {
            Ok(structural_indexes)
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unnecessary_operation, clippy::non_ascii_literal)]
    use super::serde::from_slice;
    use super::{
        deserialize, owned::to_value, owned::Object, owned::Value, to_borrowed_value,
        to_owned_value, BorrowedValue, Deserializer, OwnedValue,
    };
    use crate::tape::*;
    use halfbrown::HashMap;
    use proptest::prelude::*;
    use serde::Deserialize;
    use serde_json;

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
        assert_eq!(dbg!(to_value(&mut d1)), Ok(Value::from(h)));
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

    // How much do we care about this, it's within the same range and
    // based on floating point math inprecisions during parsing.
    // Is this a real issue worth improving?
    #[test]
    fn silly_float1() {
        let v = Value::from(3.090_144_804_232_201_7e305);
        let s = v.encode();
        dbg!(&s);
        let mut bytes = s.as_bytes().to_vec();
        let parsed = to_owned_value(&mut bytes).expect("failed to parse gernated float");
        assert_eq!(v, parsed);
    }

    #[test]
    #[ignore]
    fn silly_float2() {
        let v = Value::from(-6.990_585_694_841_803e305);
        let s = v.encode();
        dbg!(&s);
        let mut bytes = s.as_bytes().to_vec();
        let parsed = to_owned_value(&mut bytes).expect("failed to parse gernated float");
        assert_eq!(v, parsed);
    }

    //6.576692109929364e305
    fn arb_json() -> BoxedStrategy<String> {
        let leaf = prop_oneof![
            Just(Value::Static(StaticNode::Null)),
            any::<bool>()
                .prop_map(StaticNode::Bool)
                .prop_map(Value::Static),
            // (-1.0e306f64..1.0e306f64).prop_map(|f| json!(f)), // The float parsing of simd and serde are too different
            any::<i64>().prop_map(|i| json!(i)),
            ".*".prop_map(Value::from),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    // Take the inner strategy and make the two recursive cases.
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|v| json!(v)),
                    prop::collection::hash_map(".*", inner, 0..10).prop_map(|m| json!(m)),
                ]
            },
        )
        .prop_map(|v| serde_json::to_string(&v).expect("").to_string())
        .boxed()
    }

    fn arb_json_value() -> BoxedStrategy<Value> {
        let leaf = prop_oneof![
            Just(Value::Static(StaticNode::Null)),
            any::<bool>()
                .prop_map(StaticNode::Bool)
                .prop_map(Value::Static),
            //(-1.0e306f64..1.0e306f64).prop_map(|f| json!(f)), // damn you float!
            any::<i64>().prop_map(|i| json!(i)),
            ".*".prop_map(Value::from),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    // Take the inner strategy and make the two recursive cases.
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|v| json!(v)),
                    prop::collection::hash_map(".*", inner, 0..10).prop_map(|m| json!(m)),
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
            let mut e = encoded.clone();
            let res: OwnedValue = deserialize(&mut e).expect("can't convert");
            assert_eq!(val, res);
            let mut e = encoded.clone();
            let res: BorrowedValue = deserialize(&mut e).expect("can't convert");
            assert_eq!(val, res);
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
        fn prop_json(d in arb_json()) {
            if let Ok(v_serde) = serde_json::from_slice::<serde_json::Value>(&d.as_bytes()) {
                let mut d1 = d.clone();
                let d1 = unsafe{ d1.as_bytes_mut()};
                let v_simd_serde: serde_json::Value = from_slice(d1).expect("");
                // We add our own encoder in here.
                let mut d2 = v_simd_serde.to_string();
                let d2 = unsafe{ d2.as_bytes_mut()};
                let mut d3 = d.clone();
                let d3 = unsafe{ d3.as_bytes_mut()};
                assert_eq!(v_simd_serde, v_serde);
                let v_simd_owned = to_owned_value(d2);
                assert!(v_simd_owned.is_ok());
                let v_simd_borrowed = to_borrowed_value(d3);
                dbg!(&v_simd_borrowed);
                assert!(v_simd_borrowed.is_ok());
                assert_eq!(v_simd_owned.expect("simd-error"), super::OwnedValue::from(v_simd_borrowed.expect("simd-error")));
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
