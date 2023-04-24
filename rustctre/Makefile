native:
	cargo build --release --no-default-features

# Meant for wasmtime
wasm:
	 cargo +nightly build --release --target=wasm32-wasi --no-default-features --features tracing
	 cp target/wasm32-wasi/release/eviction.wasm out/eviction.tracing.wasm
	 cp target/wasm32-wasi/release/sp.wasm out/sp.tracing.wasm
	 cargo +nightly build --release --target=wasm32-wasi --no-default-features
	 cp target/wasm32-wasi/release/eviction.wasm out/eviction.wasm
	 cp target/wasm32-wasi/release/sp.wasm out/sp.wasm


	# cargo +nightly build --release --target=wasm32-wasi --no-default-features
	${WASM2WAT} target/wasm32-wasi/release/eviction.wasm -o target/wasm32-wasi/release/eviction.wat
	# for debugging, create the wat format

# TODO make one for V8
