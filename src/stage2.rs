#![allow(dead_code)]
use crate::charutils::*;
use crate::{Deserializer, Error, ErrorType, Result, SIMDJSON_PADDING};
//use crate::portability::*;

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
        let locval: u64 = *(loc.as_ptr() as *const u64);

        error = (locval & MASK4) ^ TV;
        error |= is_not_structural_or_whitespace(*loc.get_unchecked(4)) as u64;
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

        let locval: u64 = *(loc.as_ptr() as *const u64);

        // FIXME the original code looks like this:
        // error = ((locval & mask5) ^ fv) as u32;
        // but that failes on falsy as the u32 conversion
        // will mask the error on the y so we re-write it
        // it would be interesting what the consequecnes are
        error = (locval & MASK5) ^ FV;
        error |= is_not_structural_or_whitespace(*loc.get_unchecked(5)) as u64;
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
        let locval: u64 = *(loc.as_ptr() as *const u64);

        error = (locval & MASK4) ^ NV;
        error |= is_not_structural_or_whitespace(*loc.get_unchecked(4)) as u64;
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

impl<'de> Deserializer<'de> {
    pub fn validate(input: &[u8], structural_indexes: &[u32]) -> Result<(Vec<usize>, usize)> {
        let mut counts = Vec::with_capacity(structural_indexes.len());
        let mut stack = Vec::with_capacity(structural_indexes.len());
        unsafe {
            counts.set_len(structural_indexes.len());
            stack.set_len(structural_indexes.len());
        }

        let mut depth = 0;
        let mut last_start = 1;
        let mut cnt = 0;
        let mut str_len = 0;

        //        let mut i: usize = 0; // index of the structural character (0,1,2,3...)
        let mut idx: usize; // location of the structural character in the input (buf)
        let mut c: u8; // used to track the (structural) character we are looking at, updated
                       // by UPDATE_CHAR macro
        let mut i = 0;

        // this macro reads the next structural character, updating idx, i and c.
        let mut si = structural_indexes.iter().skip(1).peekable();
        macro_rules! update_char {
            () => {
                idx = *stry!(si.next().ok_or_else(|| (Error::generic(ErrorType::Syntax)))) as usize;
                i += 1;
                c = unsafe { *input.get_unchecked(idx) };
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
                                input.len() - idx
                            };
                            if d > str_len {
                                str_len = d;
                            }

                            unsafe { *counts.get_unchecked_mut(i) = d };
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
                            input.len() - idx
                        };
                        if d > str_len {
                            str_len = d;
                        }
                        unsafe { *counts.get_unchecked_mut(i) = d };
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
                            input.len() - idx
                        };
                        if d > str_len {
                            str_len = d;
                        }
                        unsafe { *counts.get_unchecked_mut(i) = d };
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
                    input.len() - idx
                };
                if d > str_len {
                    str_len = d;
                }
                unsafe { *counts.get_unchecked_mut(i) = d };
                if si.next().is_none() {
                    return Ok((counts, str_len as usize));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            b't' => {
                let len = input.len();
                let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                unsafe {
                    copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    if !is_valid_true_atom(copy.get_unchecked(idx..)) {
                        fail!(ErrorType::ExpectedNull); // TODO: better error
                    }
                };
                if si.next().is_none() {
                    return Ok((counts, str_len as usize));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            b'f' => {
                let len = input.len();
                let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                unsafe {
                    copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    if !is_valid_false_atom(copy.get_unchecked(idx..)) {
                        fail!(ErrorType::ExpectedNull); // TODO: better error
                    }
                };
                if si.next().is_none() {
                    return Ok((counts, str_len as usize));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            b'n' => {
                let len = input.len();
                let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                unsafe {
                    copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    if !is_valid_null_atom(copy.get_unchecked(idx..)) {
                        fail!(ErrorType::ExpectedNull); // TODO: better error
                    }
                };
                if si.next().is_none() {
                    return Ok((counts, str_len as usize));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            b'-' | b'0'...b'9' => {
                if si.next().is_none() {
                    return Ok((counts, str_len as usize));
                } else {
                    fail!(ErrorType::TrailingCharacters);
                }
            }
            _ => {
                fail!();
            }
        }

        loop {
            use State::*;
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
                                input.len() - idx
                            };
                            if d > str_len {
                                str_len = d;
                            }

                            unsafe { *counts.get_unchecked_mut(i) = d };
                            object_continue!();
                        }
                        b't' => {
                            if !is_valid_true_atom(unsafe { input.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedBoolean); // TODO: better error
                            }
                            object_continue!();
                        }
                        b'f' => {
                            if !is_valid_false_atom(unsafe { input.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedBoolean); // TODO: better error
                            }
                            object_continue!();
                        }
                        b'n' => {
                            if !is_valid_null_atom(unsafe { input.get_unchecked(idx..) }) {
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
                                return Ok((counts, str_len as usize));
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
                                input.len() - idx
                            };
                            if d > str_len {
                                str_len = d;
                            }

                            unsafe { *counts.get_unchecked_mut(i) = d };
                            array_continue!();
                        }
                        b't' => {
                            if !is_valid_true_atom(unsafe { input.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedBoolean); // TODO: better error
                            }
                            array_continue!();
                        }
                        b'f' => {
                            if !is_valid_false_atom(unsafe { input.get_unchecked(idx..) }) {
                                fail!(ErrorType::ExpectedBoolean); // TODO: better error
                            }
                            array_continue!();
                        }
                        b'n' => {
                            if !is_valid_null_atom(unsafe { input.get_unchecked(idx..) }) {
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
