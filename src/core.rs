use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub(crate) mod string;

#[derive(Debug, Error)]
enum Error {
    #[error("Missing amount on a transaction {0}")]
    MissingAmount(String),

    #[error("Unknown transaction type: {0}")]
    UnknownTransactionType(String),
}

#[derive(Clone, Copy, PartialEq)]
enum Transaction {
    Deposit { client: u16, tx: u32, amount: f64 },
    Withdrawal { client: u16, tx: u32, amount: f64 },
    Dispute { client: u16, tx: u32 },
    Resolve { client: u16, tx: u32 },
    ChargeBack { client: u16, tx: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CsvTransaction {
    #[serde(rename = "type")]
    t: String,
    client: u16,
    tx: u32,
    // This field is not always present for all types
    amount: Option<f64>,
}

impl TryFrom<CsvTransaction> for Transaction {
    type Error = crate::core::Error;

    fn try_from(value: CsvTransaction) -> Result<Self, Error> {
        // Expand the value
        let CsvTransaction {
            t,
            client,
            tx,
            amount,
        } = value;

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

#[cfg(test)]
mod test {

    use std::io::BufReader;

    use anyhow::{Result, anyhow};
    use csv::{Reader, StringRecord};
    use serde::Deserialize;

    use crate::core::{CsvTransaction, string::StringReader};

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
        let reader = BufReader::new(StringReader::from(EXAMPLE_CSV.to_string()));

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
