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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BalanceSnapshot {
    pub client: Client,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
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

    /// The total amount of money being held in place
    /// As withdrawals are immediately removed from the client balance that money
    /// is not considered held. It's considered withdrawn. To count it as held
    /// would improperly increase the amount of money available for subsequent
    /// withdrawals putting the account servicer at risk.
    pub fn held(&self) -> f64 {
        let mut total_held = 0f64;
        for v in self.holds.values() {
            if *v > 0f64 {
                total_held += *v
            }
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

    pub fn snapshot(&self) -> BalanceSnapshot {
        BalanceSnapshot {
            client: self.client,
            available: self.available(),
            held: self.held(),
            total: self.total,
            locked: self.locked,
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::ledger::balance::Balance;

    #[test]
    fn deposit_and_withdraw() -> Result<()> {
        let mut b = Balance::new(0);

        b.deposit(100f64)?;
        b.withdraw(10f64)?;

        assert_eq!(90f64, b.available());

        Ok(())
    }

    #[test]
    fn deposit_withdraw_hold() -> Result<()> {
        let mut b = Balance::new(0);

        b.deposit(100f64)?;
        b.withdraw(10f64)?;

        // Place a hold on the withdrawal
        b.hold(2, -10f64)?;

        assert_eq!(90f64, b.available());

        Ok(())
    }
}
