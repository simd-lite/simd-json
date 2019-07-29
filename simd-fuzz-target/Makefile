run: build
	RUSTFLAGS='-C codegen-units=1 -C target-cpu=native' cargo +nightly afl fuzz -i in -o out target/debug/simd-fuzz-target
build: 
	RUSTFLAGS='-C codegen-units=1' cargo +nightly afl build
run-sse: build-sse
	RUSTFLAGS='-C codegen-units=1 -C target-cpu=native -C target-feature=-avx2' cargo +nightly afl fuzz -i in -o out target/debug/simd-fuzz-target
build-sse:
	RUSTFLAGS='-C codegen-units=1 -C target-cpu=native -C target-feature=-avx2' cargo +nightly afl build

copy:
	for from in `ls out/crashes/id*`; do to=`echo $$from | sed -e 's;out/crashes/id:;crash;' -e 's;,.*;.json;'`; cp $$from ../simdjson-rs/data/crash/$$to; done
