all:
	wasi-sdk/bin/clang -O3 -o mem_test.wasm f1.c
	../../../wasmtime/target/release/wasmtime compile mem_test.wasm