//! Sub module for tracking the balance of a given client

use std::collections::HashMap;

use thiserror::Error;

use crate::ledger::{Client, Tx};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Account locked")]
    AccountLocked,

    #[error("Tx Already Held: {0}")]
    MultiHoldError(Tx),

    #[error("No hold on Tx: {0}")]
    NoHoldError(Tx),
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
    pub fn new(client: Client) -> Self {
        Balance {
            client,
            total: 0f64,
            holds: HashMap::new(),
            locked: false,
        }
    }

    pub fn available(&self) -> f64 {
        self.total - self.held()
    }

    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn held(&self) -> f64 {
        let mut total_held = 0f64;
        for v in self.holds.values() {
            total_held += *v;
        }
        total_held
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn lock_balance(&mut self) {
        self.locked = true;
    }

    /// Add funds to this balance
    pub fn deposit(&mut self, amount: f64) -> Result<(), Error> {
        if self.locked {
            Err(Error::AccountLocked)?
        }

        self.total += amount;

        Ok(())
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<(), Error> {
        if self.total < amount {
            Err(Error::InsufficientFunds)?
        }

        if self.locked {
            Err(Error::AccountLocked)?
        }

        self.total -= amount;

        Ok(())
    }

    pub fn hold(&mut self, tx: Tx, amount: f64) -> Result<(), Error> {
        if self.holds.contains_key(&tx) {
            Err(Error::MultiHoldError(tx))?;
        }

        self.holds.insert(tx, amount);

        Ok(())
    }

    pub fn remove_hold(&mut self, tx: Tx) -> Result<(), Error> {
        if !self.holds.contains_key(&tx) {
            Err(Error::NoHoldError(tx))?;
        }

        self.holds.remove(&tx);

        Ok(())
    }

    pub fn apply_hold(&mut self, tx: Tx) -> Result<(), Error> {
        if !self.holds.contains_key(&tx) {
            Err(Error::NoHoldError(tx))?;
        }

        self.total -= self.holds.get(&tx).expect("Checked in if clause");
        self.holds.remove(&tx);

        Ok(())
    }
}
