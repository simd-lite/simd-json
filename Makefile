
baseline:
	cargo +nightly run --example perf --features perf --release -- -b
perf:
	cargo +nightly run --example perf --features perf --release
clippy:
	touch src/lib.rs
	cargo clippy