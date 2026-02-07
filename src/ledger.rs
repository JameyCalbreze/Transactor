use std::{
    collections::{HashMap, HashSet},
    default,
};

use thiserror::Error;

use crate::ledger::balance::Balance;

pub mod balance;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Duplicate transaction id: {0}")]
    DuplicateTransaction(u32),

    #[error("Missing transaction id: {0}")]
    MissingTransaction(u32),

    #[error("Client {0} is locked")]
    AccountLocked(Client),

    #[error("Insufficient funds")]
    InsufficientFunds,
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
    /// Get a reference to the transaction id for this user
    pub fn tx(&self) -> &Tx {
        match self {
            Transaction::Deposit { tx, .. } => tx,
            Transaction::Withdrawal { tx, .. } => tx,
            Transaction::Dispute { tx, .. } => tx,
            Transaction::Resolve { tx, .. } => tx,
            Transaction::ChargeBack { tx, .. } => tx,
        }
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

#[derive(Debug, Clone, Copy, Default)]
enum TxStatus {
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

#[derive(Debug, Clone, Copy)]
struct Entry {
    /// The transaction for this entry
    t: Transaction,

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
}

/// Each user will have a ledger of transactions. This will aim at being compact
/// But perhaps expensive at large numbers of transactions for now
pub struct Ledger {
    /// Mapping of a transaction by id to the index it's written into memory
    client_tx_to_idx: HashMap<(Client, Tx), usize>,

    /// Balances for each client
    balance: HashMap<Client, Balance>,

    /// All transactions within this ledger
    transactions: Vec<Transaction>,
}

impl Ledger {
    fn new() -> Self {
        Ledger {
            client_tx_to_idx: HashMap::new(),
            balance: HashMap::new(),
            transactions: Vec::new(),
        }
    }
}
