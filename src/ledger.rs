use std::{collections::HashMap, fmt::Display};

use thiserror::Error;

use crate::ledger::balance::Balance;

pub mod balance;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Duplicate transaction id: {0}")]
    DuplicateTransaction(Tx),

    #[error("Missing transaction id: {0}")]
    MissingTransaction(Tx),

    #[error("No initial deposit for client: {0}")]
    NoInitialDeposit(Client),

    #[error("Unexpected transaction status: {0}")]
    UnexpectedTxStatus(TxStatus),

    #[error("Client account is frozen: {0}")]
    FrozenAccountError(Client),

    #[error(transparent)]
    BalanceError(#[from] balance::Error),
}

/// UserId alias for ease of reading
pub type Client = u16;

/// Transaction id alias for ease of reading
pub type Tx = u32;

/// Each of the individual operations which we may process
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transaction {
    Deposit { client: Client, tx: Tx, amount: f64 },
    Withdrawal { client: Client, tx: Tx, amount: f64 },
    Dispute { client: Client, tx: Tx },
    Resolve { client: Client, tx: Tx },
    ChargeBack { client: Client, tx: Tx },
}

impl Transaction {
    /// Get a reference to the client id for this transaction
    pub fn client(&self) -> &Client {
        match self {
            Transaction::Deposit { client, .. } => client,
            Transaction::Withdrawal { client, .. } => client,
            Transaction::Dispute { client, .. } => client,
            Transaction::Resolve { client, .. } => client,
            Transaction::ChargeBack { client, .. } => client,
        }
    }

    /// Get a reference to the transaction id for this transaction
    pub fn tx(&self) -> &Tx {
        match self {
            Transaction::Deposit { tx, .. } => tx,
            Transaction::Withdrawal { tx, .. } => tx,
            Transaction::Dispute { tx, .. } => tx,
            Transaction::Resolve { tx, .. } => tx,
            Transaction::ChargeBack { tx, .. } => tx,
        }
    }

    /// Check if this transaction is a deposit
    pub fn is_deposit(&self) -> bool {
        matches!(self, &Transaction::Deposit { .. })
    }

    fn key(&self) -> (Client, Tx) {
        match self {
            Transaction::Deposit { client, tx, .. } => (*client, *tx),
            Transaction::Withdrawal { client, tx, .. } => (*client, *tx),
            Transaction::Dispute { client, tx } => (*client, *tx),
            Transaction::Resolve { client, tx } => (*client, *tx),
            Transaction::ChargeBack { client, tx } => (*client, *tx),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TxStatus {
    /// All valid transactions are registered with an active status
    #[default]
    Active,
    /// The transaction is in dispute
    Disputed,
    /// The dispute has been resolved and the funds are released
    Resolved,
    /// The transaction has been charged back and the balance has been removed
    ChargedBack,
}

impl Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TxStatus::Active => "Active",
            TxStatus::Disputed => "Disputed",
            TxStatus::Resolved => "Resolved",
            TxStatus::ChargedBack => "ChargedBack",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    /// The transaction for this entry
    pub t: Transaction,

    /// The status of this transaction
    status: TxStatus,
}

impl Entry {
    fn new(t: Transaction) -> Self {
        Entry {
            t,
            status: TxStatus::Active,
        }
    }

    fn dispute(&mut self) -> Result<(), Error> {
        if self.status != TxStatus::Active {
            Err(Error::UnexpectedTxStatus(self.status))?
        }

        self.status = TxStatus::Disputed;

        Ok(())
    }

    fn resolve(&mut self) -> Result<(), Error> {
        if self.status != TxStatus::Disputed {
            Err(Error::UnexpectedTxStatus(self.status))?
        }

        self.status = TxStatus::Resolved;

        Ok(())
    }

    fn charge_back(&mut self) -> Result<(), Error> {
        if self.status != TxStatus::Disputed {
            Err(Error::UnexpectedTxStatus(self.status))?
        }

        self.status = TxStatus::ChargedBack;

        Ok(())
    }
}

/// Each user will have a ledger of transactions. This will aim at being compact
/// But perhaps expensive at large numbers of transactions for now
pub struct Ledger {
    /// Mapping of a transaction by id to the index it's written into memory
    client_tx_to_idx: HashMap<(Client, Tx), usize>,

    /// Balances for each client
    balance: HashMap<Client, Balance>,

    /// All transactions within this ledger
    transactions: Vec<Entry>,
}

impl Ledger {
    fn new() -> Self {
        Ledger {
            client_tx_to_idx: HashMap::new(),
            balance: HashMap::new(),
            transactions: Vec::new(),
        }
    }

    pub fn process_transaction(&mut self, t: Transaction) -> Result<(), Error> {
        let key = t.key();

        // --- Check for Reasons not to Process ---

        // Ensure we're not looking at a duplicate transaction
        if matches!(
            &t,
            &Transaction::Deposit { .. } | &Transaction::Withdrawal { .. }
        ) && self.client_tx_to_idx.contains_key(&key)
        {
            Err(Error::DuplicateTransaction(*t.tx()))?
        }

        // Return early if there is no balance for this client on non-deposit transactions
        if !t.is_deposit() && !self.balance.contains_key(t.client()) {
            Err(Error::NoInitialDeposit(*t.client()))?
        } else if !self.balance.contains_key(t.client()) {
            self.balance.insert(*t.client(), Balance::new(*t.client()));
        }

        // --- Attempt to Process ---
        let b = self.balance.get_mut(t.client()).expect("Initialized above");

        // If the balance is locked this transaction will be ignored
        if b.locked() {
            Err(Error::FrozenAccountError(*t.client()))?;
        }

        match &t {
            Transaction::Deposit { amount, .. } => {
                b.deposit(*amount)?;
            }
            Transaction::Withdrawal { amount, .. } => {
                b.withdraw(*amount)?;
            }
            Transaction::Dispute { .. } => {
                if let Some(idx) = self.client_tx_to_idx.get(&key) {
                    let entry = self
                        .transactions
                        .get_mut(*idx)
                        .expect("idx tracks growing allocation");

                    // This check should prevent the below hold from raising it's own error
                    // As we enforce strict state transitions on the private status
                    entry.dispute()?;

                    if let &Transaction::Deposit { amount, .. } = &entry.t {
                        b.hold(*t.tx(), amount)?
                    } else if let &Transaction::Withdrawal { amount, .. } = &entry.t {
                        b.hold(*t.tx(), -1f64 * amount)?
                    }
                } else {
                    Err(Error::MissingTransaction(*t.tx()))?;
                }
            }
            Transaction::Resolve { .. } => {
                if let Some(idx) = self.client_tx_to_idx.get(&key) {
                    let entry = self
                        .transactions
                        .get_mut(*idx)
                        .expect("idx tracks growing allocation");

                    // This ensures that this transaction was in the "disputed" state and forces it forward to resolved
                    entry.resolve()?;

                    // Remove the hold from this entry on the balance.
                    b.remove_hold(*t.tx())?;
                } else {
                    Err(Error::MissingTransaction(*t.tx()))?;
                }
            }
            Transaction::ChargeBack { .. } => {
                if let Some(idx) = self.client_tx_to_idx.get(&key) {
                    let entry = self
                        .transactions
                        .get_mut(*idx)
                        .expect("idx tracks growing allocation");

                    // This ensures that this transaction was in the "disputed" state and forces it forward to resolved
                    entry.charge_back()?;

                    // Remove the hold from this entry on the balance.
                    b.apply_hold(*t.tx())?;
                    b.lock_balance();
                } else {
                    Err(Error::MissingTransaction(*t.tx()))?;
                }
            }
        }

        // --- Register deposits and withdrawals ---

        if matches!(
            &t,
            &Transaction::Deposit { .. } | &Transaction::Withdrawal { .. }
        ) {
            // Get new index for this transaction
            let index = self.transactions.len();
            self.client_tx_to_idx.insert(key, index);

            // Add the transaction as an entry
            let entry = Entry::new(t);
            self.transactions.push(entry);
        }

        Ok(())
    }

    /// Get the balance of a client in the ledger. If the client has been registered
    /// There will be a Some(balance) returned
    pub fn get_available_balance(&self, client: Client) -> Option<f64> {
        match self.balance.get(&client) {
            Some(b) => Some(b.available()),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::{Result, anyhow};

    use crate::ledger::{Ledger, Transaction};

    #[test]
    fn process_first_deposit() -> Result<()> {
        let t = Transaction::Deposit {
            client: 0,
            tx: 1,
            amount: 100f64,
        };
        let mut ledger = Ledger::new();

        // Should succeed
        ledger.process_transaction(t)?;

        assert_eq!(100f64, ledger.get_available_balance(0).unwrap());

        Ok(())
    }

    #[test]
    fn deposit_and_dispute() -> Result<()> {
        let t1 = Transaction::Deposit {
            client: 0,
            tx: 1,
            amount: 100f64,
        };
        let t2 = Transaction::Dispute { client: 0, tx: 1 };

        let mut ledger = Ledger::new();

        ledger.process_transaction(t1)?;
        ledger.process_transaction(t2)?;

        assert_eq!(0f64, ledger.get_available_balance(0).unwrap());

        Ok(())
    }

    #[test]
    fn deposit_withdraw_dispute() -> Result<()> {
        let t1 = Transaction::Deposit {
            client: 0,
            tx: 1,
            amount: 100f64,
        };
        let t2 = Transaction::Withdrawal {
            client: 0,
            tx: 2,
            amount: 10f64,
        };
        let t3 = Transaction::Dispute { client: 0, tx: 2 };

        let mut ledger = Ledger::new();

        ledger.process_transaction(t1)?;
        ledger.process_transaction(t2)?;
        ledger.process_transaction(t3)?;

        assert_eq!(100f64, ledger.get_available_balance(0).unwrap());

        Ok(())
    }

    #[test]
    fn deposit_withdraw_dispute_resolve() -> Result<()> {
        let t1 = Transaction::Deposit {
            client: 0,
            tx: 1,
            amount: 100f64,
        };
        let t2 = Transaction::Withdrawal {
            client: 0,
            tx: 2,
            amount: 10f64,
        };
        let t3 = Transaction::Dispute { client: 0, tx: 2 };
        let t4 = Transaction::Resolve { client: 0, tx: 2 };

        let mut ledger = Ledger::new();

        ledger.process_transaction(t1)?;
        ledger.process_transaction(t2)?;
        ledger.process_transaction(t3)?;
        ledger.process_transaction(t4)?;

        assert_eq!(90f64, ledger.get_available_balance(0).unwrap());

        Ok(())
    }

    #[test]
    fn deposit_dispute_charge_back_no_other_actions_succeed() -> Result<()> {
        let t1 = Transaction::Deposit {
            client: 0,
            tx: 1,
            amount: 100f64,
        };
        let t2 = Transaction::Dispute { client: 0, tx: 1 };
        let t3 = Transaction::Deposit {
            client: 0,
            tx: 2,
            amount: 50f64,
        };
        let t4 = Transaction::ChargeBack { client: 0, tx: 1 };

        let mut ledger = Ledger::new();

        ledger.process_transaction(t1)?;
        ledger.process_transaction(t2)?;
        ledger.process_transaction(t3)?;
        ledger.process_transaction(t4)?;

        assert_eq!(50f64, ledger.get_available_balance(0).unwrap());

        // At this point no further actions should succeed
        assert!(
            ledger
                .process_transaction(Transaction::Deposit {
                    client: 0,
                    tx: 3,
                    amount: 10f64
                })
                .is_err()
        );
        assert!(
            ledger
                .process_transaction(Transaction::Withdrawal {
                    client: 0,
                    tx: 4,
                    amount: 40f64
                })
                .is_err()
        );
        assert!(
            ledger
                .process_transaction(Transaction::Dispute { client: 0, tx: 2 })
                .is_err()
        );
        assert!(
            ledger
                .process_transaction(Transaction::Resolve { client: 0, tx: 2 })
                .is_err()
        );
        assert!(
            ledger
                .process_transaction(Transaction::ChargeBack { client: 0, tx: 2 })
                .is_err()
        );

        assert_eq!(50f64, ledger.get_available_balance(0).unwrap());

        Ok(())
    }
}
