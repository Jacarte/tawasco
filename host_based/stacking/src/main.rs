use anyhow::Context;
use clap::Parser;
use core::sync::atomic::Ordering::{Relaxed, SeqCst};
use rand::Rng;
use rand::rngs::SmallRng;
use std::collections::hash_map::DefaultHasher;
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
use std::collections::HashSet;
use std::borrow::BorrowMut;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use rand::SeedableRng;

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
    #[arg(long = "cache_folder", default_value = "cache")]
    cache_folder: String,
    /// Erase cache on start
    #[arg(long = "remove_cache", default_value = "false")]
    remove_cache: bool,

    /// The output Wasm binary.
    output: PathBuf,
}

fn swap(current: &mut Vec<u8>,
    new_interesting: Vec<u8>) {
        *current = new_interesting;
}

struct Stacking {
    current: Vec<u8>,
    index: usize,
    count: usize,
    rnd: SmallRng,
    // The hashes will prevent regression and non performed transformations
    hashes: sled::Db,
}

impl Stacking {
    
    pub fn new(current: Vec<u8>, count: usize, seed: u64, remove_cache: bool, cache_dir: String) -> Self {
        // Remove db if exist
        if remove_cache {
            std::fs::remove_dir_all(&cache_dir.clone());
        }

        let config = sled::Config::default()
            .path(cache_dir.clone().to_owned())
            .cache_capacity(/* 4Gb */ 4 * 1024 * 1024 * 1024);

        Self {
            current,
            index: 0,
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

                            self.hashes.insert(hash, b"1");
                            
                            //orig = &b.clone();
                            self.current = b.clone();
                            self.index += 1;

                            if self.index % 10000 == 9999 {
                                eprintln!("{} mutations", self.index);
                            }

                            eprintln!("=== TRANSFORMED");
                            break;

                        }
                        Err(e) => {
                            // We could not mutate the wasm, we skip it
                        }
                    }

                }
                

            },
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

    let mut stack = Stacking::new(bytes, opts.count, opts.seed, opts.remove_cache, opts.cache_folder);

    loop {
        stack.next();

        if stack.index % opts.step == 0 {
            let name = format!("{}.{}.wasm", opts.output.to_str().unwrap(), stack.index);
            // Write the current to fs
            std::fs::write(&name, stack.current.clone()).context("Could not write the output file")?;

            eprintln!("=== STACKED");
        }
        if stack.index == opts.count {
            break
        }
    }

    // Assert that we have X different mutations
    // assert!(stack.hashes.len() == opts.count);
    
    // Write the current to fs
    std::fs::write(&opts.output, stack.current).context("Could not write the output file")?;
    Ok(())
}
