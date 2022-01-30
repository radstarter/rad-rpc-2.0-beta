#[macro_use]
extern crate lazy_static;

use parking_lot::RwLock;
use std::sync::Arc;

mod config;
mod formatter;
mod json_rpc_thread;
mod scrypto_helpers;
mod setup;

lazy_static! {
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

    let handle2 = std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_secs(180));

        let writer_lock = CONFIG.write();
        let _ = parking_lot::RwLockWriteGuard::map(writer_lock, |config| {
            config.increment_epoch();
            config
        });
    });

    let _ = handle.join();
}
