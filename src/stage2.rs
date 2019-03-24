#![allow(dead_code)]
use crate::charutils::*;
use crate::numberparse::*;
use crate::parsedjson::*;
//use crate::portability::*;
use crate::stringparse::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::{mem, ptr};

const SIMDJSON_PADDING: usize = mem::size_of::<__m256i>();

#[inline(always)]
unsafe fn is_valid_true_atom(loc: *const u8) -> bool {
    // TODO is this expensive?
    let tv: u64 = *(b"true   ".as_ptr() as *const u64);
    let mask4: u64 = 0x00_00_00_00_ff_ff_ff_ff;
    let mut error: u32;

    // TODO: does this has the same effect as:
    //   std::memcpy(&locval, loc, sizeof(uint64_t));
    let locval: u64 = *(loc as *const u64);

    error = ((locval & mask4) ^ tv) as u32;
    error |= is_not_structural_or_whitespace(*loc.offset(4));
    return error == 0;
}

#[inline(always)]
unsafe fn is_valid_false_atom(loc: *const u8) -> bool {
    // TODO: this is ugly and probably copies data every time
    let fv: u64 = *(b"false   ".as_ptr() as *const u64);
    // FIXME: The original value was 0x000000ffffffffff
    // but it seems rust swapps bit somewhere o.O od knows why
    // so we have to put the 0's last so they end up frist.
    // This could be related to the use of mem::transmute instead
    //
    let mask5: u64 = 0x000000ffffffffff;
    let mut error;

    let locval: u64 = *(loc as *const u64);

    // FIXME the original code looks like this:
    // error = ((locval & mask5) ^ fv) as u32;
    // but that failes on falsy as the u32 conversion
    // will mask the error on the y so we re-write it
    // it would be interesting what the consequecnes are
    error = ((locval ^ fv) & mask5) == 0;
    error = error || is_not_structural_or_whitespace(*loc.offset(5)) == 1;
    return error;
}

#[inline(always)]
unsafe fn is_valid_null_atom(loc: *const u8) -> bool {
    // TODO is this expensive?
    let tv: u64 = *(b"null   ".as_ptr() as *const u64);
    let mask4: u64 = 0x00_00_00_00_ff_ff_ff_ff;
    let mut error: u32;
    let locval: u64 = *(loc as *const u64);

    error = ((locval & mask4) ^ tv) as u32;
    error |= is_not_structural_or_whitespace(*loc.offset(4));
    return error == 0;
}

/************
 * The JSON is parsed to a tape, see the accompanying tape.md file
 * for documentation.
 ***********/

#[derive(PartialEq, Debug)]
pub enum MachineError {
    Capacity(usize, usize),
    Depth,
    TapeError,
    InternalError,
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
pub unsafe fn unified_machine(
    buf: &[u8],
    len: usize,
    pj: &mut ParsedJson,
) -> Result<(), MachineError> {
    let buf = buf.as_ptr();
    let mut i: usize = 0; // index of the structural character (0,1,2,3...)
    let mut idx: usize; // location of the structural character in the input (buf)
    let mut c: u8; // used to track the (structural) character we are looking at, updated
                   // by UPDATE_CHAR macro
    let mut depth: usize = 0; // could have an arbitrary starting depth
    pj.init();

    pj.init();

    if pj.bytecapacity < len {
        return Err(MachineError::Capacity(pj.bytecapacity, len));
    }

    // this macro reads the next structural character, updating idx, i and c.
    macro_rules! update_char {
        () => {
            idx = pj.structural_indexes[i] as usize;
            i += 1;
            c = *buf.offset(idx as isize);
        };
    }

    ////////////////////////////// START STATE /////////////////////////////

    pj.ret_address[depth] = b's';
    pj.containing_scope_offset[depth] = pj.get_current_loc();
    pj.write_tape(0, b'r'); // r for root, 0 is going to get overwritten
                            // the root is used, if nothing else, to capture the size of the tape
    depth += 1; // everything starts at depth = 1, depth = 0 is just for the root, the root may contain an object, an array or something else.
    if depth > pj.depthcapacity {
        //goto fail;
        return Err(MachineError::Depth);
    }

    update_char!();

    let mut state = State::Start;
    macro_rules! goto {
        ($state:expr) => {
            state = $state;
            continue;
        };
    }
    loop {
        use State::*;
        match state {
            Start => {
                match c {
                    b'{' => {
                        pj.containing_scope_offset[depth] = pj.get_current_loc();
                        pj.ret_address[depth] = b's';
                        depth += 1;
                        if depth > pj.depthcapacity {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c); // strangely, moving this to object_begin slows things down
                        goto!(ObjectBegin);
                    }
                    b'[' => {
                        pj.containing_scope_offset[depth] = pj.get_current_loc();
                        pj.ret_address[depth] = b's';
                        depth += 1;
                        if depth > pj.depthcapacity {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                        goto!(ArrayBegin);
                    }
                    b'"' => {
                        if !parse_string(buf, len, pj, depth, idx as u32) {
                            goto!(Fail);
                        }
                    }
                    b't' => {
                        // we need to make a copy to make sure that the string is NULL terminated.
                        // this only applies to the JSON document made solely of the true value.
                        // this will almost never be called in practice
                        let copy: *mut u8 = vec![0u8; len + SIMDJSON_PADDING].as_mut_ptr();
                        ptr::copy_nonoverlapping(buf, copy, len);
                        *copy.offset(len as isize) = b'\0';
                        if !is_valid_true_atom(copy.offset(idx as isize)) {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                    }
                    b'f' => {
                        // we need to make a copy to make sure that the string is NULL terminated.
                        // this only applies to the JSON document made solely of the false value.
                        // this will almost never be called in practice
                        let copy = vec![0u8; len + SIMDJSON_PADDING].as_mut_ptr();
                        ptr::copy_nonoverlapping(buf, copy, len);
                        *copy.offset(len as isize) = b'\0';
                        if !is_valid_false_atom(copy.offset(idx as isize)) {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                    }
                    b'n' => {
                        // we need to make a copy to make sure that the string is NULL terminated.
                        // this only applies to the JSON document made solely of the null value.
                        // this will almost never be called in practice
                        let copy = vec![0u8; len + SIMDJSON_PADDING].as_mut_ptr();
                        ptr::copy_nonoverlapping(buf, copy, len);
                        *copy.offset(len as isize) = b'\0';
                        if !is_valid_null_atom(copy.offset(idx as isize)) {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                    }
                    b'0'...b'9' => {
                        // we need to make a copy to make sure that the string is NULL terminated.
                        // this is done only for JSON documents made of a sole number
                        // this will almost never be called in practice
                        let copy = vec![0u8; len + SIMDJSON_PADDING].as_mut_ptr();
                        ptr::copy_nonoverlapping(buf, copy, len);
                        *copy.offset(len as isize) = b'\0';
                        if !parse_number(copy.offset(idx as isize), pj, idx as u32, false) {
                            goto!(Fail);
                        }
                    }
                    b'-' => {
                        // we need to make a copy to make sure that the string is NULL terminated.
                        // this is done only for JSON documents made of a sole number
                        // this will almost never be called in practice
                        let copy = vec![0u8; len + SIMDJSON_PADDING].as_mut_ptr();
                        ptr::copy_nonoverlapping(buf, copy, len);
                        *copy.offset(len as isize) = b'\0';
                        if !parse_number(copy.offset(idx as isize), pj, idx as u32, false) {
                            goto!(Fail);
                        }
                    }
                    _ => {
                        goto!(Fail);
                    }
                }
            }
            StartContinue => {
                // the string might not be NULL terminated.
                if i + 1 == pj.n_structural_indexes {
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
                        if !parse_string(buf, len, pj, depth, idx as u32) {
                            goto!(Fail);
                        }
                        goto!(ObjectKey);
                    }
                    b'}' => {
                        goto!(ScopeEnd);
                    }
                    c => {
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
                        if !parse_string(buf, len, pj, depth, idx as u32) {
                            goto!(Fail);
                        }
                        goto!(ObjectContinue);
                    }
                    b't' => {
                        if !is_valid_true_atom(buf.offset(idx as isize)) {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                        goto!(ObjectContinue);
                    }
                    b'f' => {
                        if !is_valid_false_atom(buf.offset(idx as isize)) {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                        goto!(ObjectContinue);
                    }
                    b'n' => {
                        if !is_valid_null_atom(buf.offset(idx as isize)) {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                        goto!(ObjectContinue);
                    }
                    b'0'...b'9' => {
                        if !parse_number(buf, pj, idx as u32, false) {
                            goto!(Fail);
                        }
                        goto!(ObjectContinue);
                    }
                    b'-' => {
                        if !parse_number(buf, pj, idx as u32, true) {
                            goto!(Fail);
                        }
                        goto!(ObjectContinue);
                    }
                    b'{' => {
                        pj.containing_scope_offset[depth] = pj.get_current_loc();
                        pj.write_tape(0, c); // here the compilers knows what c is so this gets optimized
                                             // we have not yet encountered } so we need to come back for it
                        pj.ret_address[depth] = b'o';
                        // we found an object inside an object, so we need to increment the depth
                        depth += 1;
                        if depth > pj.depthcapacity {
                            goto!(Fail);
                        }
                        goto!(ObjectBegin);
                    }
                    b'[' => {
                        pj.containing_scope_offset[depth] = pj.get_current_loc();
                        pj.write_tape(0, c); // here the compilers knows what c is so this gets optimized
                                             // we have not yet encountered } so we need to come back for it
                        pj.ret_address[depth] = b'o';
                        // we found an array inside an object, so we need to increment the depth
                        depth += 1;
                        if depth > pj.depthcapacity {
                            goto!(Fail);
                        }
                        goto!(ArrayBegin);
                    }
                    c => {
                        goto!(Fail);
                    }
                }
            }
            ObjectContinue => {
                update_char!();
                match c {
                    b',' => {
                        update_char!();
                        if c != b'"' {
                            goto!(Fail);
                        } else {
                            if !parse_string(buf, len, pj, depth, idx as u32) {
                                goto!(Fail);
                            }
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
                // write our tape location to the header scope
                depth -= 1;
                pj.write_tape(pj.containing_scope_offset[depth], c);
                pj.annotate_previousloc(pj.containing_scope_offset[depth], pj.get_current_loc());
                // goto saved_state
                if pj.ret_address[depth] == b'a' {
                    goto!(ArrayContinue);
                } else if pj.ret_address[depth] == b'o' {
                    goto!(ObjectContinue);
                } else {
                    goto!(StartContinue);
                }
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
                        if !parse_string(buf, len, pj, depth, idx as u32) {
                            goto!(Fail);
                        }
                        goto!(ArrayContinue);
                    }
                    b't' => {
                        if !is_valid_true_atom(buf.add(idx)) {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                        goto!(ArrayContinue);
                    }
                    b'f' => {
                        if !is_valid_false_atom(buf.add(idx)) {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                        goto!(ArrayContinue);
                    }
                    b'n' => {
                        if !is_valid_null_atom(buf.add(idx)) {
                            goto!(Fail);
                        }
                        pj.write_tape(0, c);
                        goto!(ArrayContinue);
                    }
                    b'0'...b'9' => {
                        if !parse_number(buf, pj, idx as u32, false) {
                            goto!(Fail);
                        }
                        goto!(ArrayContinue);
                    }
                    b'-' => {
                        if !parse_number(buf, pj, idx as u32, true) {
                            goto!(Fail);
                        }
                        goto!(ArrayContinue);
                    }
                    b'{' => {
                        // we have not yet encountered ] so we need to come back for it
                        pj.containing_scope_offset[depth] = pj.get_current_loc();
                        pj.write_tape(0, c); //  here the compilers knows what c is so this gets optimized
                        pj.ret_address[depth] = b'a';
                        // we found an object inside an array, so we need to increment the depth
                        depth = 1;
                        if depth > pj.depthcapacity {
                            goto!(Fail);
                        }

                        goto!(ObjectBegin);
                    }
                    b'[' => {
                        // we have not yet encountered ] so we need to come back for it
                        pj.containing_scope_offset[depth] = pj.get_current_loc();
                        pj.write_tape(0, c); // here the compilers knows what c is so this gets optimized
                        pj.ret_address[depth] = b'a';
                        // we found an array inside an array, so we need to increment the depth
                        depth += 1;
                        if depth > pj.depthcapacity {
                            goto!(Fail);
                        }
                        goto!(ArrayBegin);
                    }
                    c => {
                        goto!(Fail);
                    }
                }
            }
            ArrayContinue => {
                update_char!();
                match c {
                    b',' => {
                        update_char!();
                        goto!(MainArraySwitch);
                    }
                    b']' => {
                        goto!(ScopeEnd);
                    }
                    c => {
                        goto!(Fail);
                    }
                }
            }
            ////////////////////////////// FINAL STATES /////////////////////////////
            Succeed => {
                depth -= 1;
                if depth != 0 {
                    eprintln!("internal bug\n");
                    return Err(MachineError::InternalError);
                }
                if pj.containing_scope_offset[depth] != 0 {
                    eprintln!("internal bug\n");
                    return Err(MachineError::InternalError);
                }
                pj.annotate_previousloc(pj.containing_scope_offset[depth], pj.get_current_loc());
                pj.write_tape(pj.containing_scope_offset[depth], b'r'); // r is root

                //pj.isvalid = true;
                return Ok(());
            }
            Fail => {
                /*
                dbg!(i);
                dbg!(idx);
                dbg!(c as char);
                */
                return Err(MachineError::TapeError);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn true_atom() {
        unsafe { assert!(is_valid_true_atom(b"true    " as *const u8)) }
        unsafe { assert!(!is_valid_true_atom(b"tru     " as *const u8)) }
        unsafe { assert!(!is_valid_true_atom(b" rue    " as *const u8)) }
    }
    #[test]
    fn false_atom() {
        unsafe { assert!(is_valid_false_atom(b"false   " as *const u8)) }
        unsafe { assert!(!is_valid_false_atom(b"falze   " as *const u8)) }
        unsafe { assert!(!is_valid_false_atom(b"falsy   " as *const u8)) }
        unsafe { assert!(!is_valid_false_atom(b"fals    " as *const u8)) }
        unsafe { assert!(!is_valid_false_atom(b" alse   " as *const u8)) }
        //unsafe { assert!(!is_valid_false_atom(b"fals    " as *const u8)) }
        //        unsafe { assert!(!is_valid_false_atom(b"false   " as *const u8)) }
    }
    #[test]
    fn null_atom() {
        unsafe { assert!(is_valid_null_atom(b"null    " as *const u8)) }
        unsafe { assert!(!is_valid_null_atom(b"nul     " as *const u8)) }
        unsafe { assert!(!is_valid_null_atom(b" ull    " as *const u8)) }
    }
}
