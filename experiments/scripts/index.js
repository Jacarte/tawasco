// import WASI
const WASI = require('wasi');
// Load Wasm module from fs
const fs = require('fs');

let wasm = fs.readFileSync('./input.wasm');

wasm = WebAssembly.compile(wasm).then(async ww => {

    // Instantite the Wasm from the buffer
    const wasi = new WASI.WASI({
        version: 'preview1',
        args: [ /* empty for now */ ],
        env: { /* empty for now */ },
        preopens: {
        // '/sandbox': '/dev/',
        },
    });
    
    // Instantiate the Wasm code
    const instance = await WebAssembly.instantiate(ww, {
        ...wasi.getImportObject()
    });

    wasi.start(instance);

});

// Run it?
