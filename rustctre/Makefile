native:
	cargo build --release --no-default-features

# Meant for wasmtime
wasm:
	 cargo +nightly build --release --target=wasm32-wasi --no-default-features --features tracing
	# cargo +nightly build --release --target=wasm32-wasi --no-default-features
	${WASM2WAT} target/wasm32-wasi/release/eviction.wasm -o target/wasm32-wasi/release/eviction.wat
	# for debugging, create the wat format

# TODO make one for V8
