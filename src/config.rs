use radix_engine::ledger::InMemoryLedger;

pub struct Config {
    pub nonce: u64,
    pub epoch: u64,
    pub updated: bool,
    pub ledger: InMemoryLedger,
}

impl Config {
    pub fn new() -> Config {
        Config {
            nonce: 0_u64,
            epoch: 0_u64,
            updated: false,
            ledger: InMemoryLedger::with_bootstrap(),
        }
    }

    pub fn store_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
        self.updated = true;
    }

    pub fn increment_epoch(&mut self) {
        self.epoch += 1;
    }

    pub fn load(&mut self) -> (u64, u64, &mut InMemoryLedger) {
        (self.epoch, self.nonce, &mut self.ledger)
    }

    pub fn load_immutable(&self) -> &InMemoryLedger {
        &self.ledger
    }
}
