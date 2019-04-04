# simdjson-rs
Rust port of extremely fast [simdjson](https://github.com/lemire/simdjson) JSON parser with [serde](serde.rs) compatibility.
 

# readme (for real!)

For taking advantage of simdjson your system needs to be SIMD compatible. This means to compile with native cpu support and the given features. Look at [The cargo config in this repsoitory](.carg/config) to get an example.

# License

simdjson-rs itself is licensed under either of

* Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

However it ports a lot of code from [simdjson](https://github.com/lemire/simdjson) so their work and copyright on that should be respected along side.

The [serde](serde.rs) integration is based on their example and serde-json so again, their copyright should as well be respected.