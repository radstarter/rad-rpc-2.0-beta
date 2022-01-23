use radix_engine::ledger::InMemoryLedger;
use radix_engine::transaction::TransactionExecutor;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct Config {
    pub nonce: AtomicUsize,
    pub epoch: AtomicUsize,
    pub updated: AtomicBool,
}

impl Config {
    pub fn new() -> Config {
        Config {
            nonce: AtomicUsize::new(0),
            epoch: AtomicUsize::new(0),
            updated: AtomicBool::new(false),
        }
    }

    pub fn store_nonce(&mut self, executor: &TransactionExecutor<InMemoryLedger>) {
        let nonce_save = executor.nonce() as usize;
        self.updated.store(true, Ordering::SeqCst);
        self.nonce.store(nonce_save, Ordering::SeqCst);
    }

    pub fn load_nonce(&mut self) -> (u64, u64) {
        let epoch = self.epoch.load(Ordering::SeqCst);
        let nonce = self.nonce.load(Ordering::SeqCst);

        (epoch as u64, nonce as u64)
    }
}
