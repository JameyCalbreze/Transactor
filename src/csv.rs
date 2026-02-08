use std::{
    fmt::Display,
    io::{self, Write},
};

use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ledger::{Transaction, balance::BalanceSnapshot};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Missing amount on a transaction {0}")]
    MissingAmount(String),

    #[error("Unknown transaction type: {0}")]
    UnknownTransactionType(String),

    #[error("I/O Error: {0}")]
    IOError(#[from] io::Error),

    #[error("CSV Serialization Error: {0}")]
    CSVError(#[from] csv::Error),
}

/// The struct we'll read out of our input file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvTransaction {
    #[serde(rename = "type")]
    pub t: String,
    pub client: u16,
    pub tx: u32,
    // This field is not always present for all types
    pub amount: Option<f64>,
}

impl TryInto<Transaction> for CsvTransaction {
    type Error = crate::csv::Error;

    fn try_into(self) -> Result<Transaction, Error> {
        // Expand the value
        let CsvTransaction {
            t,
            client,
            tx,
            amount,
        } = self;

        match t.to_lowercase().as_str() {
            "deposit" => amount.map_or_else(
                || Err(Error::MissingAmount(format!("amount absent from deposit"))),
                |amount| Ok(Transaction::Deposit { client, tx, amount }),
            ),
            "withdrawal" => amount.map_or_else(
                || {
                    Err(Error::MissingAmount(format!(
                        "amount absent from withdrawal"
                    )))
                },
                |amount| Ok(Transaction::Withdrawal { client, tx, amount }),
            ),
            "dispute" => Ok(Transaction::Dispute { client, tx }),
            "resolve" => Ok(Transaction::Resolve { client, tx }),
            "chargeback" => Ok(Transaction::ChargeBack { client, tx }),
            _ => Err(Error::UnknownTransactionType(format!(
                "Unknown type: {}",
                t
            ))),
        }
    }
}

impl Display for CsvTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("CsvTransaction");
        s.field("t", &self.t)
            .field("client", &self.client)
            .field("tx", &self.tx);

        self.amount.map(|amount| s.field("amount", &amount));
        s.finish()
    }
}

/// Final output to standard out
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct CsvBalance {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl From<&BalanceSnapshot> for CsvBalance {
    fn from(value: &BalanceSnapshot) -> Self {
        Self {
            client: value.client,
            available: value.available,
            held: value.held,
            total: value.total,
            locked: value.locked,
        }
    }
}

/// Give a slice of snapshots of the client balances
pub fn write_balances_to_file(
    balances: &[BalanceSnapshot],
    writer: impl Write,
) -> Result<(), Error> {
    let mut csv_writer = WriterBuilder::new().from_writer(writer);

    for snapshot in balances {
        let csv_balance = CsvBalance::from(snapshot);
        csv_writer.serialize(csv_balance)?;
    }

    // Flush the contents of the writer
    csv_writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod test {

    use std::io::BufReader;

    use anyhow::{Result, anyhow};
    use csv::Reader;

    use crate::{csv::CsvTransaction, string::StringReader};

    static EXAMPLE_CSV: &str = r#"
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
"#;

    #[test]
    fn deserialize_example() -> Result<()> {
        let reader = BufReader::new(StringReader::from(EXAMPLE_CSV));

        let mut csv_reader = Reader::from_reader(reader);

        // Trim the headers as they may be formatted with white space
        let mut headers = csv_reader.headers()?.clone();
        headers.trim();

        let mut records = Vec::new();

        for r in csv_reader.into_records() {
            match r {
                Ok(mut record) => {
                    record.trim();
                    records.push(record)
                }
                Err(e) => Err(anyhow!("Kaboom! Failed to parse: {}", e))?,
            }
        }

        // Read out the headers to know which column is which - Optional as we don't need to clone to deserialize with
        print!("Headers:");
        for header in headers.iter() {
            print!(" \"{}\"", header);
        }
        println!("");

        for i in 1..records.len() {
            let csv_transaction: CsvTransaction =
                records.get(i).unwrap().deserialize(Some(&headers))?;
            println!("Deserialized transaction: {}", csv_transaction);
        }

        Ok(())
    }
}
