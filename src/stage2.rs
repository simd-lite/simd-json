#![allow(dead_code)]
use crate::charutils::*;
use crate::{Deserializer, Error, ErrorType, Result};
//use crate::portability::*;

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_valid_true_atom(loc: &[u8]) -> bool {
    // TODO is this expensive?
    let mut error: u32;
    unsafe {
        //let tv: u64 = *(b"true    ".as_ptr() as *const u64);
        // this is the same:
        const TV: u64 = 0x00_00_00_00_65_75_72_74;
        const MASK4: u64 = 0x00_00_00_00_ff_ff_ff_ff;

        // TODO: does this has the same effect as:
        //   std::memcpy(&locval, loc, sizeof(uint64_t));
        let locval: u64 = *(loc.as_ptr() as *const u64);

        error = ((locval & MASK4) ^ TV) as u32;
        error |= is_not_structural_or_whitespace(*loc.get_unchecked(4));
    }
    error == 0
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_valid_false_atom(loc: &[u8]) -> bool {
    // TODO: this is ugly and probably copies data every time
    unsafe {
        let error;
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
        error = ((locval ^ FV) & MASK5) == 0;
        error || is_not_structural_or_whitespace(*loc.get_unchecked(5)) == 1
    }
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_valid_null_atom(loc: &[u8]) -> bool {
    // TODO is this expensive?
    let mut error: u32;
    unsafe {
        //let nv: u64 = *(b"null   ".as_ptr() as *const u64);
        // this is the same:
        const NV: u64 = 0x00_00_00_00_6c_6c_75_6e;
        const MASK4: u64 = 0x00_00_00_00_ff_ff_ff_ff;
        let locval: u64 = *(loc.as_ptr() as *const u64);

        error = ((locval & MASK4) ^ NV) as u32;
        error |= is_not_structural_or_whitespace(*loc.get_unchecked(4));
    }
    error == 0
}

#[derive(Debug)]
enum State {
    Start,
    StartContinue,

    ObjectBegin,
    ObjectKey,
    ObjectContinue,

    ScopeEnd,

    ArrayBegin,
    MainArraySwitch,
    ArrayContinue,

    Succeed,
    Fail,
}

impl<'de> Deserializer<'de> {
    pub fn validate(input: &[u8], structural_indexes: &[u32]) -> Result<(Vec<usize>, usize)> {
        let mut counts = Vec::with_capacity(structural_indexes.len());
        unsafe {
            counts.set_len(structural_indexes.len());
        };
        let mut stack = Vec::with_capacity(structural_indexes.len() / 2); // since we are open close we know worst case this is 2x the size
        let mut depth = 0;
        unsafe {
            stack.set_len(structural_indexes.len() / 2);
        }

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
                idx = *si
                    .next()
                    .ok_or_else(|| (Error::generic(ErrorType::Syntax)))? as usize;

                i += 1;
                c = unsafe { *input.get_unchecked(idx) };
            };
        }

        update_char!();

        let mut state = State::Start;
        macro_rules! goto {
            ($state:expr) => {{
                state = $state;
                continue;
            }};
        }

        loop {
            use State::*;
            match state {
                Start => {
                    match c {
                        b'{' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) = (state, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 0;
                            goto!(ObjectBegin);
                        }
                        b'[' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) = (state, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 0;
                            goto!(ArrayBegin);
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
                            goto!(StartContinue);
                        }
                        b't' | b'f' | b'n' | b'-' | b'0'...b'9' => {
                            goto!(StartContinue);
                        }
                        _ => {
                            goto!(Fail);
                        }
                    }
                }
                StartContinue => {
                    // the string might not be NULL terminated.
                    if si.next().is_none() {
                        goto!(Succeed);
                    } else {
                        goto!(Fail);
                    }
                }

                ////////////////////////////// OBJECT STATES /////////////////////////////
                ObjectBegin => {
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
                            goto!(ScopeEnd);
                        }
                        _c => {
                            goto!(Fail);
                        }
                    }
                }
                ObjectKey => {
                    update_char!();
                    if c != b':' {
                        goto!(Fail);
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
                            goto!(ObjectContinue);
                        }
                        b't' | b'f' | b'n' | b'-' | b'0'...b'9' => {
                            goto!(ObjectContinue);
                        }

                        b'{' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) = (state, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 0;
                            goto!(ObjectBegin);
                        }
                        b'[' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) = (state, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 0;
                            goto!(ArrayBegin);
                        }
                        _c => {
                            goto!(Fail);
                        }
                    }
                }
                ObjectContinue => {
                    update_char!();
                    match c {
                        b',' => {
                            cnt += 1;
                            update_char!();
                            if c != b'"' {
                                goto!(Fail);
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
                            goto!(Fail);
                        }
                    }
                }

                ////////////////////////////// COMMON STATE /////////////////////////////
                ScopeEnd => {
                    if i != last_start + 1 {
                        cnt += 1;
                    }
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
                        ObjectKey => goto!(ObjectContinue),
                        MainArraySwitch => goto!(ArrayContinue),
                        Start => goto!(StartContinue),
                        _ => goto!(Fail),
                    };
                }

                ////////////////////////////// ARRAY STATES /////////////////////////////
                ArrayBegin => {
                    update_char!();
                    if c == b']' {
                        goto!(ScopeEnd);
                    }
                    goto!(MainArraySwitch);
                }
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
                            goto!(ArrayContinue);
                        }
                        b't' | b'f' | b'n' | b'-' | b'0'...b'9' => {
                            goto!(ArrayContinue);
                        }
                        b'{' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) = (state, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 0;
                            goto!(ObjectBegin);
                        }
                        b'[' => {
                            unsafe {
                                *stack.get_unchecked_mut(depth) = (state, last_start, cnt);
                            }
                            depth += 1;
                            last_start = i;
                            cnt = 0;
                            goto!(ArrayBegin);
                        }
                        _c => {
                            goto!(Fail);
                        }
                    }
                }
                ArrayContinue => {
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
                            goto!(Fail);
                        }
                    }
                }
                ////////////////////////////// FINAL STATES /////////////////////////////
                Succeed => {
                    return Ok((counts, str_len as usize));
                }
                Fail => {
                    return Err(Error::generic(ErrorType::InternalError));
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
