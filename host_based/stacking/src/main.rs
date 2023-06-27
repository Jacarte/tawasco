#![feature(internal_output_capture)]
use anyhow::Context;
use clap::Parser;
use core::sync::atomic::Ordering::{Relaxed, SeqCst};
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use std::{panic, process};
use wasm_mutate::WasmMutate;
use wasmtime::Val;

/// # Stacking for wasm-mutate.
///
/// ## Example
/// `stacking -s 1 -c 10 i.wasm -o o.wasm`
///
#[derive(Parser)]
struct Options {
    /// The input folder that contains the Wasm binaries.
    input: PathBuf,
    /// The seed of the random mutation, 0 by default
    #[arg(short = 's', long = "seed")]
    seed: u64,
    /// Number of stacked mutations
    #[arg(short = 'c', long = "count")]
    count: usize,
    /// Save every X steps
    #[arg(short = 'v', long = "save", default_value = "10000")]
    step: usize,
    /// Cache folder name. The cache is used to void repeated transformations that are not interesting.
    #[arg(long = "cache-folder", default_value = "cache")]
    cache_folder: String,
    /// Erase cache on start
    #[arg(long = "remove-cache", default_value = "false")]
    remove_cache: bool,

    /// Do IO equivalence checking. The original and the variant will be executed and their result compared.
    #[arg(long = "check-io", default_value = "false")]
    check_io: bool,

    /// IO check arguments. It will be for the original and the variant comparison if check_io is true.
    #[arg(long = "check-args")]
    check_args: Vec<String>,

    /// Execution fuel. Somehow like a timeout
    #[arg(long = "fuel", default_value = "0")]
    fuel: u64,

    /// If true, a random parent will be selected to create the new variant out of the db. The -v parameter will be ignored.
    #[arg(long = "chaos-mode", default_value = "false")]
    chaos_mode: bool,



    /// If true, checks consistency between original and variant memories
    #[arg(long = "check-mem", default_value = "false")]
    check_mem: bool,


    /// Take X variants from parent only. Only available if chaos mode is true
    #[arg(long = "variants-per-parent", default_value = "10")]
    variants_per_parent: usize,

    /// Uses wasm-mutate preserving semantics
    #[arg(long = "no-preserve-semantics", default_value = "false", action)]
    no_preserve_semantics: bool,


    /// Saves the default compiled Wasm binary
    #[arg(long = "save-compiling", default_value = "false", action)]
    save_compiling: bool,

    /// The output Wasm binary.
    output: PathBuf,

    /// Saves the execution stderr and stdout
    #[arg(long = "save-io", default_value = "false", action)]
    save_io: bool,
}

fn swap(current: &mut Vec<u8>, new_interesting: Vec<u8>) {
    *current = new_interesting;
}

struct Stacking {
    current: (Vec<u8>, usize),
    original: Vec<u8>,
    check_args: Vec<String>,
    original_state: Option<eval::ExecutionResult>,
    index: usize,
    fuel: u64,
    count: usize,
    rnd: SmallRng,
    check_mem: bool,
    // The hashes will prevent regression and non performed transformations
    hashes: sled::Db,
    chaos_mode: bool,
    variants_per_parent: usize,
    no_preserve_semantics: bool,
    save_compiling: bool,
}

impl Stacking {
    pub fn new(
        current: Vec<u8>,
        count: usize,
        seed: u64,
        remove_cache: bool,
        cache_dir: String,
        check_args: Vec<String>,
        check_io: bool,
        fuel: u64,
        chaos_mode: bool,
        check_mem: bool,
        variants_per_parent: usize,
        save_compiling: bool,
        no_preserve_semantics: bool
    ) -> Self {
        // Remove db if exist
        if remove_cache {
            std::fs::remove_dir_all(&cache_dir.clone());
        }

        let config = sled::Config::default()
            .path(cache_dir.clone().to_owned())
            .cache_capacity(/* 4Gb */ 4 * 1024 * 1024 * 1024);

        let original_state = if check_io {
            match eval::execute_single(&current, check_args.clone(), fuel) {
                Some(it) => {
                    eprintln!("Original time {}ns", it.6.as_nanos());
                    Some(it)
                }
                None => {
                    eprintln!("Could not execute the original");
                    process::exit(1);
                }
            }
        } else {
            None
        };

        Self {
            original: current.clone(),
            current: (current, 0),
            check_args,
            original_state,
            index: 0,
            chaos_mode,
            fuel,
            check_mem,
            count,
            rnd: SmallRng::seed_from_u64(seed),
            variants_per_parent,
            // Set the cache size to 3GB
            hashes: config.open().expect("Could not create external cache"),
            save_compiling,
            no_preserve_semantics
        }
    }

    pub fn next(&mut self, chaos_cb: impl Fn(&Vec<u8>, &Vec<u8>, &eval::ExecutionResult),) {
        // Mutate
        let mut wasmmutate = WasmMutate::default();
        let mut wasmmutate = wasmmutate.preserve_semantics(!self.no_preserve_semantics);

        let seed = self.rnd.gen();
        eprintln!("Seed {}", seed);
        let mut wasmmutate = wasmmutate.seed(seed);
        let cp = self.current.clone();
        let origwasm = cp.0.clone();
        let wasm = wasmmutate.run(&origwasm);
        // 

        // chaos_cb(&origwasm, &self.original);

        let hash = blake3::hash(&origwasm);
        eprintln!("Mutating {}", hash);
        match wasm {
            Ok(it) => {
                // Get the first one only
                for w in it.take(self.variants_per_parent) {
                    match w {
                        Ok(b) => {
                            // Check if the hash was not generated before
                            let hash = blake3::hash(&b);
                            let hash = hash.as_bytes().to_vec();

                            if let Ok(true) = self.hashes.contains_key(&hash) {
                                // eprintln!("Already contained");
                                // We already generated this hash, so we skip it
                                
                                continue;
                            }

                            if let Some(original_state) = &self.original_state {
                                match eval::assert_same_evaluation(
                                    &original_state,
                                    &b,
                                    self.check_args.clone(),
                                    self.fuel,
                                    self.check_mem
                                ) {
                                    Some(st) => {

                                        // The val is the value is the wasm + the hash of the previous one
                                        let val = vec![self.index.to_le_bytes().to_vec(), b.clone()].concat();
                                        let _ = self.hashes.insert(hash, val).expect("Failed to insert");

                                        // Execute to see semantic equivalence

                                        if self.chaos_mode {
                                            // TODO if chaos mode...select from the DB ?
                                            let random_item = self.rnd.gen_range(0..self.hashes.len());
                                            
                                            match self.hashes.iter().take(random_item)
                                            .next() {
                                                Some(random_item) => {

                                                    match random_item {
                                                        Ok(random_item) => {
                                                            let k = random_item.0;
                                                            // eprintln!("Random key {:?} out of {}", k, self.hashes.len());
                                                            let random_curr = random_item.1;
                                                            // The val is the value is the wasm + the index in le bytes
                                                            let index = random_curr[0..8].to_vec();
                                                            let wasm = random_curr[8..].to_vec();
                                                            self.current = (wasm, usize::from_le_bytes(index.as_slice().try_into().unwrap()));
                                                            self.index  = self.current.1 + 1;
                                                        
                                                            eprintln!("=== CHAOS {}", self.index - 1);
                                                            eprintln!("=== CHAOS COUNT {}", self.hashes.len());
                                                            // Generate the file here already
                                                            chaos_cb(&b, &origwasm, &st);
                                                            continue;
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Error {}", e);
                                                            // We could not mutate the wasm, we skip it
                                                            continue
                                                        }
                                                    }
                                                }
                                                None => {
                                                    continue
                                                }
                                            };

                                        } else {
                                            self.current = (b.clone(), self.index + 1);
                                            self.index += 1;
                
                                            if self.index % 10000 == 9999 {
                                                eprintln!("{} mutations", self.index);
                                            }
                
                                            eprintln!("=== TRANSFORMED {}", self.index);
                                            break;
                                        }
                                    }
                                    None => {
                                        break
                                    }
                                }
                            }
                            
                        }
                        Err(e) => {
                            eprintln!("Error {}", e);
                            // We could not mutate the wasm, we skip it
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error {}", e);
                // We could not mutate the wasm, we skip it
            }
        }
    }
}

fn main() {
    // Init logs
    env_logger::init();

    let opts = Options::parse();
    // load the bytes from the input file
    let bytes = std::fs::read(&opts.input).expect("Could not read the input file");

    let mut stack = Stacking::new(
        bytes,
        opts.count,
        opts.seed,
        opts.remove_cache,
        opts.cache_folder,
        opts.check_args,
        opts.check_io,
        opts.fuel,
        opts.chaos_mode,
        opts.check_mem,
        opts.variants_per_parent,
        opts.save_compiling,
        opts.no_preserve_semantics
    );

    let mut C = 0;
    loop {
        stack.next(|new, parent, rs|{
            let hash = blake3::hash(&new);
            let hash2 = blake3::hash(&parent);
            let name = format!("{}.{}.{}.chaos.wasm", opts.output.to_str().unwrap(), hash2, hash);
            // Write the current to fs
            std::fs::write(&name, new)
                .expect("Could not write the output file");
            
            // Also save the cwasm
            if opts.save_compiling {
                let mut config = wasmtime::Config::default();
                let config = config.strategy(wasmtime::Strategy::Cranelift);
                // We need to save the generated machine code to disk

                // Create a new store
                let engine = wasmtime::Engine::new(&config).unwrap();

                let module = wasmtime::Module::new(&engine, &new).unwrap();

                // Serialize it
                // TODO check if it was already serialized, avoid compiling again
                let serialized = module.serialize().unwrap();
                // Save it to disk, get the filename from the argument path
                std::fs::write(format!("{}{}.cwasm", opts.output.to_str().unwrap(), hash), serialized).unwrap();
            }

        });

        if !opts.chaos_mode {
            if stack.index % opts.step == 0 {
                let name = format!("{}.{}.wasm", opts.output.to_str().unwrap(), stack.index);
                // Write the current to fs
                std::fs::write(&name, stack.current.0.clone())
                    .expect("Could not write the output file");

                eprintln!("=== STACKED");
            }
        }
    }

    // Assert that we have X different mutations
    // assert!(stack.hashes.len() == opts.count);

    // Write the current to fs
    if !opts.chaos_mode{
        std::fs::write(&opts.output, stack.current.0).expect("Could not write the output file");
    }
    //Ok(())
}

// Somesort of the same as wasm-mutate fuzz
mod eval {
    use std::fs;
    use std::hash::Hash;
    use std::sync::Arc;
    use stdio_override::StderrOverride;
    use stdio_override::StdoutOverride;
    // ./target/release/stacking tests/wb_challenge.wasm stacking.sym --seed 100 -c 2 -v 1000 --check-args wb_challenge.wasm --check-args 00 --check-args 01 --check-args 02 --check-args 03 --check-args 04 --check-args 05 --check-args 06 --check-args 07 --check-args 08 --check-args 09 --check-args 0a --check-args 0b --check-args 0c --check-args 0d --check-args 0e --check-args 0f
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    use wasmtime_wasi::sync::WasiCtxBuilder;
    use wasmtime_wasi::WasiCtx;
    use wasmtime::Val;

    fn get_current_working_dir() -> std::io::Result<std::path::PathBuf> {
        std::env::current_dir()
    }

    /// Creates the WASI support
    pub fn create_linker(engine: &wasmtime::Engine) -> wasmtime::Linker<wasmtime_wasi::WasiCtx> {
        let mut linker = wasmtime::Linker::new(&engine);

        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
        // These methods are not in WASI by default, yet, let us assume they are
        // It is the same assumption of Swivel
        linker.clone()
    }

    pub type ExecutionResult = (
        Vec<u64>,
        Vec<Val>,
        String,
        String,
        wasmtime::Module,
        wasmtime::Instance,
        std::time::Duration,
    );

    pub fn execute_single(wasm: &[u8], args: Vec<String>, fuel: u64) -> Option<ExecutionResult> {
        let mut config = wasmtime::Config::default();
        let config = config.strategy(wasmtime::Strategy::Cranelift);

        // Set no opt for faster execution
        let config = config.cranelift_opt_level(wasmtime::OptLevel::None);

        config.cranelift_nan_canonicalization(true);
        if fuel > 0 {
            config.consume_fuel(true);
        }

        // config.consume_fuel(true);

        let engine = wasmtime::Engine::new(&config).unwrap();

        let module = match wasmtime::Module::new(&engine, &wasm) {
            Ok(o) => o,
            Err(_) => return None,
        };

        let folder_of_bin = get_current_working_dir().unwrap().display().to_string();

        let mut wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .args(&args)
            .unwrap()
            // Preopen in the CWD
            .preopened_dir(
                wasmtime_wasi::sync::Dir::open_ambient_dir(
                    folder_of_bin.clone(),
                    wasmtime_wasi::sync::ambient_authority(),
                )
                .unwrap(),
                ".",
            )
            .unwrap()
            .build();

        let mut linker = create_linker(&engine);
        let stdout_file = "./stdout.txt";
        let stderr_file = "./stderr.txt";

        // Recreate the file
        let _ = std::fs::File::create(&stdout_file);
        let _ = std::fs::File::create(&stderr_file);

        let guardout = StdoutOverride::override_file(stdout_file).unwrap();
        let guarderr = StderrOverride::override_file(stderr_file).unwrap();

        let mut store1 = wasmtime::Store::new(&engine, wasi.clone());
        if fuel > 0 {
            store1.add_fuel(fuel).unwrap();
            store1.out_of_fuel_trap();
        }

        if let Ok(instance1) = linker.instantiate(&mut store1, &module) {

            if let Some(func1) = instance1.get_func(&mut store1, "_start") {

                let now = std::time::Instant::now();

                match func1.call(&mut store1, &mut [], &mut []) {
                    Ok(e) => {
                        let elapsed = now.elapsed();

                        let stdout = fs::read_to_string(stdout_file);
                        let stderr = fs::read_to_string(stderr_file);

                        match (stdout, stderr) {
                            (Ok(stdout), Ok(stderr)) => {

                                // Get mem hash
                                let (mem_hashes, glob_vals) = assert_same_state(&module, &mut store1, instance1);
                        
                                drop(guardout);
                                drop(guarderr);

                                return Some((
                                    mem_hashes,
                                    glob_vals,
                                    stdout.into(),
                                    stderr.into(),
                                    module,
                                    instance1,
                                    elapsed,
                                ))
                            }
                            _ => {
                                eprintln!("Error reading stderr/out");
                                drop(guardout);
                                drop(guarderr);

                                return None;
                            }
                        }
                
                
                    }
                    Err(e) => {
                        let elapsed = now.elapsed();


                        let stdout = fs::read_to_string(stdout_file);
                        let stderr = fs::read_to_string(stderr_file);

                        match (stdout, stderr) {
                            (Ok(stdout), Ok(stderr)) => {

                                eprintln!("Runtime error {e} {} {}", stdout, stderr);
                            }
                            _ => {
                                // do nothing
                            }
                        }
                
                        drop(guardout);
                        drop(guarderr);
                        return None
                    }
                }

            }
        }

        return None
        
    }

    /// Compile, instantiate, and evaluate both the original and mutated Wasm.
    ///
    /// We should get identical results because we told `wasm-mutate` to preserve
    /// semantics.
    pub fn assert_same_evaluation(
        original_result: &ExecutionResult,
        mutated_wasm: &[u8],
        args: Vec<String>,fuel: u64, check_mem: bool
    ) -> Option<ExecutionResult> {
        match execute_single(mutated_wasm, args.clone(), fuel)
        {
                
        Some((mem2, glob2,  stdout2, stderr2, _mod2, instance2, time2))
             => {
                let (mem1, glob1, stdout1, stderr1, mod1, instance1, _) = original_result;
                if *stdout1 != stdout2 || *stderr1 != stderr2 {
                    eprintln!("Std is not the same");
                    return None;
                }
                // Now we compare the stores
                if check_mem {
                    if mem1.len() != mem2.len() {
                        eprintln!("Memories are not the same");
                        return None;
                    }

                    if glob1.len() != glob2.len() {
                        eprintln!("Globals are not the same");
                        return None;
                    }

                    // Compare the memories
                    // Zip them and compare the hashes in order
                    for (m1, m2) in mem1.iter().zip(mem2.iter()) {
                        if m1 != m2 {
                            eprintln!("Memories are not the same");
                            return None;
                        }
                    }

                    // The same for globals
                    for (g1, g2) in glob1.iter().zip(glob2.iter()) {
                        if ! assert_val_eq(&g1, &g2) {
                            eprintln!("Globals are not the same");
                            return None;
                        }
                    }

                    eprintln!("Invalid state");
                    return None;
                };
                // Compare the memories

                // Compare the globals

                eprintln!("Time {}ns", time2.as_nanos());

                return Some((mem2, glob2,  stdout2, stderr2, _mod2, instance2, time2));
            }
            _ => return None,
        }
    }

    fn assert_same_state(
        orig_module: &wasmtime::Module,
        orig_store: &mut wasmtime::Store<WasiCtx>,
        orig_instance: wasmtime::Instance
    ) -> (
        Vec<u64>,
        Vec<Val>,
    ) {
        let mut mem_hashes = vec![];
        let mut glob_vals = vec![];
        for export in orig_module.exports() {
            match export.ty() {
                wasmtime::ExternType::Global(_) => {
                    let orig = orig_instance
                        .get_export(&mut *orig_store, export.name())
                        .unwrap()
                        .into_global()
                        .unwrap()
                        .get(&mut *orig_store);

                    glob_vals.push(orig);
                }
                wasmtime::ExternType::Memory(_) => {
                    let orig = orig_instance
                        .get_export(&mut *orig_store, export.name())
                        .unwrap()
                        .into_memory()
                        .unwrap();
                    let mut h = DefaultHasher::default();
                    orig.data(&orig_store).hash(&mut h);
                    let orig = h.finish();
                    
                    mem_hashes.push(orig);
                }
                _ => continue,
            }
        }

        return (mem_hashes, glob_vals);
    }

    fn assert_val_eq(orig_val: &wasmtime::Val, mutated_val: &wasmtime::Val) -> bool {
        match (orig_val, mutated_val) {
            (wasmtime::Val::I32(o), wasmtime::Val::I32(m)) => return o == m,
            (wasmtime::Val::I64(o), wasmtime::Val::I64(m)) => return o == m,
            (wasmtime::Val::F32(o), wasmtime::Val::F32(m)) => {
                let o = f32::from_bits(*o);
                let m = f32::from_bits(*m);
                return o == m || (o.is_nan() && m.is_nan());
            }
            (wasmtime::Val::F64(o), wasmtime::Val::F64(m)) => {
                let o = f64::from_bits(*o);
                let m = f64::from_bits(*m);
                return o == m || (o.is_nan() && m.is_nan());
            }
            (wasmtime::Val::V128(o), wasmtime::Val::V128(m)) => return o == m,
            (wasmtime::Val::ExternRef(o), wasmtime::Val::ExternRef(m)) => {
                return o.is_none() == m.is_none()
            }
            (wasmtime::Val::FuncRef(o), wasmtime::Val::FuncRef(m)) => {
                return o.is_none() == m.is_none()
            }
            (o, m) => {
                eprintln!(
                    "mutated and original Wasm diverged: orig = {:?}; mutated = {:?}",
                    o, m,
                );
                return false;
            }
        }
    }
}
