[![Build wasm-mutate runtime only](https://github.com/Jacarte/tawasco/actions/workflows/ci.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci.yml) [![Build host_single for tracing](https://github.com/Jacarte/tawasco/actions/workflows/ci_host_based.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci_host_based.yml) [![Build host for spectre POC](https://github.com/Jacarte/tawasco/actions/workflows/ci_host_based_sequential.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci_host_based_sequential.yml)
[![Build docker for experimenting](https://github.com/Jacarte/tawasco/actions/workflows/build_docker_images.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/build_docker_images.yml) [![Build stacking tool to stack wasm-mutate mutations](https://github.com/Jacarte/tawasco/actions/workflows/ci_stacking.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci_stacking.yml) [![Build wasmtime](https://github.com/Jacarte/tawasco/actions/workflows/ci_wasmtime.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci_wasmtime.yml)

# This repo contains cache-timing and side-channel based attacks for WebAssembly, replicated from Swivel and safeside


The idea is to have POC for attacks on Wasm execution scenarios. We add a diversification evaluation with wasm-mutate in the safeside as well.

## Requirements
- Rust nightly. Install it with `rustup install nightly`. We need nightly beacuase we use some `asm!` experimental features to directly write Wasm code from Rust (see [How to deploy hand made Wasm code in Fastly Compute@Edge.](https://www.jacarte.me/blog/2021/HandMadeWasmDeploInFastly/))


## Roadmap browser

- [ ] Port contention side channel
  - [ ] Implement instruction port contention predictor.
    - [ ] Support high accurate timer (probably using Firefox version 90 for the POC).
    - [ ] Crate Wasm binary to execute in the browser.
    - [ ] Create native binary that makes port contention.
    - [ ] Measures port contention in the browser.
  - [ ] Create automatic benchmark for measuring predictor accuracy.
  - [ ] Apply wasm-mutate to the port predictor and the listener. Measure the impact on the accuracy of the predictor.


## Roadmap for whitebox crypto challenges

Questions:
- Does it make sense as a use case to whitebox a Wasm ? Yes, distributing a signed .wasm

To reproduce this attacks and defenses. We propose to use a separated machine. For security and better measurements collection.

- [x] White box cryptography [challenges](https://github.com/SideChannelMarvels/Deadpool)
  - [x] Compile C to Wasm
    - [x] CHES2016
  - [x] Perform attack
    - [x] Host based with wasmtime
      - [x] CHES2016
        - [x] DCA. Running wasmtime precompiled wasm `host_single/release/host_single wb_challenge.wasm`
        - Daredevil is able to exfiltrate the full key in around 5000 traces.
        - Note: disable ASLR for better performance.
        - The attack works only with PIN. It was easier for plotting and filtrating non-Wasm traces.
    - [x] Host based with wasmtime
- [x] Create automatic benchmark for measuring exfiltration accuracy
- [x] Apply wasm-mutate to victim. Measure the impact on the accuracy of the attack. Sadly :( wasm-mutate does not help in this case.
