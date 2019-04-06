#![allow(dead_code)]
use crate::charutils::*;
use crate::*;
//use crate::portability::*;

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_valid_true_atom(loc: &[u8]) -> bool {
    // TODO is this expensive?
    let mut error: u32;
    unsafe {
        let tv: u64 = *(b"true   ".as_ptr() as *const u64);
        let mask4: u64 = 0x00_00_00_00_ff_ff_ff_ff;

        // TODO: does this has the same effect as:
        //   std::memcpy(&locval, loc, sizeof(uint64_t));
        let locval: u64 = *(loc.as_ptr() as *const u64);

        error = ((locval & mask4) ^ tv) as u32;
        error |= is_not_structural_or_whitespace(loc[4]);
    }
    error == 0
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_valid_false_atom(loc: &[u8]) -> bool {
    // TODO: this is ugly and probably copies data every time
    unsafe {
        let error;
        let fv: u64 = *(b"false   ".as_ptr() as *const u64);
        let mask5: u64 = 0x000000ffffffffff;

        let locval: u64 = *(loc.as_ptr() as *const u64);

        // FIXME the original code looks like this:
        // error = ((locval & mask5) ^ fv) as u32;
        // but that failes on falsy as the u32 conversion
        // will mask the error on the y so we re-write it
        // it would be interesting what the consequecnes are
        error = ((locval ^ fv) & mask5) == 0;
        error || is_not_structural_or_whitespace(loc[5]) == 1
    }
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_valid_null_atom(loc: &[u8]) -> bool {
    // TODO is this expensive?
    let mut error: u32;
    unsafe {
        let tv: u64 = *(b"null   ".as_ptr() as *const u64);
        let mask4: u64 = 0x00_00_00_00_ff_ff_ff_ff;
        let locval: u64 = *(loc.as_ptr() as *const u64);

        error = ((locval & mask4) ^ tv) as u32;
        error |= is_not_structural_or_whitespace(loc[4]);
    }
    error == 0
}

/************
 * The JSON is parsed to a tape, see the accompanying tape.md file
 * for documentation.
 ***********/

#[derive(Debug)]
enum State {
    Start,
    StartContinue,

    ObjectBegin,
    ObjectKey,
    ObjectContinue,

    ScopeEnd(ItemType),

    ArrayBegin,
    MainArraySwitch,
    ArrayContinue,

    Succeed,
    Fail,
}

#[derive(Debug)]
enum ItemType {
    Object,
    Array,
}

impl<'de> Deserializer<'de> {
    pub unsafe fn unified_machine(&mut self) -> Result<value! {}> {
        //        let mut i: usize = 0; // index of the structural character (0,1,2,3...)
        let mut idx: usize; // location of the structural character in the input (buf)
        let mut c: u8; // used to track the (structural) character we are looking at, updated
                       // by UPDATE_CHAR macro
        let mut depth: usize = 0; // could have an arbitrary starting depth
                                  //pj.init();

        // this macro reads the next structural character, updating idx, i and c.
        macro_rules! update_char {
            () => {
                //idx = self.structural_indexes[i] as usize;
                //i += 1;
                //*buf.offset(idx as isize);
                c = self.next_();
                idx = self.iidx;
            };
        }

        // placehodlers
        let mut ret_address = vec![0; 100];
        let depthcapacity = 100;
        let len = self.input.len();

        ////////////////////////////// START STATE /////////////////////////////
        ret_address[depth] = b's';
        //pj.containing_scope_offset[depth] = pj.get_current_loc();
        //pj.write_tape(0, ItemType::Root); // r for root, 0 is going to get overwritten
        // the root is used, if nothing else, to capture the size of the tape
        depth += 1; // everything starts at depth = 1, depth = 0 is just for the root, the root may contain an object, an array or something else.
        if depth > depthcapacity {
            //goto fail;
            return Err(self.error(ErrorType::Depth));
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
                            //pj.containing_scope_offset[depth] = pj.get_current_loc();
                            ret_address[depth] = b's';
                            depth += 1;
                            if depth > depthcapacity {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::Object); // strangely, moving this to object_begin slows things down
                            goto!(ObjectBegin);
                        }
                        b'[' => {
                            //pj.containing_scope_offset[depth] = pj.get_current_loc();
                            ret_address[depth] = b's';
                            depth += 1;
                            if depth > depthcapacity {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::Array);
                            goto!(ArrayBegin);
                        }
                        b'"' => {
                            if self.parse_str_().is_err() {
                                goto!(Fail);
                            }
                            goto!(StartContinue);
                        }
                        b't' => {
                            // we need to make a copy to make sure that the string is NULL terminated.
                            // this only applies to the JSON document made solely of the true value.
                            // this will almost never be called in practice
                            let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                            let ptr: *mut u8 = copy.as_mut_ptr();
                            ptr.copy_from(self.input.as_ptr(), len);
                            if !is_valid_true_atom(&copy[idx..]) {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::True);
                            goto!(StartContinue);
                        }
                        b'f' => {
                            // we need to make a copy to make sure that the string is NULL terminated.
                            // this only applies to the JSON document made solely of the false value.
                            // this will almost never be called in practice
                            let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                            let ptr: *mut u8 = copy.as_mut_ptr();
                            ptr.copy_from(self.input.as_ptr(), len);
                            if !is_valid_false_atom(&copy[idx..]) {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::False);
                            goto!(StartContinue);
                        }
                        b'n' => {
                            // we need to make a copy to make sure that the string is NULL terminated.
                            // this only applies to the JSON document made solely of the null value.
                            // this will almost never be called in practice
                            let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                            let ptr: *mut u8 = copy.as_mut_ptr();
                            ptr.copy_from(self.input.as_ptr(), len);
                            if !is_valid_null_atom(&copy[idx..]) {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::Null);
                            goto!(StartContinue);
                        }
                        b'0'...b'9' => {
                            // we need to make a copy to make sure that the string is NULL terminated.
                            // this is done only for JSON documents made of a sole number
                            // this will almost never be called in practice
                            //let copy = vec![0u8; len + SIMDJSON_PADDING].as_mut_ptr();
                            //copy.copy_from(buf, len);
                            if self.parse_number_(false).is_err() {
                                goto!(Fail);
                            }
                            goto!(StartContinue);
                        }
                        b'-' => {
                            // we need to make a copy to make sure that the string is NULL terminated.
                            // this is done only for JSON documents made of a sole number
                            // this will almost never be called in practice
                            //let copy = vec![0u8; len + SIMDJSON_PADDING].as_mut_ptr();
                            //copy.copy_from(buf, len);
                            if self.parse_number_(true).is_err() {
                                goto!(Fail);
                            }
                            goto!(StartContinue);
                        }
                        _ => {
                            goto!(Fail);
                        }
                    }
                }
                StartContinue => {
                    // the string might not be NULL terminated.
                    if self.idx + 1 == self.structural_indexes.len() {
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
                            if self.parse_short_str_().is_err() {
                                goto!(Fail);
                            }
                            goto!(ObjectKey);
                        }
                        b'}' => {
                            goto!(ScopeEnd(ItemType::Object));
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
                            if self.parse_short_str_().is_err() {
                                goto!(Fail);
                            }
                            goto!(ObjectContinue);
                        }
                        b't' => {
                            if !is_valid_true_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::True);
                            goto!(ObjectContinue);
                        }
                        b'f' => {
                            if !is_valid_false_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::False);
                            goto!(ObjectContinue);
                        }
                        b'n' => {
                            if !is_valid_null_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::Null);
                            goto!(ObjectContinue);
                        }
                        b'0'...b'9' => {
                            if self.parse_number_(false).is_err() {
                                goto!(Fail);
                            }
                            goto!(ObjectContinue);
                        }
                        b'-' => {
                            if self.parse_number_(true).is_err() {
                                goto!(Fail);
                            }
                            goto!(ObjectContinue);
                        }
                        b'{' => {
                            //pj.containing_scope_offset[depth] = pj.get_current_loc();
                            //pj.write_tape(0, ItemType::Object); // here the compilers knows what c is so this gets optimized
                            // we have not yet encountered } so we need to come back for it
                            ret_address[depth] = b'o';
                            // we found an object inside an object, so we need to increment the depth
                            depth += 1;
                            if depth > depthcapacity {
                                goto!(Fail);
                            }
                            goto!(ObjectBegin);
                        }
                        b'[' => {
                            //pj.containing_scope_offset[depth] = pj.get_current_loc();
                            //pj.write_tape(0, ItemType::Array); // here the compilers knows what c is so this gets optimized
                            // we have not yet encountered } so we need to come back for it
                            ret_address[depth] = b'o';
                            // we found an array inside an object, so we need to increment the depth
                            depth += 1;
                            if depth > depthcapacity {
                                goto!(Fail);
                            }
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
                            update_char!();
                            if c != b'"' {
                                goto!(Fail);
                            } else {
                                if self.parse_short_str_().is_err() {
                                    goto!(Fail);
                                }
                                goto!(ObjectKey);
                            }
                        }
                        b'}' => {
                            goto!(ScopeEnd(ItemType::Object));
                        }
                        _ => {
                            goto!(Fail);
                        }
                    }
                }

                ////////////////////////////// COMMON STATE /////////////////////////////
                ScopeEnd(_item_type) => {
                    // write our tape location to the header scope
                    depth -= 1;
                    /*
                    pj.write_tape(pj.containing_scope_offset[depth], item_type);
                    pj.annotate_previousloc(
                        pj.containing_scope_offset[depth],
                        pj.get_current_loc(),
                    );
                    */
                    // goto saved_state
                    if ret_address[depth] == b'a' {
                        goto!(ArrayContinue);
                    } else if ret_address[depth] == b'o' {
                        goto!(ObjectContinue);
                    } else {
                        goto!(StartContinue);
                    }
                }

                ////////////////////////////// ARRAY STATES /////////////////////////////
                ArrayBegin => {
                    update_char!();
                    if c == b']' {
                        goto!(ScopeEnd(ItemType::Array));
                    }
                    goto!(MainArraySwitch);
                }
                MainArraySwitch => {
                    // we call update char on all paths in, so we can peek at c on the
                    // on paths that can accept a close square brace (post-, and at start)
                    match c {
                        b'"' => {
                            if self.parse_str_().is_err() {
                                goto!(Fail);
                            }
                            goto!(ArrayContinue);
                        }
                        b't' => {
                            if !is_valid_true_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::True);
                            goto!(ArrayContinue);
                        }
                        b'f' => {
                            if !is_valid_false_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::False);
                            goto!(ArrayContinue);
                        }
                        b'n' => {
                            if !is_valid_null_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            //pj.write_tape(0, ItemType::Null);
                            goto!(ArrayContinue);
                        }
                        b'0'...b'9' => {
                            if self.parse_number_(false).is_err() {
                                goto!(Fail);
                            }
                            goto!(ArrayContinue);
                        }
                        b'-' => {
                            if self.parse_number_(true).is_err() {
                                goto!(Fail);
                            }
                            goto!(ArrayContinue);
                        }
                        b'{' => {
                            // we have not yet encountered ] so we need to come back for it
                            //pj.containing_scope_offset[depth] = pj.get_current_loc();
                            //pj.write_tape(0, ItemType::Object); //  here the compilers knows what c is so this gets optimized
                            ret_address[depth] = b'a';
                            // we found an object inside an array, so we need to increment the depth
                            depth += 1;
                            if depth > depthcapacity {
                                goto!(Fail);
                            }

                            goto!(ObjectBegin);
                        }
                        b'[' => {
                            // we have not yet encountered ] so we need to come back for it
                            //pj.containing_scope_offset[depth] = pj.get_current_loc();
                            //pj.write_tape(0, ItemType::Array); // here the compilers knows what c is so this gets optimized
                            ret_address[depth] = b'a';
                            // we found an array inside an array, so we need to increment the depth
                            depth += 1;
                            if depth > depthcapacity {
                                goto!(Fail);
                            }
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
                            update_char!();
                            goto!(MainArraySwitch);
                        }
                        b']' => {
                            goto!(ScopeEnd(ItemType::Array));
                        }
                        _c => {
                            goto!(Fail);
                        }
                    }
                }
                ////////////////////////////// FINAL STATES /////////////////////////////
                Succeed => {
                    depth -= 1;
                    if depth != 0 {
                        return Err(self.error(ErrorType::InternalError));
                    }
                    /*
                    if pj.containing_scope_offset[depth] != 0 {
                        return Err(ErrorType::InternalError);
                    }
                    pj.annotate_previousloc(
                        pj.containing_scope_offset[depth],
                        pj.get_current_loc(),
                    );
                    pj.write_tape(pj.containing_scope_offset[depth], ItemType::Root); // r is root
                     */

                    //pj.isvalid = true;
                    return Ok(Value::Null);
                }
                Fail => {
                    /*
                    dbg!(i);
                    dbg!(idx);
                    dbg!(c as char);
                    */
                    return Err(self.error(ErrorType::InternalError));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::to_tape;
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

    #[test]
    fn apache_builds() {
        use std::fs::File;
        use std::io::{self, Read, Write};
        let mut vec = Vec::new();
        File::open("data/apache_builds.json")
            .unwrap()
            .read_to_end(&mut vec)
            .unwrap();
        to_tape(&mut vec).unwrap();
    }
}
