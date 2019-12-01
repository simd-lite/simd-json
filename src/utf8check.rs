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
    pub rawbytes: T,
    pub high_nibbles: T,
    pub carried_continuations: T,
}

pub(crate) trait Utf8Check<T: Copy> {
    fn new_processed_utf_bytes() -> ProcessedUtfBytes<T>;

    fn push_last_byte_of_a_to_b(a: T, b: T) -> T;

    fn push_last_2bytes_of_a_to_b(a: T, b: T) -> T;

    fn check_smaller_than_0xf4(current_bytes: T, has_error: &mut T);

    fn continuation_lengths(high_nibbles: T) -> T;

    fn carry_continuations(initial_lengths: T, previous_carries: T) -> T;

    fn check_continuations(initial_lengths: T, carries: T, has_error: &mut T);

    fn check_first_continuation_max(current_bytes: T, off1_current_bytes: T, has_error: &mut T);

    fn check_overlong(
        current_bytes: T,
        off1_current_bytes: T,
        hibits: T,
        previous_hibits: T,
        has_error: &mut T,
    );

    fn count_nibbles(bytes: T, answer: &mut ProcessedUtfBytes<T>);

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn check_utf8_bytes(
        current_bytes: T,
        previous: &ProcessedUtfBytes<T>,
        has_error: &mut T,
    ) -> ProcessedUtfBytes<T> {
        let mut pb: ProcessedUtfBytes<T> = Self::new_processed_utf_bytes();
        Self::count_nibbles(current_bytes, &mut pb);

        Self::check_smaller_than_0xf4(current_bytes, has_error);

        let initial_lengths = Self::continuation_lengths(pb.high_nibbles);

        pb.carried_continuations =
            Self::carry_continuations(initial_lengths, previous.carried_continuations);

        Self::check_continuations(initial_lengths, pb.carried_continuations, has_error);

        let off1_current_bytes = Self::push_last_byte_of_a_to_b(previous.rawbytes, pb.rawbytes);
        Self::check_first_continuation_max(current_bytes, off1_current_bytes, has_error);

        Self::check_overlong(
            current_bytes,
            off1_current_bytes,
            pb.high_nibbles,
            previous.high_nibbles,
            has_error,
        );
        pb
    }
}
