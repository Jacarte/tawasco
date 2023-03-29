all:
	gcc v1.c -O3 -o  v1
	gcc cache.c -O3 -o cache
	gcc stride.c -o stride

w:
	clang v1.c -O3 --target=wasm32-wasi -o v1.wasm
