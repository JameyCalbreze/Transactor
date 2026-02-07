use std::collections::HashMap;

use crate::csv::CsvTransaction;

/// Each of the individual operations which we may process
#[derive(Clone, Copy, PartialEq)]
pub enum Transaction {
    Deposit { client: u16, tx: u32, amount: f64 },
    Withdrawal { client: u16, tx: u32, amount: f64 },
    Dispute { client: u16, tx: u32 },
    Resolve { client: u16, tx: u32 },
    ChargeBack { client: u16, tx: u32 },
}

/// Each user will have a ledger of transactions. This will aim at being compact
/// But perhaps expensive at large numbers of transactions for now
pub struct Ledger {
    /// Mapping of a transaction by id to the index it's written into memory
    tx_to_i: HashMap<u32, usize>,

    /// All transactions within this ledger
    transactions: Vec<Transaction>,
}


