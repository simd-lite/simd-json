use crate::*;
use simd_lite::aarch64::*;
use simd_lite::NeonInit;

/*
 * legal utf-8 byte sequence
 * http://www.unicode.org/versions/Unicode6.0.0/ch03.pdf - page 94
 *
 *  Code Points        1st       2s       3s       4s
 *  U+0000..U+007F     00..7F
 *  U+0080..U+07FF     C2..DF   80..BF
 *  U+0800..U+0FFF     E0       A0..BF   80..BF
 *  U+1000..U+CFFF     E1..EC   80..BF   80..BF
 *  U+D000..U+D7FF     ED       80..9F   80..BF
 *  U+E000..U+FFFF     EE..EF   80..BF   80..BF
 *  U+10000..U+3FFFF   F0       90..BF   80..BF   80..BF
 *  U+40000..U+FFFFF   F1..F3   80..BF   80..BF   80..BF
 *  U+100000..U+10FFFF F4       80..8F   80..BF   80..BF
 *
 */

/*****************************/
#[cfg_attr(not(feature = "no-inline"), inline)]
fn push_last_byte_of_a_to_b(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    unsafe { vextq_s8(a, b, 16 - 1) }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
fn push_last_2bytes_of_a_to_b(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    unsafe { vextq_s8(a, b, 16 - 2) }
}

// all byte values must be no larger than 0xF4
#[cfg_attr(not(feature = "no-inline"), inline)]
fn check_smaller_than_0xf4(current_bytes: int8x16_t, has_error: &mut int8x16_t) {
    // unsigned, saturates to 0 below max
    *has_error = unsafe {
        vorrq_s8(
            *has_error,
            vreinterpretq_s8_u8(vqsubq_u8(
                vreinterpretq_u8_s8(current_bytes),
                vdupq_n_u8(0xF4),
            )),
        )
    };
}

macro_rules! nibbles_tbl {
    () => {
        int8x16_t::new([
            1i8, 1, 1, 1, 1, 1, 1, 1, // 0xxx (ASCII)
            0, 0, 0, 0, // 10xx (continuation)
            2, 2, // 110x
            3, // 1110
            4, // 1111, next should be 0 (not checked here)
        ])
    };
}

#[cfg_attr(not(feature = "no-inline"), inline)]
unsafe fn continuation_lengths(high_nibbles: int8x16_t) -> int8x16_t {
    vqtbl1q_s8(nibbles_tbl!(), vreinterpretq_u8_s8(high_nibbles))
}

#[cfg_attr(not(feature = "no-inline"), inline)]
unsafe fn carry_continuations(
    initial_lengths: int8x16_t,
    previous_carries: int8x16_t,
) -> int8x16_t {
    let right1: int8x16_t = vreinterpretq_s8_u8(vqsubq_u8(
        vreinterpretq_u8_s8(push_last_byte_of_a_to_b(previous_carries, initial_lengths)),
        vdupq_n_u8(1),
    ));
    let sum: int8x16_t = vaddq_s8(initial_lengths, right1);
    let right2: int8x16_t = vreinterpretq_s8_u8(vqsubq_u8(
        vreinterpretq_u8_s8(push_last_2bytes_of_a_to_b(previous_carries, sum)),
        vdupq_n_u8(2),
    ));
    vaddq_s8(sum, right2)
}

#[cfg_attr(not(feature = "no-inline"), inline)]
unsafe fn check_continuations(
    initial_lengths: int8x16_t,
    carries: int8x16_t,
    has_error: &mut int8x16_t,
) {
    // overlap || underlap
    // carry > length && length > 0 || !(carry > length) && !(length > 0)
    // (carries > length) == (lengths > 0)
    let overunder: uint8x16_t = vceqq_u8(
        vcgtq_s8(carries, initial_lengths),
        vcgtq_s8(initial_lengths, vdupq_n_s8(0)),
    );

    *has_error = vorrq_s8(*has_error, vreinterpretq_s8_u8(overunder));
}

// when 0xED is found, next byte must be no larger than 0x9F
// when 0xF4 is found, next byte must be no larger than 0x8F
// next byte must be continuation, ie sign bit is set, so signed < is ok
#[cfg_attr(not(feature = "no-inline"), inline)]
fn check_first_continuation_max(
    current_bytes: int8x16_t,
    off1_current_bytes: int8x16_t,
    has_error: &mut int8x16_t,
) {
    unsafe {
        let mask_ed: uint8x16_t = vceqq_s8(off1_current_bytes, vdupq_n_s8(-19 /* 0xED */));
        let mask_f4: uint8x16_t = vceqq_s8(off1_current_bytes, vdupq_n_s8(-12 /* 0xF4 */));

        let badfollow_ed: uint8x16_t =
            vandq_u8(vcgtq_s8(current_bytes, vdupq_n_s8(-97 /* 0x9F */)), mask_ed);
        let badfollow_f4: uint8x16_t = vandq_u8(
            vcgtq_s8(current_bytes, vdupq_n_s8(-113 /* 0x8F */)),
            mask_f4,
        );

        *has_error = vorrq_s8(
            *has_error,
            vreinterpretq_s8_u8(vorrq_u8(badfollow_ed, badfollow_f4)),
        );
    }
}

macro_rules! initial_mins_tbl {
    () => {
        int8x16_t::new([
            -128i8, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128,
            -128, // 10xx => false
            -62,  // 0xC2
            -128, // 110x
            -31,  // 0xE1 => 1110
            -15,  // 0xF1 => 1111
        ])
    };
}

macro_rules! second_mins_tbl {
    () => {
        int8x16_t::new([
            -128i8, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128,
            -128, // 10xx => false
            127, 127,  // 110x => true
            -96,  // 0xA0 => 1110
            -112, // 0x90 => 1111
        ])
    };
}

// map off1_hibits => error condition
// hibits     off1    cur
// C       => < C2 && true
// E       => < E1 && < A0
// F       => < F1 && < 90
// else      false && false
#[cfg_attr(not(feature = "no-inline"), inline)]
fn check_overlong(
    current_bytes: int8x16_t,
    off1_current_bytes: int8x16_t,
    hibits: int8x16_t,
    previous_hibits: int8x16_t,
    has_error: &mut int8x16_t,
) {
    unsafe {
        let off1_hibits: int8x16_t = push_last_byte_of_a_to_b(previous_hibits, hibits);
        let initial_mins: int8x16_t =
            vqtbl1q_s8(initial_mins_tbl!(), vreinterpretq_u8_s8(off1_hibits));

        let initial_under: uint8x16_t = vcgtq_s8(initial_mins, off1_current_bytes);

        let second_mins: int8x16_t =
            vqtbl1q_s8(second_mins_tbl!(), vreinterpretq_u8_s8(off1_hibits));
        let second_under: uint8x16_t = vcgtq_s8(second_mins, current_bytes);
        *has_error = vorrq_s8(
            *has_error,
            vreinterpretq_s8_u8(vandq_u8(initial_under, second_under)),
        );
    }
}

impl Default for ProcessedUtfBytes<int8x16_t> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn default() -> Self {
        unsafe {
            Self {
                rawbytes: vdupq_n_s8(0x00),
                high_nibbles: vdupq_n_s8(0x00),
                carried_continuations: vdupq_n_s8(0x00),
            }
        }
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
fn count_nibbles(bytes: int8x16_t, answer: &mut ProcessedUtfBytes<int8x16_t>) {
    answer.rawbytes = bytes;
    answer.high_nibbles = unsafe {
        vandq_s8(
            vreinterpretq_s8_u8(vshrq_n_u8(vreinterpretq_u8_s8(bytes), 4)),
            vmovq_n_s8(0x0F),
        )
    };
}

// check whether the current bytes are valid UTF-8
// at the end of the function, previous gets updated
#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) fn check_utf8_bytes(
    current_bytes: int8x16_t,
    previous: &mut ProcessedUtfBytes<int8x16_t>,
    has_error: &mut int8x16_t,
) -> ProcessedUtfBytes<int8x16_t> {
    let mut pb = ProcessedUtfBytes::default();
    unsafe {
        count_nibbles(current_bytes, &mut pb);

        check_smaller_than_0xf4(current_bytes, has_error);

        let initial_lengths: int8x16_t = continuation_lengths(pb.high_nibbles);

        pb.carried_continuations =
            carry_continuations(initial_lengths, previous.carried_continuations);

        check_continuations(initial_lengths, pb.carried_continuations, has_error);

        let off1_current_bytes: int8x16_t =
            push_last_byte_of_a_to_b(previous.rawbytes, pb.rawbytes);
        check_first_continuation_max(current_bytes, off1_current_bytes, has_error);

        check_overlong(
            current_bytes,
            off1_current_bytes,
            pb.high_nibbles,
            previous.high_nibbles,
            has_error,
        );
    }
    pb
}
