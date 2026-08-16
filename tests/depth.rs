// Regression tests for https://github.com/simd-lite/simd-json/issues/473
//
// Building the tape is iterative and copes with any nesting depth, but every
// consumer of the tape - the `Value` builders, the JSON writers and the serde
// deserializers - walks it recursively. Without a bound, deeply nested input
// exhausts the stack and aborts the process.

use simd_json::{Buffers, DEFAULT_MAX_DEPTH, ErrorType};

/// `n` nested containers, e.g. `nest("[", "]", 3)` -> `[[[]]]`.
fn nest(open: &str, close: &str, n: usize) -> Vec<u8> {
    let mut s = String::with_capacity(n * (open.len() + close.len()));
    for _ in 0..n {
        s.push_str(open);
    }
    for _ in 0..n {
        s.push_str(close);
    }
    s.into_bytes()
}

/// `n` nested objects, e.g. `nest_obj(2)` -> `{"a":{"a":null}}`.
fn nest_obj(n: usize) -> Vec<u8> {
    let mut s = String::new();
    for _ in 0..n {
        s.push_str(r#"{"a":"#);
    }
    s.push_str("null");
    for _ in 0..n {
        s.push('}');
    }
    s.into_bytes()
}

fn is_depth_error(e: &simd_json::Error) -> bool {
    *e.error() == ErrorType::DepthLimitExceeded
}

#[test]
fn arrays_at_the_limit_parse() {
    let mut ok = nest("[", "]", DEFAULT_MAX_DEPTH);
    assert!(simd_json::to_owned_value(&mut ok).is_ok());
}

#[test]
fn arrays_past_the_limit_are_rejected() {
    let mut bad = nest("[", "]", DEFAULT_MAX_DEPTH + 1);
    let e = simd_json::to_owned_value(&mut bad).expect_err("should have hit the depth limit");
    assert!(is_depth_error(&e), "unexpected error: {e:?}");
}

#[test]
fn objects_at_the_limit_parse() {
    let mut ok = nest_obj(DEFAULT_MAX_DEPTH);
    assert!(simd_json::to_owned_value(&mut ok).is_ok());
}

#[test]
fn objects_past_the_limit_are_rejected() {
    let mut bad = nest_obj(DEFAULT_MAX_DEPTH + 1);
    let e = simd_json::to_owned_value(&mut bad).expect_err("should have hit the depth limit");
    assert!(is_depth_error(&e), "unexpected error: {e:?}");
}

/// Alternating containers exercise all four in-loop check sites (`ObjectKey`
/// and `MainArraySwitch`, each with `{` and `[`), not just the array ones.
#[test]
fn alternating_containers_are_counted() {
    // Each repetition adds two levels: one array, one object.
    let reps = DEFAULT_MAX_DEPTH / 2 + 1;
    let mut s = String::new();
    for _ in 0..reps {
        s.push_str(r#"[{"a":"#);
    }
    s.push_str("null");
    for _ in 0..reps {
        s.push_str("}]");
    }
    let mut bad = s.into_bytes();
    let e = simd_json::to_owned_value(&mut bad).expect_err("should have hit the depth limit");
    assert!(is_depth_error(&e), "unexpected error: {e:?}");
}

/// The limit lives in stage 2, so every entry point that builds a tape from
/// bytes inherits it.
#[test]
fn every_entry_point_rejects() {
    let deep = nest("[", "]", DEFAULT_MAX_DEPTH + 1);

    let mut d = deep.clone();
    assert!(is_depth_error(
        &simd_json::to_owned_value(&mut d).unwrap_err()
    ));

    let mut d = deep.clone();
    assert!(is_depth_error(
        &simd_json::to_borrowed_value(&mut d).unwrap_err()
    ));

    let mut d = deep.clone();
    assert!(is_depth_error(&simd_json::to_tape(&mut d).unwrap_err()));

    let mut d = deep.clone();
    assert!(is_depth_error(
        &simd_json::Deserializer::from_slice(&mut d).unwrap_err()
    ));

    #[cfg(feature = "serde_impl")]
    {
        let mut d = deep.clone();
        assert!(is_depth_error(
            &simd_json::serde::from_slice::<simd_json::OwnedValue>(&mut d).unwrap_err()
        ));

        // a derive-shaped target, not just the Value visitor
        let mut d = deep.clone();
        assert!(is_depth_error(
            &simd_json::serde::from_slice::<Vec<Vec<simd_json::OwnedValue>>>(&mut d).unwrap_err()
        ));
    }
}

/// Unterminated deep input must error rather than hang or abort.
#[test]
fn unterminated_deep_input_errors() {
    let mut bad = "[".repeat(DEFAULT_MAX_DEPTH + 1).into_bytes();
    assert!(simd_json::to_owned_value(&mut bad).is_err());
}

#[test]
fn lowering_the_limit_works() {
    let mut buffers = Buffers::with_max_depth(64, 4);

    let mut ok = nest("[", "]", 4);
    assert!(simd_json::to_owned_value_with_buffers(&mut ok, &mut buffers).is_ok());

    let mut bad = nest("[", "]", 5);
    let e = simd_json::to_owned_value_with_buffers(&mut bad, &mut buffers)
        .expect_err("should have hit the depth limit");
    assert!(is_depth_error(&e), "unexpected error: {e:?}");
}

/// Raising the limit works, and proves tape building really is depth
/// independent. Deliberately does *not* materialize a `Value` from the result -
/// that is the recursive step this limit exists to bound.
#[test]
fn raising_the_limit_allows_a_deep_tape() {
    let mut buffers = Buffers::with_max_depth(64, usize::MAX);

    let mut very_deep = nest("[", "]", 100_000);
    let tape = simd_json::to_tape_with_buffers(&mut very_deep, &mut buffers)
        .expect("tape building is iterative, depth should not matter");
    assert_eq!(tape.0.len(), 100_000);
}

/// The actual security assertion: a document at the limit must fit in the stack
/// a spawned thread gets by default, which is the smallest stack real code
/// routinely runs on. This is the guardrail that fires if anyone raises
/// `DEFAULT_MAX_DEPTH` past what the recursive consumers can afford.
///
/// Note this can only assert the *bounded* case survives - a Rust stack
/// overflow aborts the process without unwinding, so the unbounded case is not
/// something a test can catch.
#[test]
fn limit_fits_in_a_default_thread_stack() {
    // 2MiB, std's default for a spawned thread. Measured need at 1024 levels is
    // ~403KiB release / ~727KiB debug for build-then-write, so this has roughly
    // 3-5x headroom: loose enough not to be flaky across platforms, tight
    // enough that raising DEFAULT_MAX_DEPTH fourfold trips it.
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut d = nest("[", "]", DEFAULT_MAX_DEPTH);
            let v = simd_json::to_owned_value(&mut d).expect("owned value at the limit");
            // the writers recurse too
            let _ = v.to_string();
            drop(v); // drop glue recurses as well

            let mut d = nest("[", "]", DEFAULT_MAX_DEPTH);
            let v = simd_json::to_borrowed_value(&mut d).expect("borrowed value at the limit");
            let _ = v.to_string();
        })
        .expect("spawn")
        .join()
        .expect("the parser overflowed a 512KiB stack at DEFAULT_MAX_DEPTH");
}

/// Permanent documentation of #473. Aborts the process by design, so it never
/// runs in CI.
#[test]
#[ignore = "intentionally overflows the stack; reproduces #473"]
fn issue_473_repro() {
    let mut buffers = Buffers::with_max_depth(64, usize::MAX);
    let mut very_deep = nest("[", "]", 1_000_000);
    let _ = simd_json::to_owned_value_with_buffers(&mut very_deep, &mut buffers);
}
