# SIMD JSON for Rust &emsp; [![Build Status]][simd-json.rs] [![Build Status ARM]][drone.io] [![Quality]][simd-json.rs]  [![Latest Version]][crates.io] [![Code Coverage]][coveralls]

[Build Status ARM]: https://cloud.drone.io/api/badges/simd-lite/simd-json/status.svg
[drone.io]: https://cloud.drone.io/simd-lite/simd-json
[Build Status]: https://github.com/simd-lite/simd-json/workflows/Tests/badge.svg
[Quality]: https://github.com/simd-lite/simd-json/workflows/Quality/badge.svg
[simd-json.rs]: https://simd-json.rs
[Latest Version]: https://img.shields.io/crates/v/simd-json.svg
[crates.io]: https://crates.io/crates/simd-json
[Code Coverage]: https://coveralls.io/repos/github/simd-lite/simd-json/badge.svg?branch=main
[coveralls]: https://coveralls.io/github/simd-lite/simd-json?branch=main

**Rust port of extremely fast [simdjson](https://github.com/lemire/simdjson) JSON parser with [serde][1] compatibility.**

---

## readme (for real!)

### simdjson version

**Currently tracking version 0.2.x of simdjson upstream (work in progress, feedback is welcome!).**

### CPU target

To be able to take advantage of `simd-json` your system needs to be SIMD-capable. On `x86`, it will select the best SIMD feature set (`avx2` or `sse4.2`) during runtime. If `simd-json` is compiled with SIMD support, it will disable runtime detection.

`simd-json` supports AVX2, SSE4.2, NEON, and simd128 (wasm) natively. It also includes an unoptimized fallback implementation using native rust for other platforms; however, this is a last resort measure and nothing we'd recommend relying on.

### Performance characteristics

- CPU native cpu compilation results in the best performance.
- CPU detection for AVX and SSE4.2 is the second fastest (on x86_* only).
- Portable std::simd is the next fast implementation when compiled with a native CPU target.
- std::simd or the rust native implementation is the least performant.

### allocator

For best performance, we highly suggest using [mimalloc](https://crates.io/crates/mimalloc) or [jemalloc](https://crates.io/crates/jemalloc) instead of the system allocator used by default. Another recent allocator that works well (but we have yet to test it in production) is [snmalloc](https://github.com/microsoft/snmalloc).

### `runtime-detection`

This feature allows selecting the optimal algorithm based on available features during runtime; it has no effect on non-x86 or x86_64 platforms. When neither `AVX2` nor `SSE4.2` is supported, it will fall back to a native rust implementation.

Note that an application compiled with `runtime-detection` will not run as fast as an application compiled for a specific CPU. The reason is that rust can't optimise as far as the instruction set when it uses the generic instruction set, and non-simd parts of the code won't be optimised for the given instruction set either.

### `portable`

**Currently disabled**

An implementation of the algorithm using `std::simd` and up to 512 byte wide registers, currently disabled due to dependencies and is highly experimental.

### `serde_impl`

`simd-json` is compatible with serde and `serde-json`. The Value types provided implement serializers and deserializers. In addition to that `simd-json` implements the `Deserializer` trait for the parser so it can deserialize anything that implements the serde `Deserialize` trait. Note, that serde provides both a `Deserializer` and a `Deserialize` trait.

That said the serde support is contained in the `serde_impl` feature which is part of the default feature set of `simd-json`, but it can be disabled.

### `known-key`

The `known-key` feature changes the hash mechanism for the DOM representation of the underlying JSON object, from `ahash` to `fxhash`. The `ahash` hasher is faster at hashing and provides protection against DOS attacks by forcing multiple keys into a single hashing bucket. The `fxhash` hasher, on the other hand allows for repeatable hashing results, which in turn allows memoizing hashes for well known keys and saving time on lookups. In workloads that are heavy on accessing some well-known keys, this can be a performance advantage.

The `known-key` feature is optional and disabled by default and should be explicitly configured.

### `value-no-dup-keys`


**This flag has no effect on simd-json itself but purely affects the `Value` structs.**

The `value-no-dup-keys` feature flag toggles stricter behavior for objects when deserializing into a `Value`. When enabled, the Value deserializer will remove duplicate keys in a JSON object and only keep the last one. If not set duplicate keys are considered undefined behavior and Value will not make guarantees on it's behavior.

### `big-int-as-float`

The `big-int-as-float` feature flag treats very large integers that won't fit into u64 as f64 floats. This prevents parsing errors if the JSON you are parsing contains very large numbers. Keep in mind that f64 loses some precision when representing very large numbers.

## safety

`simd-json` uses **a lot** of unsafe code.

There are a few reasons for this:

* SIMD intrinsics are inherently unsafe. These uses of unsafe are inescapable in a library such as `simd-json`.
* We work around some performance bottlenecks imposed by safe rust. These are avoidable, but at a performance cost. This is a more considered path in `simd-json`.


`simd-json` goes through extra scrutiny for unsafe code. These steps are:

* Unit tests - to test 'the obvious' cases, edge cases, and regression cases
* Structural constructive property based testing - We generate random valid JSON objects to exercise the full `simd-json` codebase stochastically. Floats are currently excluded since slightly different parsing algorithms lead to slightly different results here. In short "is simd-json correct".
* Data-oriented property-based testing of string-like data - to assert that sequences of legal printable characters don't panic or crash the parser (they might and often error so - they are not valid JSON!)
* Destructive Property based testing - make sure that no illegal byte sequences crash the parser in any way
* Fuzzing - fuzz based on upstream & jsonorg simd pass/fail cases
* Miri testing for UB

This doesn't ensure complete safety nor is at a bulletproof guarantee, but it does go a long way
to assert that the library is of high production quality and fit for purpose for practical industrial applications.

## Other interesting things

There are also bindings for upstream `simdjson` available [here](https://github.com/SunDoge/simdjson-rust)

## License

simd-json itself is licensed under either of

* Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

However it ports a lot of code from [simdjson](https://github.com/lemire/simdjson) so their work and copyright on that should be respected along side.

The [serde][1] integration is based on their example and `serde-json` so again, their copyright should as well be respected.

[1]: https://serde.rs

### All Thanks To Our Contributors:
<a href="https://github.com/simd-lite/simd-json/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=simd-lite/simd-json" />
</a>
