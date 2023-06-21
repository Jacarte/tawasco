[![Build wasm-mutate runtime only](https://github.com/Jacarte/tawasco/actions/workflows/ci.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci.yml) [![Build host_single for tracing](https://github.com/Jacarte/tawasco/actions/workflows/ci_host_based.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci_host_based.yml) [![Build host for spectre POC](https://github.com/Jacarte/tawasco/actions/workflows/ci_host_based_sequential.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci_host_based_sequential.yml)
[![Build docker for experimenting](https://github.com/Jacarte/tawasco/actions/workflows/build_docker_images.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/build_docker_images.yml) [![Build stacking tool to stack wasm-mutate mutations](https://github.com/Jacarte/tawasco/actions/workflows/ci_stacking.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci_stacking.yml) [![Build wasmtime](https://github.com/Jacarte/tawasco/actions/workflows/ci_wasmtime.yml/badge.svg)](https://github.com/Jacarte/tawasco/actions/workflows/ci_wasmtime.yml)

# This repo contains cache-timing and side-channel based attacks for WebAssembly


The idea is to have POC for attacks on the two Wasm execution scenarios: in the browser and backend based. We code the attacks in Rust and we compile them to Wasm.

In the following gif we show how the Spectre POC (in the same binary) works on exfiltrating data. This same mechanism can be use to violates the CFI, as shown un [Swivel](https://arxiv.org/abs/2102.12730).

![gif](/docs/video.gif)


## Requirements
- Rust nightly. Install it with `rustup install nightly`. We need nightly beacuase we use some `asm!` experimental features to directly write Wasm code from Rust (see [How to deploy hand made Wasm code in Fastly Compute@Edge.](https://www.jacarte.me/blog/2021/HandMadeWasmDeploInFastly/))


## Repo structure

- [`host_based`](/host_based): Contains the backend based architecture and the POCs for the attacks.
  - [`host`](/host_based/host): The host tool with wasmtime as the engine.
  - [`host_single` (Linux only)](/host_based/host_single): The host tool with wasmtime as the engine. Hooks added to filter non-Wasm traces with TracerPIN. Static and deterministic memory allocation for linear memory and executable memory.
  - [`rustctre`](/host_based/rustctre): The POCs for the attacks. We implement the POCs in Rust and we compile them to Wasm binaries.
    - [`cache_time_predictor.rs`](/host_based/rustctre/src/cache_time_predictor.rs): The cache miss/hit time predictor.
    - [`eviction.rs`](/host_based/rustctre/src/eviction.rs): The cache timing attack. This simple POC [just explicitly evicts](https://github.com/Jacarte/TAWasm/blob/420017590f641682defbf8114ffa881d984e7709/host_based/rustctre/src/eviction.rs#L87) the cache and measures the time to access the memory.
    - [`spectre_wasm.rs`](/host_based/rustctre/src/spectre_wasm.rs): The Spectre V1 attack in the same Wasm binary.
    - [`spectre_wasm_sync_simulated.rs`](/host_based/rustctre/src/spectre_wasm_sync_simulated.rs): The Spectre V1 attack exfiltrating from the host. This assumes that the host contains secret values (TODO double check the assumptions of Swivel).
  - [`Makefile`](/host_based/Makefile): The Makefile to compile the Wasm POCs. The binaries can be compiled directly with `Cargo build --target=wasm32-wasi`, yet we do some processing for the binaries to collect some data.
  - [`wasmtime`](/wasmtime): Our fork of wasmtime. We change the allocation (only in unix like OS) to be deterministic and handled in the embedding host. The linear memory can be handled by using the default wasmtime config. Yet, we also want to control where the executable memory is allocated.
  - [`Tracer`](/Tracer): Our fork of the Tracer project (https://github.com/SideChannelMarvels/Tracer). We add a new callback to pause and unpause recoding of PIN traces while executing Wasmtime.


## Roadmap host based:

- [ ] Speculative/Cache time side-channel in Wasm
 - [x] Cache timing threshold prediction. See `host_based/rustctre/cache_time_predictor.rs`
 - [x] Using wasmtime embedded into as the host engine. See `host_based/host/src/main.rs`
 - [ ] Attack cases:
   - [x] **C1**: Simple cache timing example. See `host_based/rustctre/eviction.rs`
   - [x] **C2**: Simple Spectre V1 in the same Wasm binary. See `host_based/rustctre/spectre_wasm.rs`
   - [ ] **C4**  : Attacker and victim in different Wasm binaries.
- [x] Test if precompiled Wasm files make a difference. 
  - They do, at least in execution time when we autmatically test the accuracy
- [ ] Create automatic benchmark for measuring exfiltration accuracy.
- [ ] Apply wasm-mutate to both, attacker or victim. Measure the impact on the accuracy of the attack.

## Roadmap browser

- [ ] Port contention side channel
  - [ ] Implement instruction port contention predictor.
    - [ ] Support high accurate timer (probably using Firefox version 90 for the POC).
    - [ ] Crate Wasm binary to execute in the browser.
    - [ ] Create native binary that makes port contention.
    - [ ] Measures port contention in the browser.
  - [ ] Create automatic benchmark for measuring predictor accuracy.
  - [ ] Apply wasm-mutate to the port predictor and the listener. Measure the impact on the accuracy of the predictor.


## Roadmap mixed

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
- [x] Apply wasm-mutate to victim. Measure the impact on the accuracy of the attack.
