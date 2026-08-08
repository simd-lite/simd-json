//! Equivalence and soundness harness for `fill_tape_padded`.
//!
//! `fill_tape_padded` parses out of the caller's own buffer, so string unescaping writes
//! into the same bytes stage 2 reads from. That aliasing is the one thing this path has
//! that `fill_tape` does not, so every test here pins the padded result against the
//! copying `fill_tape` on the same document, and the layout mirrors the intended caller
//! shape: several rows packed into one padded scratch, `Buffers` reused across rows.
//!
//! Worth running under Miri after any change to stage 2 or the per-ISA `parse_str`, in
//! both borrow models and on a second architecture to cover the SIMD and scalar backends:
//!
//! ```text
//! cargo +nightly miri test --test fill_tape_padded
//! MIRIFLAGS=-Zmiri-tree-borrows cargo +nightly miri test --test fill_tape_padded
//! ```

use simd_json::{Buffers, INPUT_PADDING, Tape, fill_tape, fill_tape_padded};

/// Packs `rows` into one scratch buffer with a single trailing padding run, parses each
/// row through `fill_tape_padded`, and returns per-row `Ok(debug of nodes)` /
/// `Err(debug of error)`.
///
/// Every row except the last is followed by the next row's bytes rather than by padding,
/// which is what makes this layout worth testing: row N's SIMD over-read reaches into row
/// N+1, and row N+1 is parsed after row N has already unescaped in place.
fn parse_rows(rows: &[&[u8]]) -> Vec<String> {
    let mut scratch: Vec<u8> = Vec::new();
    let mut offsets = vec![0_usize];
    for r in rows {
        scratch.extend_from_slice(r);
        offsets.push(scratch.len());
    }
    // Spaces, not zeros: a root-level number or atom is terminated by the byte after it,
    // and a NUL is not a valid terminator.
    scratch.resize(scratch.len() + INPUT_PADDING, b' ');

    let mut buffers = Buffers::new(256);
    let mut out = Vec::new();
    for row in 0..rows.len() {
        let row_len = offsets[row + 1] - offsets[row];
        let padded = &mut scratch[offsets[row]..];
        let mut tape = Tape::null();
        let res = unsafe { fill_tape_padded(padded, row_len, &mut buffers, &mut tape) };
        out.push(match res {
            Ok(()) => format!("{:?}", tape.0),
            Err(e) => format!("ERR {e:?}"),
        });
    }
    out
}

/// The reference result: the same document through the copying `fill_tape`.
fn parse_reference(doc: &[u8]) -> String {
    let mut buf = doc.to_vec();
    let mut buffers = Buffers::new(256);
    let mut tape = Tape::null();
    match fill_tape(&mut buf, &mut buffers, &mut tape) {
        Ok(()) => format!("{:?}", tape.0),
        Err(e) => format!("ERR {e:?}"),
    }
}

/// Asserts every document parses identically through both entry points, each as its own
/// single-row scratch.
fn assert_matches_fill_tape(docs: &[&[u8]]) {
    for doc in docs {
        let got = &parse_rows(&[doc])[0];
        assert_eq!(
            got,
            &parse_reference(doc),
            "fill_tape_padded diverged from fill_tape on {:?}",
            String::from_utf8_lossy(doc)
        );
    }
}

#[test]
fn plain_documents() {
    assert_matches_fill_tape(&[
        br#"{"a":"plain","b":"also plain","c":7}"#,
        br#"{"a":1,"b":-2.5,"c":true,"d":false,"e":null}"#,
        br#"[1,"two",{"three":3},[4,[5]]]"#,
        br#""just a string""#,
        br#"42"#,
        br#"{}"#,
        br#"[]"#,
    ]);
}

/// A root-level number or atom runs to the end of the logical input, so the byte that
/// terminates it is the caller's first padding byte.
#[test]
fn root_scalars_terminated_by_padding() {
    assert_matches_fill_tape(&[
        br#"123"#,
        br#"-2374611873366417043"#,
        br#"-1.5e3"#,
        br#"1.7976931348623157e308"#,
        br#"true"#,
        br#"false"#,
        br#"null"#,
        // and one nesting level in, where the terminator is still real input
        br#"[123]"#,
        br#"{"a":true}"#,
    ]);
}

#[test]
fn escapes_everywhere() {
    assert_matches_fill_tape(&[
        br#"{"a":"has\nescape","b":2}"#,
        br#"{"a":"\n","b":"\t\r\b\f\\\/\"","c":"end\\"}"#,
        // embedded JSON: the escape-dense real-world shape
        br#"{"payload":"{\"inner\":{\"k\":\"v\",\"n\":1},\"list\":[\"a\",\"b\"]}","after":9}"#,
        // unicode escapes incl. surrogate pair and multi-byte output
        br#"{"u":"\u00e9\u4e2d\ud83d\ude00","v":"x"}"#,
        "{\"u\":\"\u{e9}\u{4e2d} literal\",\"w\":\"then\\nescaped\"}".as_bytes(),
        // escaped key, value read after an unescaped write
        br#"{"k\ney":123,"z":"tail"}"#,
        // atoms and numbers after escape-heavy strings
        br#"{"s":"a\\b\\c\\d","t":true,"n":null,"f":-1.25e3}"#,
        // multi-byte literal content
        "{\"k\":\"caf\u{e9} \u{4e2d}\u{6587} \u{1f642}\",\"n\":1}".as_bytes(),
    ]);
}

/// One escape at every position of strings sized around the 16/32/64-byte lanes, so
/// in-place write-back and lane loads get exercised at each relative offset.
#[test]
fn escape_positions_cross_simd_boundaries() {
    let sizes: &[usize] = if cfg!(miri) {
        &[31, 32, 33]
    } else {
        &[
            1, 2, 15, 16, 17, 30, 31, 32, 33, 34, 47, 48, 49, 63, 64, 65, 66, 96,
        ]
    };
    for &size in sizes {
        for pos in 0..size.saturating_sub(1) {
            let mut val = vec![b'a'; size];
            val[pos] = b'\\';
            val[pos + 1] = b'n';
            let mut doc = Vec::from(&br#"{"k":""#[..]);
            doc.extend_from_slice(&val);
            doc.extend_from_slice(br#"","m":7}"#);
            assert_matches_fill_tape(&[&doc]);
        }
    }
}

/// Last row in the scratch: the unescape write-back lands directly against the padding.
#[test]
fn escape_at_end_of_logical_input() {
    assert_matches_fill_tape(&[br#"{"k":"tail\n"}"#, br#"{"k":"tail\\"}"#, br#"["aA"]"#]);
}

#[test]
fn multi_row_scratch_shares_padding() {
    let rows: &[&[u8]] = &[
        br#"{"a":"x\ny","b":1}"#,
        br#"{"a":"plain","b":2}"#,
        "{\"a\":\"\u{e9}\u{e8}\",\"b\":3}".as_bytes(),
        br#"{"a":"end\\"}"#,
        // trailing row whose value runs into the padding
        br#"{"a":42}"#,
    ];
    let got = parse_rows(rows);
    for (doc, got) in rows.iter().zip(&got) {
        assert_eq!(
            got,
            &parse_reference(doc),
            "row {:?} diverged",
            String::from_utf8_lossy(doc)
        );
    }
}

#[test]
fn invalid_documents_match_fill_tape() {
    assert_matches_fill_tape(&[
        br#"{"a":"bad\qescape"}"#,
        br#"{"a":"\ud83d"}"#, // unpaired surrogate
        br#"{"a":tru}"#,
        br#"{"a":1"#,
        br#"{"a" 1}"#,
        br#"{"a":}"#,
        br#"[1,]"#,
        br#""unterminated"#,
        br#""#,
    ]);
}

/// Invalid UTF-8 in the logical input must be rejected, same as `fill_tape`.
#[test]
fn invalid_utf8_matches_fill_tape() {
    assert_matches_fill_tape(&[
        &[b'"', 0xff, 0xfe, b'"'],
        &[b'{', b'"', b'k', b'"', b':', b'"', 0x80, b'"', b'}'],
    ]);
}

/// Deterministic xorshift so the corpus is reproducible (no rand dev-dependency).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn random_string(rng: &mut Rng, out: &mut Vec<u8>) {
    out.push(b'"');
    for _ in 0..rng.below(80) {
        match rng.below(12) {
            0 => out.extend_from_slice(b"\\n"),
            1 => out.extend_from_slice(b"\\\""),
            2 => out.extend_from_slice(b"\\\\"),
            3 => out.extend_from_slice(b"\\u00e9"),
            4 => out.extend_from_slice(b"\\ud83d\\ude00"),
            5 => out.extend_from_slice("\u{4e2d}".as_bytes()),
            _ => {
                let run = 1 + rng.below(37);
                let c = b'a' + (rng.below(26) as u8);
                out.extend(std::iter::repeat_n(c, run));
            }
        }
    }
    out.push(b'"');
}

fn random_value(rng: &mut Rng, depth: usize, out: &mut Vec<u8>) {
    let pick = if depth >= 3 {
        rng.below(4)
    } else {
        rng.below(6)
    };
    match pick {
        0 => random_string(rng, out),
        1 => out.extend_from_slice(rng.next().to_string().as_bytes()),
        2 => {
            out.extend_from_slice(
                format!("-{}.{}e-2", rng.below(1000), rng.below(1000)).as_bytes(),
            );
        }
        3 => out.extend_from_slice([&b"true"[..], b"false", b"null"][rng.below(3)]),
        4 => {
            out.push(b'[');
            for i in 0..rng.below(4) {
                if i > 0 {
                    out.push(b',');
                }
                random_value(rng, depth + 1, out);
            }
            out.push(b']');
        }
        _ => {
            out.push(b'{');
            for i in 0..rng.below(4) {
                if i > 0 {
                    out.push(b',');
                }
                random_string(rng, out);
                out.push(b':');
                random_value(rng, depth + 1, out);
            }
            out.push(b'}');
        }
    }
}

#[test]
fn randomized_equivalence_with_fill_tape() {
    let iterations = if cfg!(miri) { 20 } else { 500 };
    let mut rng = Rng(0x5eed_cafe_f00d_0001);
    for _ in 0..iterations {
        // Several rows per scratch, so most rows are followed by real input.
        let mut rows: Vec<Vec<u8>> = Vec::new();
        for _ in 0..1 + rng.below(4) {
            let mut doc = Vec::new();
            doc.push(b'{');
            for i in 0..1 + rng.below(5) {
                if i > 0 {
                    doc.push(b',');
                }
                random_string(&mut rng, &mut doc);
                doc.push(b':');
                random_value(&mut rng, 1, &mut doc);
            }
            doc.push(b'}');
            rows.push(doc);
        }
        let row_refs: Vec<&[u8]> = rows.iter().map(Vec::as_slice).collect();
        let got = parse_rows(&row_refs);
        for (doc, got) in row_refs.iter().zip(&got) {
            assert_eq!(
                got,
                &parse_reference(doc),
                "diverged on {:?}",
                String::from_utf8_lossy(doc)
            );
        }
    }
}
