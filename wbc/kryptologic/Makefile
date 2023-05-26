all: DemoKey_table_encrypt.c
	bash download_wasi_sdk.sh
	wasi-sdk/bin/clang -o kryptologic.wasm DemoKey_table_encrypt.c
	../../../wasmtime/target/release/wasmtime compile kryptologic.wasm

test:
	python2 trace_it.py

	# ${WASMTIME} run wb_challenge.wasm
