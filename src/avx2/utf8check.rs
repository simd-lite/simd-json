use crate::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

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

// all byte values must be no larger than 0xF4

/*****************************/
#[cfg_attr(not(feature = "no-inline"), inline)]
fn push_last_byte_of_a_to_b(a: __m256i, b: __m256i) -> __m256i {
    unsafe { _mm256_alignr_epi8(b, _mm256_permute2x128_si256(a, b, 0x21), 15) }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
fn push_last_2bytes_of_a_to_b(a: __m256i, b: __m256i) -> __m256i {
    unsafe { _mm256_alignr_epi8(b, _mm256_permute2x128_si256(a, b, 0x21), 14) }
}

// all byte values must be no larger than 0xF4
#[cfg_attr(not(feature = "no-inline"), inline)]
fn check_smaller_than_0xf4(current_bytes: __m256i, has_error: &mut __m256i) {
    // unsigned, saturates to 0 below max
    *has_error = unsafe {
        _mm256_or_si256(
            *has_error,
            _mm256_subs_epu8(current_bytes, _mm256_set1_epi8(static_cast_i8!(0xF4_u8))),
        )
    };
}

#[cfg_attr(not(feature = "no-inline"), inline)]
fn continuation_lengths(high_nibbles: __m256i) -> __m256i {
    unsafe {
        _mm256_shuffle_epi8(
            _mm256_setr_epi8(
                1, 1, 1, 1, 1, 1, 1, 1, // 0xxx (ASCII)
                0, 0, 0, 0, // 10xx (continuation)
                2, 2, // 110x
                3, // 1110
                4, // 1111, next should be 0 (not checked here)
                1, 1, 1, 1, 1, 1, 1, 1, // 0xxx (ASCII)
                0, 0, 0, 0, // 10xx (continuation)
                2, 2, // 110x
                3, // 1110
                4, // 1111, next should be 0 (not checked here)
            ),
            high_nibbles,
        )
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
unsafe fn carry_continuations(initial_lengths: __m256i, previous_carries: __m256i) -> __m256i {
    let right1: __m256i = _mm256_subs_epu8(
        push_last_byte_of_a_to_b(previous_carries, initial_lengths),
        _mm256_set1_epi8(1),
    );
    let sum: __m256i = _mm256_add_epi8(initial_lengths, right1);
    let right2: __m256i = _mm256_subs_epu8(
        push_last_2bytes_of_a_to_b(previous_carries, sum),
        _mm256_set1_epi8(2),
    );
    _mm256_add_epi8(sum, right2)
}

#[cfg_attr(not(feature = "no-inline"), inline)]
unsafe fn check_continuations(initial_lengths: __m256i, carries: __m256i, has_error: &mut __m256i) {
    // overlap || underlap
    // carry > length && length > 0 || !(carry > length) && !(length > 0)
    // (carries > length) == (lengths > 0)
    let overunder: __m256i = _mm256_cmpeq_epi8(
        _mm256_cmpgt_epi8(carries, initial_lengths),
        _mm256_cmpgt_epi8(initial_lengths, _mm256_setzero_si256()),
    );

    *has_error = _mm256_or_si256(*has_error, overunder);
}

// when 0xED is found, next byte must be no larger than 0x9F
// when 0xF4 is found, next byte must be no larger than 0x8F
// next byte must be continuation, ie sign bit is set, so signed < is ok
#[cfg_attr(not(feature = "no-inline"), inline)]
fn check_first_continuation_max(
    current_bytes: __m256i,
    off1_current_bytes: __m256i,
    has_error: &mut __m256i,
) {
    unsafe {
        let mask_ed: __m256i = _mm256_cmpeq_epi8(
            off1_current_bytes,
            _mm256_set1_epi8(static_cast_i8!(0xED_u8)),
        );
        let mask_f4: __m256i = _mm256_cmpeq_epi8(
            off1_current_bytes,
            _mm256_set1_epi8(static_cast_i8!(0xF4_u8)),
        );

        let badfollow_ed: __m256i = _mm256_and_si256(
            _mm256_cmpgt_epi8(current_bytes, _mm256_set1_epi8(static_cast_i8!(0x9F_u8))),
            mask_ed,
        );
        let badfollow_f4: __m256i = _mm256_and_si256(
            _mm256_cmpgt_epi8(current_bytes, _mm256_set1_epi8(static_cast_i8!(0x8F_u8))),
            mask_f4,
        );

        *has_error = _mm256_or_si256(*has_error, _mm256_or_si256(badfollow_ed, badfollow_f4));
    }
}

// map off1_hibits => error condition
// hibits     off1    cur
// C       => < C2 && true
// E       => < E1 && < A0
// F       => < F1 && < 90
// else      false && false
#[cfg_attr(not(feature = "no-inline"), inline)]
fn check_overlong(
    current_bytes: __m256i,
    off1_current_bytes: __m256i,
    hibits: __m256i,
    previous_hibits: __m256i,
    has_error: &mut __m256i,
) {
    unsafe {
        let off1_hibits: __m256i = push_last_byte_of_a_to_b(previous_hibits, hibits);
        let initial_mins: __m256i = _mm256_shuffle_epi8(
            _mm256_setr_epi8(
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128, // 10xx => false
                static_cast_i8!(0xC2_u8),
                -128,                     // 110x
                static_cast_i8!(0xE1_u8), // 1110
                static_cast_i8!(0xF1_u8), // 1111
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128, // 10xx => false
                static_cast_i8!(0xC2_u8),
                -128,                     // 110x
                static_cast_i8!(0xE1_u8), // 1110
                static_cast_i8!(0xF1_u8),
            ), // 1111
            off1_hibits,
        );

        let initial_under: __m256i = _mm256_cmpgt_epi8(initial_mins, off1_current_bytes);

        let second_mins: __m256i = _mm256_shuffle_epi8(
            _mm256_setr_epi8(
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128, // 10xx => false
                127,
                127,                      // 110x => true
                static_cast_i8!(0xA0_u8), // 1110
                static_cast_i8!(0x90_u8), // 1111
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128,
                -128, // 10xx => false
                127,
                127,                      // 110x => true
                static_cast_i8!(0xA0_u8), // 1110
                static_cast_i8!(0x90_u8),
            ), // 1111
            off1_hibits,
        );
        let second_under: __m256i = _mm256_cmpgt_epi8(second_mins, current_bytes);
        *has_error = _mm256_or_si256(*has_error, _mm256_and_si256(initial_under, second_under));
    }
}

impl Default for ProcessedUtfBytes<__m256i> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn default() -> Self {
        unsafe {
            Self {
                rawbytes: _mm256_setzero_si256(),
                high_nibbles: _mm256_setzero_si256(),
                carried_continuations: _mm256_setzero_si256(),
            }
        }
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
fn count_nibbles(bytes: __m256i, answer: &mut ProcessedUtfBytes<__m256i>) {
    answer.rawbytes = bytes;
    answer.high_nibbles =
        unsafe { _mm256_and_si256(_mm256_srli_epi16(bytes, 4), _mm256_set1_epi8(0x0F)) };
}

// check whether the current bytes are valid UTF-8
// at the end of the function, previous gets updated
#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) fn check_utf8_bytes(
    current_bytes: __m256i,
    previous: &ProcessedUtfBytes<__m256i>,
    has_error: &mut __m256i,
) -> ProcessedUtfBytes<__m256i> {
    let mut pb = ProcessedUtfBytes::<__m256i>::default();
    unsafe {
        count_nibbles(current_bytes, &mut pb);

        check_smaller_than_0xf4(current_bytes, has_error);

        let initial_lengths: __m256i = continuation_lengths(pb.high_nibbles);

        pb.carried_continuations =
            carry_continuations(initial_lengths, previous.carried_continuations);

        check_continuations(initial_lengths, pb.carried_continuations, has_error);

        let off1_current_bytes: __m256i = push_last_byte_of_a_to_b(previous.rawbytes, pb.rawbytes);
        check_first_continuation_max(current_bytes, off1_current_bytes, has_error);

        check_overlong(
            current_bytes,
            off1_current_bytes,
            pb.high_nibbles,
            previous.high_nibbles,
            has_error,
        );
        pb
    }
}
