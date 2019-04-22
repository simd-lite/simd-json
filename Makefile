
baseline:
	cargo +nightly run --example perf --features perf --release -- -b
perf:
	cargo +nightly run --example perf --features perf --release
