# SIMD Json for Rust &emsp; [![Build Status]][circleci.com] [![Windows Build Status]][appveyor.com] [![Latest Version]][crates.io]

[Build Status]: https://circleci.com/gh/Licenser/simdjson-rs/tree/master.svg?style=svg
[circleci.com]: https://circleci.com/gh/Licenser/simdjson-rs/tree/master
[Windows Build Status]: https://ci.appveyor.com/api/projects/status/0kf0v6hj5v2gite9?svg=true
[appveyor.com]: https://ci.appveyor.com/project/Licenser/simdjson-rs
[Latest Version]: https://img.shields.io/crates/v/simd-json.svg
[crates.io]: https://crates.io/crates/simd-json

**Rust port of extremely fast [simdjson](https://github.com/lemire/simdjson) JSON parser with [serde](serde.rs) compatibility.**

---

## readme (for real!)

### CPU target

For taking advantage of simdjson your system needs to be SIMD compatible. This means to compile with native cpu support and the given features. Look at [The cargo config in this repository](.cargo/config) to get an example.

### jemalloc

If you are writing performance centric code, make sure to use jemalloc and not the system allocator (that has now become default in rust), it gives an very noticable boost in performance.

## serde

simd-json.rs is compatible with serde and serde-json. The Value types provided implement serializers and deserialisers. In addition to that simd-json.rs implements the Deserializer for the parser so it can deserialize anything that implements the serde Deserialize trait.

That said serde is contained in the `serde_impl` feature which is part of the default feature set, but it can be disabled.

### serializing

sims-json.rs does not provide capabiltiy to serialize JSON data, serde-json does a good job at this and there would be very little won by re-implementing it.

### unsafe

simd-json.rs uses **a lot** of unsafe code first of all since all SIMD-intrinsics are inherently unsafe and also to work around some bottlenecks safe rust code. So this requires extra scrutiny and do to this diligently testing takes 5 shapes:

* Poor old unit tests - to test general 'obvious' cases and edge cases as well as regressions
* Property based testing on valid json - tests if valid but random generated json parses the same in simd-json and in serde-json (floats here are excloded since slighty different parsing algorihtms lead to slighty different results)
* Property based testing on random 'human readable' data - make sure that randomly generated sequences of pritnable characters don't panic or crash the parser (they might and often error so - they are not valid json!)
* Property based testing on random byte sequences - make sure that no random set of bytes will crash the parser
* Fuzzing (using afl) - fuzz based on upstream simd pass/fail cases

This sure doesn't ensure complete safetly but it does go a long way.

## Other interesting things

There are also bindings for simdjson available [here](https://github.com/SunDoge/simdjson-rust)

## License

simdjson-rs itself is licensed under either of

* Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

However it ports a lot of code from [simdjson](https://github.com/lemire/simdjson) so their work and copyright on that should be respected along side.

The [serde](serde.rs) integration is based on their example and serde-json so again, their copyright should as well be respected.
