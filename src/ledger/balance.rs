//! Sub module for tracking the balance of a given client

use std::collections::HashMap;

use thiserror::Error;

use crate::ledger::{Client, Tx};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Insufficient funds")]
    InsufficientFunds,
}

/// Struct for tracking the underlying balance of a client
#[derive(Debug, Clone)]
pub struct Balance {
    /// Client which owns this balance entry
    client: Client,

    /// Total balance within this account
    total: f64,

    /// Track individual holds on transactions
    holds: HashMap<Tx, f64>,

    /// Is this account locked
    locked: bool,
}

impl Balance {
    fn new(client: Client) -> Self {
        Balance {
            client,
            total: 0f64,
            holds: HashMap::new(),
            locked: false,
        }
    }

    fn available(&self) -> f64 {
        self.total - self.held()
    }

    fn total(&self) -> f64 {
        self.total
    }

    fn held(&self) -> f64 {
        let mut total_held = 0f64;
        for v in self.holds.values() {
            total_held += *v;
        }
        total_held
    }

    fn locked(&self) -> bool {
        self.locked
    }

    fn lock_balance(&mut self) {
        self.locked = true;
    }
}
