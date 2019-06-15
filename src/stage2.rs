#![allow(dead_code)]
use crate::charutils::*;
use crate::portability::*;
use crate::stringparse::*;
use crate::{Deserializer, Error, ErrorType, Result, SIMDJSON_PADDING};
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem;

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_valid_true_atom(loc: &[u8]) -> bool {
    // TODO is this expensive?
    let mut error: u64;
    unsafe {
        //let tv: u64 = *(b"true    ".as_ptr() as *const u64);
        // this is the same:
        const TV: u64 = 0x00_00_00_00_65_75_72_74;
        const MASK4: u64 = 0x00_00_00_00_ff_ff_ff_ff;

        // TODO: does this has the same effect as:
        //   std::memcpy(&locval, loc, sizeof(uint64_t));
        #[allow(clippy::cast_ptr_alignment)]
        let locval: u64 = *(loc.as_ptr() as *const u64);

        error = (locval & MASK4) ^ TV;
        error |= u64::from(is_not_structural_or_whitespace(*loc.get_unchecked(4)));
    }
    error == 0
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_valid_false_atom(loc: &[u8]) -> bool {
    // TODO: this is ugly and probably copies data every time
    let mut error: u64;
    unsafe {
        //let fv: u64 = *(b"false   ".as_ptr() as *const u64);
        // this is the same:

        const FV: u64 = 0x00_00_00_65_73_6c_61_66;
        const MASK5: u64 = 0x00_00_00_ff_ff_ff_ff_ff;

        #[allow(clippy::cast_ptr_alignment)]
        let locval: u64 = *(loc.as_ptr() as *const u64);

        // FIXME the original code looks like this:
        // error = ((locval & mask5) ^ fv) as u32;
        // but that failes on falsy as the u32 conversion
        // will mask the error on the y so we re-write it
        // it would be interesting what the consequecnes are
        error = (locval & MASK5) ^ FV;
        error |= u64::from(is_not_structural_or_whitespace(*loc.get_unchecked(5)));
    }
    error == 0
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_valid_null_atom(loc: &[u8]) -> bool {
    // TODO is this expensive?
    let mut error: u64;
    unsafe {
        //let nv: u64 = *(b"null   ".as_ptr() as *const u64);
        // this is the same:
        const NV: u64 = 0x00_00_00_00_6c_6c_75_6e;
        const MASK4: u64 = 0x00_00_00_00_ff_ff_ff_ff;
        #[allow(clippy::cast_ptr_alignment)]
        let locval: u64 = *(loc.as_ptr() as *const u64);

        error = (locval & MASK4) ^ NV;
        error |= u64::from(is_not_structural_or_whitespace(*loc.get_unchecked(4)));
    }
    error == 0
}

#[derive(Debug)]
enum State {
    ObjectKey,
    ScopeEnd,
    MainArraySwitch,
}
#[derive(Debug)]
enum StackState {
    Start,
    Object,
    Array,
}

// We parse a string that's likely to be less then 32 characters and without any
// fancy in it like object keys
#[cfg_attr(not(feature = "no-inline"), inline(always))]
fn extract_str_short(
    data: &[u8],
    input: &mut [u8],
    offset: &mut usize,
    mut idx: usize,
) -> Result<usize> {
    idx += 1;
    let src: &[u8] = unsafe { data.get_unchecked(idx..) };
    /*
    unsafe {
        dbg!(&String::from_utf8_unchecked(src.to_vec()));
    }
    */
    // We relocateds at the beginning and ensured that we are 0 terminated
    // so we always can copy from src.
    #[allow(clippy::cast_ptr_alignment)]
    let v: __m256i =
        unsafe { _mm256_loadu_si256(src.get_unchecked(..32).as_ptr() as *const __m256i) };
    let bs_bits: u32 = unsafe {
        static_cast_u32!(_mm256_movemask_epi8(_mm256_cmpeq_epi8(
            v,
            _mm256_set1_epi8(b'\\' as i8)
        )))
    };
    let quote_mask = unsafe { _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'"' as i8)) };
    let quote_bits = unsafe { static_cast_u32!(_mm256_movemask_epi8(quote_mask)) };
    if (bs_bits.wrapping_sub(1) & quote_bits) != 0 {
        let quote_dist: u32 = trailingzeroes(u64::from(quote_bits)) as u32;
        unsafe {
            if input.len() - *offset < 32 {
                input
                    .as_mut_ptr()
                    .add(*offset)
                    .copy_from(src.as_ptr(), quote_dist as usize);
            } else {
                _mm256_storeu_si256(input.as_mut_ptr().add(*offset) as *mut __m256i, v)
            }
        }
        *offset += quote_dist as usize;

        return Ok(quote_dist as usize);
    } else {
        extract_str(data, input, offset, idx - 1)
    }
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
fn extract_str(data: &[u8], input: &mut [u8], offset: &mut usize, mut idx: usize) -> Result<usize> {
    // Add 1 to skip the initial "
    idx += 1;
    let mut padding = [0u8; 32];
    //let mut read: usize = 0;

    // we include the terminal '"' so we know where to end
    // This is safe since we check sub's lenght in the range access above and only
    // create sub sliced form sub to `sub.len()`.

    // if we don't need relocation we can write directly to the input
    // saving us to copy data to the string storage first and then
    // back tot he input.
    // We can't always do that as if we're less then 32 characters
    // behind we'll overwrite important parts of the input.
    let src: &[u8] = unsafe { &data.get_unchecked(idx..) };
    let dst: &mut [u8] = unsafe { input.get_unchecked_mut(*offset..) };

    let mut src_i: usize = 0;
    let mut dst_i: usize = 0;
    loop {
        let v: __m256i = if src.len() >= src_i + 32 {
            // This is safe since we ensure src is at least 32 wide
            #[allow(clippy::cast_ptr_alignment)]
            unsafe {
                _mm256_loadu_si256(src.as_ptr().add(src_i) as *const __m256i)
            }
        } else {
            unsafe {
                padding
                    .get_unchecked_mut(..src.len() - src_i)
                    .clone_from_slice(src.get_unchecked(src_i..));
                // This is safe since we ensure src is at least 32 wide
                #[allow(clippy::cast_ptr_alignment)]
                _mm256_loadu_si256(padding.as_ptr() as *const __m256i)
            }
        };

        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            _mm256_storeu_si256(dst.as_mut_ptr().add(dst_i) as *mut __m256i, v)
        };

        // store to dest unconditionally - we can overwrite the bits we don't like
        // later
        let bs_bits: u32 = unsafe {
            static_cast_u32!(_mm256_movemask_epi8(_mm256_cmpeq_epi8(
                v,
                _mm256_set1_epi8(b'\\' as i8)
            )))
        };
        let quote_mask = unsafe { _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'"' as i8)) };
        let quote_bits = unsafe { static_cast_u32!(_mm256_movemask_epi8(quote_mask)) };
        if (bs_bits.wrapping_sub(1) & quote_bits) != 0 {
            // we encountered quotes first. Move dst to point to quotes and exit
            // find out where the quote is...
            let quote_dist: u32 = trailingzeroes(u64::from(quote_bits)) as u32;

            ///////////////////////
            // Above, check for overflow in case someone has a crazy string (>=4GB?)
            // But only add the overflow check when the document itself exceeds 4GB
            // Currently unneeded because we refuse to parse docs larger or equal to 4GB.
            ////////////////////////

            // we advance the point, accounting for the fact that we have a NULl termination

            dst_i += quote_dist as usize;
            *offset += dst_i as usize;

            return Ok(dst_i);

            // we compare the pointers since we care if they are 'at the same spot'
            // not if they are the same value
        }
        if (quote_bits.wrapping_sub(1) & bs_bits) != 0 {
            // find out where the backspace is
            let bs_dist: u32 = trailingzeroes(u64::from(bs_bits));
            let escape_char: u8 = unsafe { *src.get_unchecked(src_i + bs_dist as usize + 1) };
            // we encountered backslash first. Handle backslash
            if escape_char == b'u' {
                // move src/dst up to the start; they will be further adjusted
                // within the unicode codepoint handling code.
                src_i += bs_dist as usize;
                dst_i += bs_dist as usize;
                let (o, s) = if let Ok(r) =
                    handle_unicode_codepoint(unsafe { src.get_unchecked(src_i..) }, unsafe {
                        dst.get_unchecked_mut(dst_i..)
                    }) {
                    r
                } else {
                    return Err(Error::generic(ErrorType::InvlaidUnicodeCodepoint));
                };
                if o == 0 {
                    return Err(Error::generic(ErrorType::InvlaidUnicodeCodepoint));
                };
                // We moved o steps forword at the destiation and 6 on the source
                src_i += s;
                dst_i += o;
            } else {
                // simple 1:1 conversion. Will eat bs_dist+2 characters in input and
                // write bs_dist+1 characters to output
                // note this may reach beyond the part of the buffer we've actually
                // seen. I think this is ok
                let escape_result: u8 = unsafe { *ESCAPE_MAP.get_unchecked(escape_char as usize) };
                if escape_result == 0 {
                    return Err(Error::generic(ErrorType::InvalidEscape));
                }
                unsafe {
                    *dst.get_unchecked_mut(dst_i + bs_dist as usize) = escape_result;
                }
                src_i += bs_dist as usize + 2;
                dst_i += bs_dist as usize + 1;
            }
        } else {
            // they are the same. Since they can't co-occur, it means we encountered
            // neither.
            src_i += 32;
            dst_i += 32;
        }
    }
}

impl<'de> Deserializer<'de> {
    pub fn validate(
        data: &[u8],
        input: &mut [u8],
        structural_indexes: &[u32],
    ) -> Result<(Vec<usize>, usize)> {
        let mut counts = Vec::with_capacity(structural_indexes.len());
        let mut stack = Vec::with_capacity(structural_indexes.len());
        unsafe {
            counts.set_len(structural_indexes.len());
            stack.set_len(structural_indexes.len());
        }

        let mut depth = 0;
        let mut last_start = 1;
        let mut cnt = 0;
        let mut offset: usize = 0;

        // let mut i: usize = 0; // index of the structural character (0,1,2,3...)
        // location of the structural character in the input (buf)
        let mut idx: usize;
        // used to track the (structural) character we are looking at, updated
        // by UPDATE_CHAR macro
        let mut c: u8;
        let mut i = 0;

        // this macro reads the next structural character, updating idx, i and c.
        let mut si = structural_indexes.iter().skip(1).peekable();
        macro_rules! update_char {
            () => {
                idx = *stry!(si.next().ok_or_else(|| (Error::generic(ErrorType::Syntax)))) as usize;
                i += 1;
                c = unsafe { *data.get_unchecked(idx) };
            };
        }

        let mut state;
        macro_rules! goto {
            ($state:expr) => {{
                state = $state;
                continue;
            }};
        }

        // The continue cases are the most frequently called onces it's
        // worth pulling them out into a macro (aka inlining them)
        // Since we don't have a 'gogo' in rust.
        macro_rules! array_continue {
            () => {{
                update_char!();
                match c {
                    b',' => {
                        cnt += 1;
                        update_char!();
                        goto!(MainArraySwitch);
                    }
                    b']' => {
                        goto!(ScopeEnd);
                    }
                    _c => {
                        fail!(ErrorType::ExpectedArrayContent);
                    }
                }
            }};
        }

        macro_rules! object_continue {
            () => {{
                update_char!();
                match c {
                    b',' => {
                        cnt += 1;
                        update_char!();
                        if c != b'"' {
                            fail!(ErrorType::ExpectedObjectKey);
                        } else {
                            let d = if let Some(next) = si.peek() {
                                (**next as usize) - idx
                            } else {
                                // If we're the last element we count to the end
                                data.len() - idx
                            };

                            if d < 32 {
                                unsafe {
                                    *counts.get_unchecked_mut(i) =
                                        stry!(extract_str_short(data, input, &mut offset, idx))
                                };
                            } else {
                                unsafe {
                                    *counts.get_unchecked_mut(i) =
                                        stry!(extract_str(data, input, &mut offset, idx))
                                };
                            }
                            goto!(ObjectKey);
                        }
                    }
                    b'}' => {
                        goto!(ScopeEnd);
                    }
                    _ => {
                        fail!(ErrorType::ExpectedObjectContent);
                    }
                }
            }};
        }

        macro_rules! array_begin {
            () => {{
                update_char!();
                if c == b']' {
                    cnt = 0;
                    goto!(ScopeEnd);
                }
                goto!(MainArraySwitch);
            }};
        }

        macro_rules! object_begin {
            () => {{
                update_char!();
                match c {
                    b'"' => {
                        let d = if let Some(next) = si.peek() {
                            (**next as usize) - idx
                        } else {
                            // If we're the last element we count to the end
                            data.len() - idx
                        };

                        if d < 32 {
                            unsafe {
                                *counts.get_unchecked_mut(i) =
                                    stry!(extract_str_short(data, input, &mut offset, idx))
                            };
                        } else {
                            unsafe {
                                *counts.get_unchecked_mut(i) =
                                    stry!(extract_str(data, input, &mut offset, idx))
                            };
                        }
                        goto!(ObjectKey);
                    }
                    b'}' => {
                        cnt = 0;
                        goto!(ScopeEnd);
                    }
                    _c => {
                        fail!(ErrorType::ExpectedObjectContent);
                    }
                }
            }};
        }

        macro_rules! fail {
            () => {
                return Err(Error::new(i, idx, c as char, ErrorType::InternalError));
            };
            ($t:expr) => {
                return Err(Error::new(i, idx, c as char, $t));
            };
        }
        // State start, we pull this outside of the
        // loop to reduce the number of requried checks
        update_char!();
        match c {
            b'{' => {
                unsafe {
                    *stack.get_unchecked_mut(depth) = (StackState::Start, last_start, cnt);
                }
                depth += 1;
                last_start = i;
                cnt = 1;

                update_char!();
                match c {
                    b'"' => {
                        let d = if let Some(next) = si.peek() {
                            (**next as usize) - idx
                        } else {
                            // If we're the last element we count to the end
                            data.len() - idx
                        };

                        if d < 32 {
                            unsafe {
                                *counts.get_unchecked_mut(i) =
                                    stry!(extract_str_short(data, input, &mut offset, idx))
                            };
                        } else {
                            unsafe {
                                *counts.get_unchecked_mut(i) =
                                    stry!(extract_str(data, input, &mut offset, idx))
                            };
                        }
                        state = State::ObjectKey;
                    }
                    b'}' => {
                        cnt = 0;
                        state = State::ScopeEnd;
                    }
                    _c => {
                        fail!(ErrorType::ExpectedObjectContent);
                    }
                }
            }
            b'[' => {
                unsafe {
                    *stack.get_unchecked_mut(depth) = (StackState::Start, last_start, cnt);
                }
                depth += 1;
                last_start = i;
                cnt = 1;

                update_char!();
                if c == b']' {
                    cnt = 0;
                    state = State::ScopeEnd;
                } else {
                    state = State::MainArraySwitch
                }
            }
            b'"' => {
                let d = if let Some(next) = si.peek() {
                    (**next as usize) - idx
                } else {
                    // If we're the last element we count to the end
                    data.len() - idx
                };

                if d < 32 {
                    unsafe {
                        *counts.get_unchecked_mut(i) =
                            stry!(extract_str_short(data, input, &mut offset, idx))
                    };
                } else {
                    unsafe {
                        *counts.get_unchecked_mut(i) =
                            stry!(extract_str(data, input, &mut offset, idx))
                    };
                }
                if si.next().is_none() {
                    return Ok((counts, offset));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            b't' => {
                let len = data.len();
                let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                unsafe {
                    copy.as_mut_ptr().copy_from(data.as_ptr(), len);
                    if !is_valid_true_atom(copy.get_unchecked(idx..)) {
                        fail!(ErrorType::ExpectedNull); // TODO: better error
                    }
                };
                if si.next().is_none() {
                    return Ok((counts, offset));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            b'f' => {
                let len = data.len();
                let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                unsafe {
                    copy.as_mut_ptr().copy_from(data.as_ptr(), len);
                    if !is_valid_false_atom(copy.get_unchecked(idx..)) {
                        fail!(ErrorType::ExpectedNull); // TODO: better error
                    }
                };
                if si.next().is_none() {
                    return Ok((counts, offset));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            b'n' => {
                let len = data.len();
                let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                unsafe {
                    copy.as_mut_ptr().copy_from(data.as_ptr(), len);
                    if !is_valid_null_atom(copy.get_unchecked(idx..)) {
                        fail!(ErrorType::ExpectedNull); // TODO: better error
                    }
                };
                if si.next().is_none() {
                    return Ok((counts, offset));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            b'-' | b'0'...b'9' => {
                if si.next().is_none() {
                    return Ok((counts, offset));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            _ => {
                fail!();
            }
        }

        loop {
            use self::State::*;
            match state {
                ////////////////////////////// OBJECT STATES /////////////////////////////
                ObjectKey => {
                    update_char!();
                    if unlikely!(c != b':') {
                        fail!(ErrorType::ExpectedObjectColon);
                    }
                    update_char!();
                    match c {
                        b'"' => {
                            let d = if let Some(next) = si.peek() {
                                (**next as usize) - idx
                            } else {
                                // If we're the last element we count to the end
                                data.len() - idx
                            };

                            if d < 32 {
                                unsafe {
                                    *counts.get_unchecked_mut(i) =
                                        stry!(extract_str_short(data, input, &mut offset, idx));
                                }
                            } else {
                                unsafe {
                                    *counts.get_unchecked_mut(i) =
                                        stry!(extract_str(data, input, &mut offset, idx))
                                };
                            }
                            object_continue!();
                        }
                        b't' => {
                            if !is_valid_true_atom(unsafe { data.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedBoolean); // TODO: better error
                            }
                            object_continue!();
                        }
                        b'f' => {
                            if !is_valid_false_atom(unsafe { data.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedBoolean); // TODO: better error
                            }
                            object_continue!();
                        }
                        b'n' => {
                            if !is_valid_null_atom(unsafe { data.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedNull); // TODO: better error
                            }
                            object_continue!();
                        }
                        b'-' | b'0'...b'9' => {
                            object_continue!();
                        }
                        b'{' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) =
                                    (StackState::Object, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 1;
                            object_begin!();
                        }
                        b'[' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) =
                                    (StackState::Object, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 1;
                            array_begin!();
                        }
                        _c => {
                            fail!();
                        }
                    }
                }
                ////////////////////////////// COMMON STATE /////////////////////////////
                ScopeEnd => {
                    if depth == 0 {
                        return Err(Error::generic(ErrorType::Syntax));
                    }
                    depth -= 1;
                    unsafe {
                        *counts.get_unchecked_mut(last_start) = cnt;
                    }

                    let (a_state, a_last_start, a_cnt) = unsafe { stack.get_unchecked(depth) };
                    //                    let (a_state, a_last_start, a_cnt) = unsafe {  };
                    //stry!(stack.pop().ok_or_else(|| Error::generic(ErrorType::Syntax)));

                    last_start = *a_last_start;
                    cnt = *a_cnt;

                    match &a_state {
                        StackState::Object => object_continue!(),
                        StackState::Array => array_continue!(),
                        StackState::Start => {
                            if si.next().is_none() {
                                return Ok((counts, offset));
                            } else {
                                fail!();
                            }
                        }
                    };
                }

                ////////////////////////////// ARRAY STATES /////////////////////////////
                MainArraySwitch => {
                    // we call update char on all paths in, so we can peek at c on the
                    // on paths that can accept a close square brace (post-, and at start)
                    match c {
                        b'"' => {
                            let d = if let Some(next) = si.peek() {
                                (**next as usize) - idx
                            } else {
                                // If we're the last element we count to the end
                                data.len() - idx
                            };

                            if d < 32 {
                                unsafe {
                                    *counts.get_unchecked_mut(i) =
                                        stry!(extract_str_short(data, input, &mut offset, idx))
                                };
                            } else {
                                unsafe {
                                    *counts.get_unchecked_mut(i) =
                                        stry!(extract_str(data, input, &mut offset, idx))
                                };
                            }
                            array_continue!();
                        }
                        b't' => {
                            if !is_valid_true_atom(unsafe { data.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedBoolean); // TODO: better error
                            }
                            array_continue!();
                        }
                        b'f' => {
                            if !is_valid_false_atom(unsafe { data.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedBoolean); // TODO: better error
                            }
                            array_continue!();
                        }
                        b'n' => {
                            if !is_valid_null_atom(unsafe { data.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedNull); // TODO: better error
                            }
                            array_continue!();
                        }
                        b'-' | b'0'...b'9' => {
                            array_continue!();
                        }
                        b'{' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) =
                                    (StackState::Array, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 1;
                            object_begin!();
                        }
                        b'[' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) =
                                    (StackState::Array, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 1;
                            array_begin!();
                        }
                        _c => {
                            fail!();
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn true_atom() {
        assert!(is_valid_true_atom(b"true    "));
        assert!(!is_valid_true_atom(b"tru     "));
        assert!(!is_valid_true_atom(b" rue    "));
    }
    #[test]
    fn false_atom() {
        assert!(is_valid_false_atom(b"false   "));
        assert!(!is_valid_false_atom(b"falze   "));
        assert!(!is_valid_false_atom(b"falsy   "));
        assert!(!is_valid_false_atom(b"fals    "));
        assert!(!is_valid_false_atom(b" alse   "));

        //unsafe { assert!(!is_valid_false_atom(b"fals    " as *const u8)) }
        //        unsafe { assert!(!is_valid_false_atom(b"false   " as *const u8)) }
    }
    #[test]
    fn null_atom() {
        assert!(is_valid_null_atom(b"null    "));
        assert!(!is_valid_null_atom(b"nul     "));
        assert!(!is_valid_null_atom(b" ull    "));
    }
}
