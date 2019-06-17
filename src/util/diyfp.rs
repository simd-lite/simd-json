// This file comes from the `dtoa` port by David Tolnay:
// https://github.com/dtolnay/dtoa
//
// Copyright 2016 Dtoa Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{mem, ops};

const DIY_SIGNIFICAND_SIZE: isize = 64;
const DP_SIGNIFICAND_SIZE: isize = 52;
const DP_EXPONENT_BIAS: isize = 0x3FF + DP_SIGNIFICAND_SIZE;
const DP_MIN_EXPONENT: isize = -DP_EXPONENT_BIAS;
const DP_EXPONENT_MASK: u64 = 0x7FF0000000000000;
const DP_SIGNIFICAND_MASK: u64 = 0x000FFFFFFFFFFFFF;
const DP_HIDDEN_BIT: u64 = 0x0010000000000000;

#[derive(Copy, Clone, Debug)]
pub struct DiyFp {
    pub f: u64,
    pub e: isize,
}

impl DiyFp {
    pub fn new(f: u64, e: isize) -> Self {
        DiyFp { f: f, e: e }
    }

    /*
    explicit DiyFp(double d) {
        union {
            double d;
            uint64_t u64;
        } u = { d };

        int biased_e = static_cast<int>((u.u64 & kDpExponentMask) >> kDpSignificandSize);
        uint64_t significand = (u.u64 & kDpSignificandMask);
        if (biased_e != 0) {
            f = significand + kDpHiddenBit;
            e = biased_e - kDpExponentBias;
        }
        else {
            f = significand;
            e = kDpMinExponent + 1;
        }
    }
    */
    pub unsafe fn from_f64(d: f64) -> Self {
        let u: u64 = mem::transmute(d);

        let biased_e = ((u & DP_EXPONENT_MASK) >> DP_SIGNIFICAND_SIZE) as isize;
        let significand = u & DP_SIGNIFICAND_MASK;
        if biased_e != 0 {
            DiyFp {
                f: significand + DP_HIDDEN_BIT,
                e: biased_e - DP_EXPONENT_BIAS,
            }
        } else {
            DiyFp {
                f: significand,
                e: DP_MIN_EXPONENT + 1,
            }
        }
    }

    /*
    DiyFp Normalize() const {
        DiyFp res = *this;
        while (!(res.f & (static_cast<uint64_t>(1) << 63))) {
            res.f <<= 1;
            res.e--;
        }
        return res;
    }
    */
    pub fn normalize(self) -> DiyFp {
        let mut res = self;
        while (res.f & (1u64 << 63)) == 0 {
            res.f <<= 1;
            res.e -= 1;
        }
        res
    }

    /*
    DiyFp NormalizeBoundary() const {
        DiyFp res = *this;
        while (!(res.f & (kDpHiddenBit << 1))) {
            res.f <<= 1;
            res.e--;
        }
        res.f <<= (kDiySignificandSize - kDpSignificandSize - 2);
        res.e = res.e - (kDiySignificandSize - kDpSignificandSize - 2);
        return res;
    }
    */
    pub fn normalize_boundary(self) -> DiyFp {
        let mut res = self;
        while (res.f & DP_HIDDEN_BIT << 1) == 0 {
            res.f <<= 1;
            res.e -= 1;
        }
        res.f <<= DIY_SIGNIFICAND_SIZE - DP_SIGNIFICAND_SIZE - 2;
        res.e -= DIY_SIGNIFICAND_SIZE - DP_SIGNIFICAND_SIZE - 2;
        res
    }

    /*
    void NormalizedBoundaries(DiyFp* minus, DiyFp* plus) const {
        DiyFp pl = DiyFp((f << 1) + 1, e - 1).NormalizeBoundary();
        DiyFp mi = (f == kDpHiddenBit) ? DiyFp((f << 2) - 1, e - 2) : DiyFp((f << 1) - 1, e - 1);
        mi.f <<= mi.e - pl.e;
        mi.e = pl.e;
        *plus = pl;
        *minus = mi;
    }
    */
    pub fn normalized_boundaries(self) -> (DiyFp, DiyFp) {
        let pl = DiyFp::new((self.f << 1) + 1, self.e - 1).normalize_boundary();
        let mut mi = if self.f == DP_HIDDEN_BIT {
            DiyFp::new((self.f << 2) - 1, self.e - 2)
        } else {
            DiyFp::new((self.f << 1) - 1, self.e - 1)
        };
        mi.f <<= mi.e - pl.e;
        mi.e = pl.e;
        (mi, pl)
    }
}

impl ops::Sub for DiyFp {
    type Output = DiyFp;
    fn sub(self, rhs: DiyFp) -> DiyFp {
        DiyFp {
            f: self.f - rhs.f,
            e: self.e,
        }
    }
}

impl ops::Mul for DiyFp {
    type Output = DiyFp;
    fn mul(self, rhs: DiyFp) -> DiyFp {
        let m32 = 0xFFFFFFFFu64;
        let a = self.f >> 32;
        let b = self.f & m32;
        let c = rhs.f >> 32;
        let d = rhs.f & m32;
        let ac = a * c;
        let bc = b * c;
        let ad = a * d;
        let bd = b * d;
        let mut tmp = (bd >> 32) + (ad & m32) + (bc & m32);
        tmp += 1u64 << 31; // mult_round
        DiyFp {
            f: ac + (ad >> 32) + (bc >> 32) + (tmp >> 32),
            e: self.e + rhs.e + 64,
        }
    }
}

fn get_cached_power_by_index(index: usize) -> DiyFp {
    // 10^-348, 10^-340, ..., 10^340
    static CACHED_POWERS_F: [u64; 87] = [
        0xfa8fd5a0081c0288,
        0xbaaee17fa23ebf76,
        0x8b16fb203055ac76,
        0xcf42894a5dce35ea,
        0x9a6bb0aa55653b2d,
        0xe61acf033d1a45df,
        0xab70fe17c79ac6ca,
        0xff77b1fcbebcdc4f,
        0xbe5691ef416bd60c,
        0x8dd01fad907ffc3c,
        0xd3515c2831559a83,
        0x9d71ac8fada6c9b5,
        0xea9c227723ee8bcb,
        0xaecc49914078536d,
        0x823c12795db6ce57,
        0xc21094364dfb5637,
        0x9096ea6f3848984f,
        0xd77485cb25823ac7,
        0xa086cfcd97bf97f4,
        0xef340a98172aace5,
        0xb23867fb2a35b28e,
        0x84c8d4dfd2c63f3b,
        0xc5dd44271ad3cdba,
        0x936b9fcebb25c996,
        0xdbac6c247d62a584,
        0xa3ab66580d5fdaf6,
        0xf3e2f893dec3f126,
        0xb5b5ada8aaff80b8,
        0x87625f056c7c4a8b,
        0xc9bcff6034c13053,
        0x964e858c91ba2655,
        0xdff9772470297ebd,
        0xa6dfbd9fb8e5b88f,
        0xf8a95fcf88747d94,
        0xb94470938fa89bcf,
        0x8a08f0f8bf0f156b,
        0xcdb02555653131b6,
        0x993fe2c6d07b7fac,
        0xe45c10c42a2b3b06,
        0xaa242499697392d3,
        0xfd87b5f28300ca0e,
        0xbce5086492111aeb,
        0x8cbccc096f5088cc,
        0xd1b71758e219652c,
        0x9c40000000000000,
        0xe8d4a51000000000,
        0xad78ebc5ac620000,
        0x813f3978f8940984,
        0xc097ce7bc90715b3,
        0x8f7e32ce7bea5c70,
        0xd5d238a4abe98068,
        0x9f4f2726179a2245,
        0xed63a231d4c4fb27,
        0xb0de65388cc8ada8,
        0x83c7088e1aab65db,
        0xc45d1df942711d9a,
        0x924d692ca61be758,
        0xda01ee641a708dea,
        0xa26da3999aef774a,
        0xf209787bb47d6b85,
        0xb454e4a179dd1877,
        0x865b86925b9bc5c2,
        0xc83553c5c8965d3d,
        0x952ab45cfa97a0b3,
        0xde469fbd99a05fe3,
        0xa59bc234db398c25,
        0xf6c69a72a3989f5c,
        0xb7dcbf5354e9bece,
        0x88fcf317f22241e2,
        0xcc20ce9bd35c78a5,
        0x98165af37b2153df,
        0xe2a0b5dc971f303a,
        0xa8d9d1535ce3b396,
        0xfb9b7cd9a4a7443c,
        0xbb764c4ca7a44410,
        0x8bab8eefb6409c1a,
        0xd01fef10a657842c,
        0x9b10a4e5e9913129,
        0xe7109bfba19c0c9d,
        0xac2820d9623bf429,
        0x80444b5e7aa7cf85,
        0xbf21e44003acdd2d,
        0x8e679c2f5e44ff8f,
        0xd433179d9c8cb841,
        0x9e19db92b4e31ba9,
        0xeb96bf6ebadf77d9,
        0xaf87023b9bf0ee6b,
    ];
    static CACHED_POWERS_E: [i16; 87] = [
        -1220, -1193, -1166, -1140, -1113, -1087, -1060, -1034, -1007, -980, -954, -927, -901,
        -874, -847, -821, -794, -768, -741, -715, -688, -661, -635, -608, -582, -555, -529, -502,
        -475, -449, -422, -396, -369, -343, -316, -289, -263, -236, -210, -183, -157, -130, -103,
        -77, -50, -24, 3, 30, 56, 83, 109, 136, 162, 189, 216, 242, 269, 295, 322, 348, 375, 402,
        428, 455, 481, 508, 534, 561, 588, 614, 641, 667, 694, 720, 747, 774, 800, 827, 853, 880,
        907, 933, 960, 986, 1013, 1039, 1066,
    ];
    DiyFp::new(CACHED_POWERS_F[index], CACHED_POWERS_E[index] as isize)
}

/*
inline DiyFp GetCachedPower(int e, int* K) {
    //int k = static_cast<int>(ceil((-61 - e) * 0.30102999566398114)) + 374;
    double dk = (-61 - e) * 0.30102999566398114 + 347;  // dk must be positive, so can do ceiling in positive
    int k = static_cast<int>(dk);
    if (dk - k > 0.0)
        k++;

    unsigned index = static_cast<unsigned>((k >> 3) + 1);
    *K = -(-348 + static_cast<int>(index << 3));    // decimal exponent no need lookup table

    return GetCachedPowerByIndex(index);
}
*/
#[inline]
pub fn get_cached_power(e: isize) -> (DiyFp, isize) {
    let dk = (-61 - e) as f64 * 0.30102999566398114f64 + 347f64; // dk must be positive, so can do ceiling in positive
    let mut k = dk as isize;
    if dk - k as f64 > 0.0 {
        k += 1;
    }

    let index = ((k >> 3) + 1) as usize;
    let k = -(-348 + (index << 3) as isize); // decimal exponent no need lookup table

    (get_cached_power_by_index(index), k)
}
