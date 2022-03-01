
baseline:
	cargo +nightly run --example perf --features perf --release -- -b
perf:
	cargo +nightly run --example perf --features perf --release
clippy:
	touch src/lib.rs
	cargo clippy
wasmtest:
	cargo clean --target-dir target
	cargo build --tests --target wasm32-wasi --target-dir target
	wasmtime run --wasm-features=simd target/wasm32-wasi/debug/deps/simd_json*.wasm
	wasmtime run --dir=. --wasm-features=simd target/wasm32-wasi/debug/deps/jsonchecker*.wasm
