/*
 * legal utf-8 byte sequence
 * http://www.unicode.org/versions/Unicode6.0.0/ch03.pdf - page 94
 *
 *  Code Points        1st       2s       3s       4s
 * U+0000..U+007F     00..7F
 * U+0080..U+07FF     C2..DF   80..BF
 * U+0800..U+0FFF     E0       A0..BF   80..BF
 * U+1000..U+CFFF     E1..EC   80..BF   80..BF
 * U+D000..U+D7FF     ED       80..9F   80..BF
 * U+E000..U+FFFF     EE..EF   80..BF   80..BF
 * U+10000..U+3FFFF   F0       90..BF   80..BF   80..BF
 * U+40000..U+FFFFF   F1..F3   80..BF   80..BF   80..BF
 * U+100000..U+10FFFF F4       80..8F   80..BF   80..BF
 *
 */

pub(crate) struct ProcessedUtfBytes<T> {
    pub prev: T,
    pub incomplete: T,
    pub error: T,
}

pub(crate) trait Utf8Check<T: Copy> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    unsafe fn check_bytes(
        current: T,
        previous: T,
        incomplete: T,
        error: T,
    ) -> ProcessedUtfBytes<T> {
        let mut pb = ProcessedUtfBytes {
            prev: current,
            incomplete,
            error,
        };

        if Self::is_ascii(current) {
            pb.error = Self::check_eof(pb.error, pb.incomplete)
        } else {
            pb.error = Self::check_utf8(current, previous, pb.error);
            pb.incomplete = Self::is_incomplete(current);
        }

        pb
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    unsafe fn check_utf8(input: T, prev: T, error: T) -> T {
        let prev1 = Self::prev1(input, prev);
        let sc = Self::check_special_cases(input, prev1);
        Self::or(error, Self::check_multibyte_lengths(input, prev, sc))
    }

    unsafe fn new() -> ProcessedUtfBytes<T>;
    unsafe fn or(a: T, b: T) -> T;
    unsafe fn is_ascii(input: T) -> bool;
    unsafe fn check_eof(error: T, incomplete: T) -> T;
    unsafe fn is_incomplete(input: T) -> T;
    unsafe fn prev1(input: T, prev: T) -> T;
    unsafe fn check_special_cases(input: T, prev1: T) -> T;
    unsafe fn check_multibyte_lengths(input: T, prev: T, special_cases: T) -> T;
    unsafe fn must_be_2_3_continuation(prev2: T, prev3: T) -> T;
    unsafe fn has_error(error: T) -> bool;
}
