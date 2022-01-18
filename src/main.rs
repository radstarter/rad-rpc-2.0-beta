#[macro_use]
extern crate lazy_static;

use parking_lot::RwLock;
use radix_engine::ledger::*;
use std::sync::Arc;

mod config;
mod formatter;
mod json_rpc_thread;
mod scrypto_helpers;
mod setup;

lazy_static! {
    static ref LEDGER: Arc<RwLock<InMemoryLedger>> =
        Arc::new(RwLock::new(InMemoryLedger::with_bootstrap()));
    static ref CONFIG: Arc<RwLock<config::Config>> = Arc::new(RwLock::new(config::Config::new()));
}

fn main() {
    //Change the path to the wasm files that it needs to publish in setup::create_setup_file
    setup::create_setup_file_example();
    setup::run_setup();

    println!("Spawning HTTP Server");

    let handle = std::thread::spawn(|| {
        json_rpc_thread::rpc_thread();
    });

    let _ = handle.join();
}
