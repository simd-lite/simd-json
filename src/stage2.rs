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

    ScopeEnd,

    ArrayBegin,
    MainArraySwitch,
    ArrayContinue,

    Succeed,
    Fail,
}

impl<'de> Deserializer<'de> {
    pub unsafe fn unified_machine(&mut self) -> Result<value! {}> {
        //        let mut i: usize = 0; // index of the structural character (0,1,2,3...)
        let mut idx: usize; // location of the structural character in the input (buf)
        let mut c: u8; // used to track the (structural) character we are looking at, updated
                       // by UPDATE_CHAR macro
        let mut depth: usize = 0; // could have an arbitrary starting depth

        // this macro reads the next structural character, updating idx, i and c.
        macro_rules! update_char {
            () => {
                c = self.next_();
                idx = self.iidx;
            };
        }

        // placehodlers
        let mut ret_address = vec![0; 100];
        let depthcapacity = 100;
        let len = self.input.len();

        let mut keys: Vec<&str> = Vec::with_capacity(depthcapacity);
        keys.push("---illegal json---");
        let mut objects: Vec<Map> = Vec::with_capacity(depthcapacity);
        objects.push(Map::new());
        let mut arrays: Vec<Vec<Value>> = Vec::with_capacity(depthcapacity);
        arrays.push(Vec::new());
        let mut current_key: &str = "---illegal json---";
        let mut current: Value = Value::Null;
        let mut current_object = &mut objects[0];
        let mut current_array = &mut arrays[0];

        ////////////////////////////// START STATE /////////////////////////////
        ret_address[depth] = b's';
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
                            objects.push(Map::with_capacity(self.count_elements()));
                            let l = objects.len();
                            current_object = &mut objects[l - 1];
                            keys.push(current_key);
                            ret_address[depth] = b's';
                            depth += 1;
                            if depth > depthcapacity {
                                goto!(Fail);
                            }
                            goto!(ObjectBegin);
                        }
                        b'[' => {
                            arrays.push(Vec::with_capacity(self.count_elements()));
                            let l = arrays.len();
                            current_array = &mut arrays[l - 1];
                            ret_address[depth] = b's';
                            depth += 1;
                            if depth > depthcapacity {
                                goto!(Fail);
                            }
                            goto!(ArrayBegin);
                        }
                        b'"' => {
                            if let Ok(s) = self.parse_str_() {
                                current = Value::from(s);
                                goto!(StartContinue);
                            } else {
                                goto!(Fail);
                            }
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
                            current = Value::Bool(true);
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
                            current = Value::Bool(false);
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
                            current = Value::Null;
                            goto!(StartContinue);
                        }
                        b'0'...b'9' => {
                            // we need to make a copy to make sure that the string is NULL terminated.
                            // this is done only for JSON documents made of a sole number
                            // this will almost never be called in practice
                            if let Ok(n) = self.parse_number_(false) {
                                current = n;
                                goto!(StartContinue);
                            } else {
                                goto!(Fail);
                            }
                        }
                        b'-' => {
                            // we need to make a copy to make sure that the string is NULL terminated.
                            // this is done only for JSON documents made of a sole number
                            // this will almost never be called in practice
                            if let Ok(n) = self.parse_number_(true) {
                                current = n;
                                goto!(StartContinue);
                            } else {
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
                            if let Ok(s) = self.parse_short_str_() {
                                current_key = s;
                                goto!(ObjectKey);
                            } else {
                                goto!(Fail);
                            }
                        }
                        b'}' => {
                            current = Value::Object(objects.pop().unwrap());
                            let l = objects.len();
                            current_object = &mut objects[l - 1];
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
                            if let Ok(s) = self.parse_short_str_() {
                                #[cfg(not(feature = "no-borrow"))]
                                current_object.insert(current_key, Value::from(s));
                                #[cfg(feature = "no-borrow")]
                                current_object.insert(current_key.to_owned(), Value::from(s));
                                goto!(ObjectContinue);
                            } else {
                                goto!(Fail);
                            }
                        }
                        b't' => {
                            if !is_valid_true_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            #[cfg(not(feature = "no-borrow"))]
                            current_object.insert(current_key, Value::from(true));
                            #[cfg(feature = "no-borrow")]
                            current_object.insert(current_key.to_owned(), Value::from(true));
                            goto!(ObjectContinue);
                        }
                        b'f' => {
                            if !is_valid_false_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            #[cfg(not(feature = "no-borrow"))]
                            current_object.insert(current_key, Value::from(false));
                            #[cfg(feature = "no-borrow")]
                            current_object.insert(current_key.to_owned(), Value::from(false));
                            goto!(ObjectContinue);
                        }
                        b'n' => {
                            if !is_valid_null_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            #[cfg(not(feature = "no-borrow"))]
                            current_object.insert(current_key, Value::Null);
                            #[cfg(feature = "no-borrow")]
                            current_object.insert(current_key.to_owned(), Value::Null);
                            goto!(ObjectContinue);
                        }
                        b'0'...b'9' => {
                            if let Ok(n) = self.parse_number_(false) {
                                #[cfg(not(feature = "no-borrow"))]
                                current_object.insert(current_key, n);
                                #[cfg(feature = "no-borrow")]
                                current_object.insert(current_key.to_owned(), n);
                                goto!(ObjectContinue);
                            } else {
                                goto!(Fail);
                            }
                        }
                        b'-' => {
                            if let Ok(n) = self.parse_number_(true) {
                                #[cfg(not(feature = "no-borrow"))]
                                current_object.insert(current_key, n);
                                #[cfg(feature = "no-borrow")]
                                current_object.insert(current_key.to_owned(), n);
                                goto!(ObjectContinue);
                            } else {
                                goto!(Fail);
                            }
                        }
                        b'{' => {
                            // we have not yet encountered } so we need to come back for it
                            objects.push(Map::with_capacity(self.count_elements()));
                            let l = objects.len();
                            current_object = &mut objects[l - 1];
                            keys.push(current_key);
                            ret_address[depth] = b'o';
                            // we found an object inside an object, so we need to increment the depth
                            depth += 1;
                            if depth > depthcapacity {
                                goto!(Fail);
                            }
                            goto!(ObjectBegin);
                        }
                        b'[' => {
                            // we have not yet encountered } so we need to come back for it
                            arrays.push(Vec::with_capacity(self.count_elements()));
                            let l = arrays.len();
                            current_array = &mut arrays[l - 1];
                            keys.push(current_key);
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
                                if let Ok(s) = self.parse_short_str_() {
                                    current_key = s;
                                    goto!(ObjectKey);
                                } else {
                                    goto!(Fail);
                                }
                            }
                        }
                        b'}' => {
                            current = Value::Object(objects.pop().unwrap());
                            let l = objects.len();
                            current_object = &mut objects[l - 1];
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
                    // goto saved_state
                    if ret_address[depth] == b'a' {
                        let mut we_got_to_do_this_for_ownership_meh = Value::Null;
                        mem::swap(&mut current, &mut we_got_to_do_this_for_ownership_meh);
                        &current_array.push(we_got_to_do_this_for_ownership_meh);
                        goto!(ArrayContinue);
                    } else if ret_address[depth] == b'o' {
                        let mut we_got_to_do_this_for_ownership_meh = Value::Null;
                        mem::swap(&mut current, &mut we_got_to_do_this_for_ownership_meh);
                        current_key = keys.pop().unwrap();
                        #[cfg(not(feature = "no-borrow"))]
                        current_object.insert(current_key, we_got_to_do_this_for_ownership_meh);
                        #[cfg(feature = "no-borrow")]
                        current_object
                            .insert(current_key.to_owned(), we_got_to_do_this_for_ownership_meh);
                        goto!(ObjectContinue);
                    } else {
                        goto!(StartContinue);
                    }
                }

                ////////////////////////////// ARRAY STATES /////////////////////////////
                ArrayBegin => {
                    update_char!();
                    if c == b']' {
                        current = Value::Array(arrays.pop().unwrap());
                        let l = arrays.len();
                        current_array = &mut arrays[l - 1];
                        goto!(ScopeEnd);
                    }
                    goto!(MainArraySwitch);
                }
                MainArraySwitch => {
                    // we call update char on all paths in, so we can peek at c on the
                    // on paths that can accept a close square brace (post-, and at start)
                    match c {
                        b'"' => {
                            if let Ok(s) = self.parse_str_() {
                                &current_array.push(Value::from(s));
                            } else {
                                goto!(Fail);
                            }
                            goto!(ArrayContinue);
                        }
                        b't' => {
                            if !is_valid_true_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            &current_array.push(Value::Bool(true));
                            goto!(ArrayContinue);
                        }
                        b'f' => {
                            if !is_valid_false_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            &current_array.push(Value::Bool(false));
                            goto!(ArrayContinue);
                        }
                        b'n' => {
                            if !is_valid_null_atom(&self.input[idx..]) {
                                goto!(Fail);
                            }
                            &current_array.push(Value::Null);
                            goto!(ArrayContinue);
                        }
                        b'0'...b'9' => {
                            if let Ok(n) = self.parse_number_(false) {
                                &current_array.push(n);
                                goto!(ArrayContinue);
                            } else {
                                goto!(Fail);
                            }
                        }
                        b'-' => {
                            if let Ok(n) = self.parse_number_(true) {
                                &current_array.push(n);
                                goto!(ArrayContinue);
                            } else {
                                goto!(Fail);
                            }
                        }
                        b'{' => {
                            // we have not yet encountered ] so we need to come back for it
                            objects.push(Map::with_capacity(self.count_elements()));
                            let l = objects.len();
                            current_object = &mut objects[l - 1];
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
                            arrays.push(Vec::with_capacity(self.count_elements()));
                            let l = arrays.len();
                            current_array = &mut arrays[l - 1];
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
                            current = Value::Array(arrays.pop().unwrap());
                            let l = arrays.len();
                            current_array = &mut arrays[l - 1];
                            goto!(ScopeEnd);
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
                    return Ok(current);
                }
                Fail => {
                    return Err(self.error(ErrorType::InternalError));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Read;

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
        let mut vec = Vec::new();
        File::open("data/apache_builds.json")
            .unwrap()
            .read_to_end(&mut vec)
            .unwrap();
        to_value(&mut vec).unwrap();
    }

    #[test]
    fn citm_catalog() {
        let mut vec = Vec::new();
        File::open("data/citm_catalog.json")
            .unwrap()
            .read_to_end(&mut vec)
            .unwrap();
        to_value(&mut vec).unwrap();
    }
}
