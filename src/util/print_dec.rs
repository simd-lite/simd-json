// This is a modified version of the `itoa` crate by David Tolnay:
// https://github.com/dtolnay/itoa
//
// The crate itself borrows code from the stdlib of Rust.
//
// The algorithm here was modified from being able to just writing integers,
// to printing decimal floating points.

use crate::stry;
use std::{io, mem, ptr, slice};

const DEC_DIGITS_LUT: &[u8] = b"0001020304050607080910111213141516171819\
      2021222324252627282930313233343536373839\
      4041424344454647484950515253545556575859\
      6061626364656667686970717273747576777879\
      8081828384858687888990919293949596979899";

const ZEROFILL: &[u8] = &[b'0'; 20];

#[inline(always)]
unsafe fn write_num(n: &mut u64, curr: &mut isize, buf_ptr: *mut u8, lut_ptr: *const u8) {
    // eagerly decode 4 digits at a time
    while *n >= 10000 {
        let rem = (*n % 10000) as isize;
        *n /= 10000;

        let d1 = (rem / 100) << 1;
        let d2 = (rem % 100) << 1;
        *curr -= 4;
        ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(*curr), 2);
        ptr::copy_nonoverlapping(lut_ptr.offset(d2), buf_ptr.offset(*curr + 2), 2);
    }

    // decode 2 more digits
    if *n >= 100 {
        let d1 = ((*n % 100) << 1) as isize;
        *n /= 100;
        *curr -= 2;
        ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(*curr), 2);
    }

    // decode last 1 or 2 digits
    if *n < 10 {
        *curr -= 1;
        *buf_ptr.offset(*curr) = (*n as u8) + b'0';
    } else {
        let d1 = (*n << 1) as isize;
        *curr -= 2;
        ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(*curr), 2);
    }
}

pub unsafe fn write<W: io::Write>(
    wr: &mut W,
    positive: bool,
    mut n: u64,
    exponent: i16,
) -> io::Result<()> {
    if !positive {
        stry!(wr.write_all(b"-"));
    }

    if n == 0 {
        return wr.write_all(b"0");
    }

    let mut buf: [u8; 30] = mem::uninitialized();
    let mut curr = buf.len() as isize;
    let buf_ptr = buf.as_mut_ptr();
    let lut_ptr = DEC_DIGITS_LUT.as_ptr();

    if exponent == 0 {
        write_num(&mut n, &mut curr, buf_ptr, lut_ptr);

        return wr.write_all(slice::from_raw_parts(
            buf_ptr.offset(curr),
            buf.len() - curr as usize,
        ));
    } else if exponent < 0 {
        let mut e = safe_abs(exponent);

        // Decimal number with a fraction that's fully printable
        if e < 18 {
            // eagerly decode 4 digits at a time
            for _ in 0..e >> 2 {
                let rem = (n % 10000) as isize;
                n /= 10000;

                let d1 = (rem / 100) << 1;
                let d2 = (rem % 100) << 1;
                curr -= 4;
                ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
                ptr::copy_nonoverlapping(lut_ptr.offset(d2), buf_ptr.offset(curr + 2), 2);
            }

            e &= 3;

            // write the remaining 3, 2 or 1 digits
            if e & 2 == 2 {
                let d1 = ((n % 100) << 1) as isize;
                n /= 100;
                curr -= 2;
                ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
            }

            if e & 1 == 1 {
                curr -= 1;
                *buf_ptr.offset(curr) = ((n % 10) as u8) + b'0';
                n /= 10;
            }

            curr -= 1;
            *buf_ptr.offset(curr) = b'.';

            write_num(&mut n, &mut curr, buf_ptr, lut_ptr);

            return wr.write_all(slice::from_raw_parts(
                buf_ptr.offset(curr),
                buf.len() - curr as usize,
            ));
        }

        // Not easily printable, write down fraction, then full number, then exponent

        // Since we move the decimal point right after the first digit, we have to adjust the
        // exponent part. If the number is long enough, this may result in the exponent switching
        // sign from negative to positive - we have to handle this case separately.
        let mut exponent_positive = false;
        if n < 10 {
            // Single digit, no fraction
            curr -= 1;
            *buf_ptr.offset(curr) = ((n % 10) as u8) + b'0';
        } else {
            // eagerly decode 4 digits at a time
            while n >= 100_000 {
                let rem = (n % 10_000) as isize;
                n /= 10_000;

                let d1 = (rem / 100) << 1;
                let d2 = (rem % 100) << 1;
                curr -= 4;
                ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
                ptr::copy_nonoverlapping(lut_ptr.offset(d2), buf_ptr.offset(curr + 2), 2);
            }

            // decode 2 more digits
            if n >= 1_000 {
                let d1 = ((n % 100) << 1) as isize;
                n /= 100;
                curr -= 2;
                ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
            }

            // decode last 1 or 2 digits
            if n < 100 {
                curr -= 1;
                *buf_ptr.offset(curr) = ((n % 10) as u8) + b'0';
                n /= 10;
            } else {
                let d1 = ((n % 100) << 1) as isize;
                n /= 100;
                curr -= 2;
                ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
            }

            let printed_so_far = buf.len() as u16 - curr as u16;

            if printed_so_far <= e {
                // Subtract the amount of digits printed in the fraction
                // from the exponent that we still need to print using
                // the `e` notation
                e -= printed_so_far;
            } else {
                // Same as e = |e - printed_so_far|.
                e = printed_so_far - e;
                exponent_positive = true;
            }

            curr -= 1;
            *buf_ptr.offset(curr) = b'.';

            write_num(&mut n, &mut curr, buf_ptr, lut_ptr);
        }

        // Write out the number with a fraction
        stry!(wr.write_all(slice::from_raw_parts(
            buf_ptr.offset(curr),
            buf.len() - curr as usize
        )));

        // Omit the 'e' notation for e == 0
        if e == 0 {
            return Ok(());
        }
        // Write the remaining `e` notation, with proper sign
        if exponent_positive {
            wr.write_all(b"e+")?;
        } else {
            wr.write_all(b"e-")?;
        }
        return write(wr, true, u64::from(e), 0);
    }

    // Exponent greater than 0
    write_num(&mut n, &mut curr, buf_ptr, lut_ptr);
    let printed = buf.len() - curr as usize;

    // No need for `e` notation, just print out zeroes
    if (printed + exponent as usize) <= 20 {
        stry!(wr.write_all(slice::from_raw_parts(
            buf_ptr.offset(curr),
            buf.len() - curr as usize
        )));

        return wr.write_all(&ZEROFILL[..exponent as usize]);
    }

    let mut e = exponent as u64;

    // More than one digit, turn into a fraction
    if printed != 1 {
        *buf_ptr.offset(curr - 1) = *buf_ptr.offset(curr);
        *buf_ptr.offset(curr) = b'.';
        curr -= 1;
        e += (printed as u64) - 1;
    }

    stry!(wr.write_all(slice::from_raw_parts(
        buf_ptr.offset(curr),
        buf.len() - curr as usize
    )));
    stry!(wr.write_all(b"e"));
    write(wr, true, e, 0)
}

fn safe_abs(x: i16) -> u16 {
    if let Some(y) = x.checked_abs() {
        y as u16
    } else {
        i16::max_value() as u16 + 1u16
    }
}
