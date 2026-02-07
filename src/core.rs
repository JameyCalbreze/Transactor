use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("Failed to deserialize a transaction {0}")]
    TransDeError(String),

    #[error("Unknown transaction type")]
    UnknownTransactionType(String),
}

#[derive(Clone, Copy, PartialEq)]
enum Transaction {
    Deposit { client: u16, tx: u32, amount: f64 },
    Withdrawal { client: u16, tx: u32, amount: f64 },
    Dispute { client: u16, tx: u32 },
    Resolve { client: u16, tx: u32 },
    Chargeback { client: u16, tx: u32 },
}

#[derive(Clone, Serialize, Deserialize)]
struct CsvTransaction {
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
                || Err(Error::TransDeError(format!("amount absent from deposit"))),
                |amount| Ok(Transaction::Deposit { client, tx, amount }),
            ),
            "withdrawal" => amount.map_or_else(
                || {
                    Err(Error::TransDeError(format!(
                        "amount absent from withdrawal"
                    )))
                },
                |amount| Ok(Transaction::Withdrawal { client, tx, amount }),
            ),
            "dispute" => Ok(Transaction::Dispute { client, tx }),
            "resolve" => Ok(Transaction::Resolve { client, tx }),
            "chargeback" => Ok(Transaction::Chargeback { client, tx }),
            _ => Err(Error::UnknownTransactionType(format!(
                "Unknown type: {}",
                t
            ))),
        }
    }
}

#[cfg(test)]
mod test {

    use anyhow::Result;

    use crate::utils::StringReader;

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
        let mut r = StringReader::from(EXAMPLE_CSV.to_string());

        Ok(())
    }
}
