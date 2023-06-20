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

    /// The output Wasm binary.
    output: PathBuf,
}

fn swap(current: &mut Vec<u8>, new_interesting: Vec<u8>) {
    *current = new_interesting;
}

struct Stacking {
    current: Vec<u8>,
    original: Vec<u8>,
    check_args: Vec<String>,
    original_state: Option<eval::ExecutionResult>,
    index: usize,
    fuel: u64,
    count: usize,
    rnd: SmallRng,
    // The hashes will prevent regression and non performed transformations
    hashes: sled::Db,
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
                    eprintln!("Original time {}ns", it.5.as_nanos());
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
            current,
            check_args,
            original_state,
            index: 0,
            fuel,
            count,
            rnd: SmallRng::seed_from_u64(seed),
            // Set the cache size to 3GB
            hashes: config.open().expect("Could not create external cache"),
        }
    }

    pub fn next(&mut self) {
        let mut new = self.current.clone();

        // Mutate
        let mut wasmmutate = WasmMutate::default();
        let mut wasmmutate = wasmmutate.preserve_semantics(true);

        let seed = self.rnd.gen();
        eprintln!("Seed {}", seed);
        let mut wasmmutate = wasmmutate.seed(seed);
        let cp = self.current.clone();
        let wasm = wasmmutate.run(&cp);

        match wasm {
            Ok(it) => {
                // Get the first one only
                for w in it {
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
                                if !eval::assert_same_evaluation(
                                    &self.original,
                                    &b,
                                    self.check_args.clone(),
                                    self.fuel,
                                ) {
                                    break;
                                }
                            }
                            self.hashes.insert(hash, b"1");

                            // Execute to see semantic equivalence

                            self.current = b.clone();
                            self.index += 1;

                            if self.index % 10000 == 9999 {
                                eprintln!("{} mutations", self.index);
                            }

                            eprintln!("=== TRANSFORMED {}", self.index);
                            break;
                        }
                        Err(e) => {
                            // We could not mutate the wasm, we skip it
                        }
                    }
                }
            }
            Err(e) => {
                // We could not mutate the wasm, we skip it
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    // Init logs
    env_logger::init();

    let opts = Options::parse();
    // load the bytes from the input file
    let bytes = std::fs::read(&opts.input).context("Could not read the input file")?;

    let mut stack = Stacking::new(
        bytes,
        opts.count,
        opts.seed,
        opts.remove_cache,
        opts.cache_folder,
        opts.check_args,
        opts.check_io,
        opts.fuel,
    );

    loop {
        stack.next();

        if stack.index % opts.step == 0 {
            let name = format!("{}.{}.wasm", opts.output.to_str().unwrap(), stack.index);
            // Write the current to fs
            std::fs::write(&name, stack.current.clone())
                .context("Could not write the output file")?;

            eprintln!("=== STACKED");
        }
        if stack.index == opts.count {
            break;
        }
    }

    // Assert that we have X different mutations
    // assert!(stack.hashes.len() == opts.count);

    // Write the current to fs
    std::fs::write(&opts.output, stack.current).context("Could not write the output file")?;
    Ok(())
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
        wasmtime::Store<WasiCtx>,
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

        let instance1 = linker.instantiate(&mut store1, &module).unwrap();

        let func1 = instance1.get_func(&mut store1, "_start").unwrap();

        let now = std::time::Instant::now();
        let r1 = func1.call(&mut store1, &mut [], &mut []).unwrap();
        let elapsed = now.elapsed();

        let stdout = fs::read_to_string(stdout_file).expect("Cannot read stdout");
        let stderr = fs::read_to_string(stderr_file).expect("Cannot read stderr");
        drop(guardout);
        drop(guarderr);
        Some((
            store1,
            stdout.into(),
            stderr.into(),
            module,
            instance1,
            elapsed,
        ))
    }

    /// Compile, instantiate, and evaluate both the original and mutated Wasm.
    ///
    /// We should get identical results because we told `wasm-mutate` to preserve
    /// semantics.
    pub fn assert_same_evaluation(
        original_wasm: &[u8],
        mutated_wasm: &[u8],
        args: Vec<String>,fuel: u64
    ) -> bool {
        match (
            execute_single(original_wasm, args.clone(), fuel),
            execute_single(mutated_wasm, args.clone(), fuel),
        ) {
            (
                Some((mut store1, stdout1, stderr1, mod1, instance1, _)),
                Some((mut store2, stdout2, stderr2, _mod2, instance2, time2)),
            ) => {
                if stdout1 != stdout2 || stderr1 != stderr2 {
                    eprintln!("Std is not the same");
                    return false;
                }
                // Now we compare the stores
                if !assert_same_state(&mod1, &mut store1, instance1, &mut store2, instance2) {
                    eprintln!("Invalid state");
                    return false;
                };
                // Compare the memories

                // Compare the globals

                eprintln!("Time {}ns", time2.as_nanos());

                return true;
            }
            _ => return false,
        }
    }

    fn assert_same_state(
        orig_module: &wasmtime::Module,
        orig_store: &mut wasmtime::Store<WasiCtx>,
        orig_instance: wasmtime::Instance,
        mutated_store: &mut wasmtime::Store<WasiCtx>,
        mutated_instance: wasmtime::Instance,
    ) -> bool {
        for export in orig_module.exports() {
            match export.ty() {
                wasmtime::ExternType::Global(_) => {
                    let orig = orig_instance
                        .get_export(&mut *orig_store, export.name())
                        .unwrap()
                        .into_global()
                        .unwrap()
                        .get(&mut *orig_store);
                    let mutated = mutated_instance
                        .get_export(&mut *mutated_store, export.name())
                        .unwrap()
                        .into_global()
                        .unwrap()
                        .get(&mut *mutated_store);

                    if !assert_val_eq(&orig, &mutated) {
                        eprintln!("Globals are not the same");
                        return false;
                    }
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
                    let mutated = mutated_instance
                        .get_export(&mut *mutated_store, export.name())
                        .unwrap()
                        .into_memory()
                        .unwrap();
                    let mut h = DefaultHasher::default();
                    mutated.data(&mutated_store).hash(&mut h);
                    let mutated = h.finish();

                    if orig != mutated {
                        eprintln!("original and mutated Wasm memories diverged");
                        return false;
                    }
                }
                _ => continue,
            }
        }

        return true;
    }

    /*
    fn assert_same_calls(
        orig_module: &wasmtime::Module,
        orig_store: &mut wasmtime::Store<WasiCtx>,
        orig_instance: wasmtime::Instance,
        mutated_store: &mut wasmtime::Store<WasiCtx>,
        mutated_instance: wasmtime::Instance,
    ) -> bool {
        for export in orig_module.exports() {
            let func_ty = match export.ty() {
                wasmtime::ExternType::Func(func_ty) => func_ty,
                _ => continue,
            };
            let orig_func = orig_instance
                .get_func(&mut *orig_store, export.name())
                .unwrap();
            let mutated_func = mutated_instance
                .get_func(&mut *mutated_store, export.name())
                .unwrap();
            let args = dummy::dummy_values(func_ty.params());
            let mut orig_results = vec![Val::I32(0); func_ty.results().len()];
            let mut mutated_results = orig_results.clone();
            log::debug!("invoking `{}`", export.name());
            match (
                {
                    orig_store.add_fuel(1_000).unwrap();
                    orig_func.call(&mut *orig_store, &args, &mut orig_results)
                },
                {
                    mutated_store.add_fuel(1000).unwrap();
                    mutated_func.call(&mut *mutated_store, &args, &mut mutated_results)
                },
            ) {
                (Ok(()), Ok(())) => {
                    for (orig_val, mutated_val) in orig_results.iter().zip(mutated_results.iter()) {
                        assert_val_eq(orig_val, mutated_val);
                    }
                }
                // If either test case ran out of fuel then that's ok since
                // mutation may add code or delete code which causes one side to
                // take more or less fuel than the other. In this situation,
                // however, execution has diverged so throw out the test case.
                (Err(e), _) | (_, Err(e))
                    if e.downcast_ref() == Some(&wasmtime::Trap::OutOfFuel) =>
                {
                    return false
                }
                (Err(orig), Err(mutated)) => {
                    log::debug!("original error {orig:?}");
                    log::debug!("mutated error {mutated:?}");
                    continue;
                }
                (orig, mutated) => panic!(
                    "mutated and original Wasm diverged: orig = {:?}; mutated = {:?}",
                    orig, mutated,
                ),
            }
        }

        true
    }

     */

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
