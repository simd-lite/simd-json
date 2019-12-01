# SIMD Json for Rust &emsp; [![Sanity]][simd-json.rs] [![Build Status]][simd-json.rs] [![Build Status ARM]][drone.io] [![Latest Version]][crates.io] [![Code Coverage]][codecov.io]

[Build Status ARM]: https://cloud.drone.io/api/badges/simd-lite/simdjson-rs/status.svg
[drone.io]: https://cloud.drone.io/simd-lite/simdjson-rs
[Build Status]: https://github.com/simd-lite/simdjson-rs/workflows/Rust%20%28tests%29/badge.svg
[Sanity]: https://github.com/simd-lite/simdjson-rs/workflows/Rust%20%28sanity%20checks%29/badge.svg
[simd-json.rs]: https://simd-json.rs
[Latest Version]: https://img.shields.io/crates/v/simd-json.svg
[crates.io]: https://crates.io/crates/simd-json
[Code Coverage]: https://codecov.io/gh/simd-lite/simdjson-rs/badge.svg
[codecov.io]: https://codecov.io/gh/simd-lite/simdjson-rs

**Rust port of extremely fast [simdjson](https://github.com/lemire/simdjson) JSON parser with [serde][1] compatibility.**

---

## readme (for real!)

### simdjson version

**Currently tracking version 0.2.x of simdjson upstream (work in progress, feedback welcome!).**

### CPU target

To be able to take advantage of simdjson your system needs to be SIMD compatible. This means to compile with native cpu support and the given features. Look at [The cargo config in this repository](.cargo/config) to get an example.

simd-json.rs supports AVX2, SSE4.2 and NEON.

### jemalloc

If you are writing performance centric code, make sure to use jemalloc and not the system allocator (which has now become default in rust), it gives a very noticeable boost in performance.

## serde

simdjson-rs is compatible with serde and serde-json. The Value types provided implement serializers and deserializers. In addition to that simdjson-rs implements the Deserializer for the parser so it can deserialize anything that implements the serde Deserialize trait.

That said serde is contained in the `serde_impl` feature which is part of the default feature set, but it can be disabled.

### known-key

The `known-key` feature changes hasher for the objects, from `ahash` to `fxhash`, ahash is faster at hashing and provides protection against DOS attacks by forcing multiple keys into a single hashing bucket. `fxhash`  on the other hand allows for repeatable hashing results, that allows memorizing hashes for well know keys and saving time on lookups. In workloads that are heavy at accessing some well known keys this can be a performance advantage.

### serializing

simdjson-rs is not capable of serializing JSON data as there would be very little gain by re-implementing it. For serialization, we recommend serde-json.


### unsafe

simdjson-rs uses **a lot** of unsafe code first of all since all SIMD-intrinsics are inherently unsafe and also to work around some bottlenecks introduced by rust's safe nature. This requires extra scrutiny and thus needs to be diligently tested according to these 5 steps:

* Poor old unit tests - to test general 'obvious' cases and edge cases as well as regressions
* Property based testing on valid json - tests if valid but random generated json parses the same in simd-json and in serde-json (floats here are excluded since slighty different parsing algorihtms lead to slighty different results)
* Property based testing on random 'human readable' data - make sure that randomly generated sequences of printable characters don't panic or crash the parser (they might and often error so - they are not valid json!)
* Property based testing on random byte sequences - make sure that no random set of bytes will crash the parser
* Fuzzing (using afl) - fuzz based on upstream simd pass/fail cases

This certainly doesn't ensure complete safety but it does go a long way.

## Other interesting things

There are also bindings for simdjson available [here](https://github.com/SunDoge/simdjson-rust)

## License

simdjson-rs itself is licensed under either of

* Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

However it ports a lot of code from [simdjson](https://github.com/lemire/simdjson) so their work and copyright on that should be respected along side.

The [serde][1] integration is based on their example and serde-json so again, their copyright should as well be respected.

[1]: https://serde.rs
