#![allow(dead_code)]
use crate::charutils::*;
//use crate::portability::*;

//#[inline(always)]
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

//#[inline(always)]
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

//#[inline(always)]
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
