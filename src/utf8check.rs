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
    pub input: T,
    pub prev: T,
    pub incomplete: T,
    pub error: T,
}

pub(crate) trait Utf8Check<T: Copy> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    unsafe fn check_bytes(
        current_bytes: T,
        previous: &ProcessedUtfBytes<T>,
    ) -> ProcessedUtfBytes<T> {
        let mut pb = ProcessedUtfBytes {
            input: current_bytes,
            prev: previous.input,
            incomplete: previous.incomplete,
            error: previous.error,
        };

        if Self::is_ascii(&mut pb) {
            Self::check_eof(&mut pb)
        } else {
            Self::check_utf8(&mut pb);
            pb.incomplete = Self::is_incomplete(&mut pb);
        }

        pb
    }

    unsafe fn check_utf8(pb: &mut ProcessedUtfBytes<T>) {
        let prev1 = Self::prev1(pb);
        let sc = Self::check_special_cases(pb, prev1);
        pb.error = Self::or(pb.error, Self::check_multibyte_lengths(pb, sc));
    }

    unsafe fn new() -> ProcessedUtfBytes<T>;
    unsafe fn or(a: T, b: T) -> T;
    unsafe fn is_ascii(pb: &mut ProcessedUtfBytes<T>) -> bool;
    unsafe fn check_eof(pb: &mut ProcessedUtfBytes<T>);
    unsafe fn is_incomplete(pb: &mut ProcessedUtfBytes<T>) -> T;
    unsafe fn prev1(pb: &mut ProcessedUtfBytes<T>) -> T;
    unsafe fn check_special_cases(pb: &mut ProcessedUtfBytes<T>, prev1: T) -> T;
    unsafe fn check_multibyte_lengths(pb: &mut ProcessedUtfBytes<T>, special_cases: T) -> T;
    unsafe fn must_be_2_3_continuation(prev2: T, prev3: T) -> T;
    unsafe fn has_error(pb: &ProcessedUtfBytes<T>) -> bool;
}
