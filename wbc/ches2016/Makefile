all: challenge.c chow_aes3_encrypt_wb.c
	bash download_wasi_sdk.sh
	wasi-sdk/bin/clang -O3 -o wb_challenge.wasm challenge.c chow_aes3_encrypt_wb.c

test:
	python2 trace_it.py

	# ${WASMTIME} run wb_challenge.wasm
