# SIMD Json for Rust &emsp; [![Build Status]][simd-json.rs] [![Build Status ARM]][drone.io] [![Quality]][simd-json.rs]  [![Latest Version]][crates.io] [![Code Coverage]][coveralls]

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

**Currently tracking version 0.2.x of simdjson upstream (work in progress, feedback welcome!).**

### CPU target

To be able to take advantage of `simd-json` your system needs to be SIMD capable. This means that it needs to compile with native cpu support and the given features. This also requires that projects using `simd-json` also need to be configured with native cpu support. Look at [The cargo config in this repository](.cargo/config) to get an example of how to configure this in your project.

`simd-json` supports AVX2, SSE4.2 and NEON.

Unless the `allow-non-simd` feature is passed to your `simd-json` dependency in your `Cargo.toml` `simd-json` will fail to compile, this is to prevent unexpected slowness in fallback mode that can be hard to understand and hard to debug.

### allocator

For best performance we highly suggest using [mimalloc](https://crates.io/crates/mimalloc) or [jemalloc](https://crates.io/crates/jemalloc) instead of the system allocator used by default. Another recent allocator that works well ( but we have yet to test in production a setting ) is [snmalloc](https://github.com/microsoft/snmalloc).

## serde

`simd-json` is compatible with serde and `serde-json`. The Value types provided implement serializers and deserializers. In addition to that `simd-json` implements the `Deserializer` trait for the parser so it can deserialize anything that implements the serde `Deserialize` trait. Note, that serde provides both a `Deserializer` and a `Deserialize` trait.

That said the serde support is contained in the `serde_impl` feature which is part of the default feature set of `simd-json`, but it can be disabled.

### known-key

The `known-key` feature changes the hash mechanism for the DOM representation of the underlying JSON object, from `ahash` to `fxhash`. The `ahash` hasher is faster at hashing and provides protection against DOS attacks by forcing multiple keys into a single hashing bucket. The `fxhash` hasher on the other hand allows for repeatable hashing results, which in turn allows memoizing hashes for well known keys and saving time on lookups. In workloads that are heavy at accessing some well known keys this can be a performance advantage.

The `known-key` feature is optional and disabled by default and should be explicitly configured.

### serializing

`simd-json` is not capable of serializing JSON data as there would be very little gain in re-implementing it. For serialization, we typically rely on `serde-json`.

For DOM values we provide convience methods for serialization.

For struct values we defer to external serde-compatible serialization mechanisms.

### unsafe

`simd-json` uses **a lot** of unsafe code.

There are a few reasons for this:

* SIMD intrinsics are inherently unsafe. These uses of unsafe are inescapable in a library such as `simd-json`.
* We work around some performance bottlenecks imposed by safe rust. These are avoidable, but at a cost to performance. This is a more considered path in `simd-json`.


`simd-json` goes through extra scrutiny for unsafe code. These steps are:

* Unit tests - to test 'the obvious' cases, edge cases, and regression cases
* Structural constructive property based testing - We generate random valid JSON objects to exercise the full `simd-json` codebase stochastically. Floats are currently excluded since slighty different parsing algorihtms lead to slighty different results here. In short "is simd-json correct".
* Data-oriented property based testing of string-like data - to assert that sequences of legal printable characters don't panic or crash the parser (they might and often error so - they are not valid json!)
* Destructive Property based testing - make sure that no illegal byte sequences crash the parser in any way
* Fuzzing (using American Fuzzy Lop - afl) - fuzz based on upstream simd pass/fail cases

This doesn't ensure complete safety nor is at a bullet proof guarantee, but it does go a long way
to asserting that the library is production quality and fit for purpose for practical industrial applications.

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
