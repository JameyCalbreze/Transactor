# Transactor

This is a small personal project to both experiment within the Rust language and take time to consider the nuance of accounting rules. This repo reads in transactions from a CSV file and keeps track of the running balances in memory. Each row of the CSV is processed as needed through the use of a [`BufReader`] and fed into the [`csv`] crate for deserialization.

## Transaction State Machine

There is a simple state machine for each transaction. Each transaction starts in the `Active` state, and for simplicity, is considered to be complete. Every transaction is considered to be disputable at most 1 one time. This transition brings the transaction to the `Disputed` state. From the `Disputed` state the transaction may be `Resolved` which is effectively equivalent to the `Active` state, but can not be disputed again. The `ChargedBack` state is a reversal of the transaction and results in the halting of all future transactions on the account.

## Assumptions / Learnings

It was assumed that withdrawals and deposits were both disputable transactions. As both transactions immediately modify the total and available balances of the account it's important to consider the real number of dollars which are held in place during a dispute. The safest play is to assume all dollars are in egress from the account, and hold accordingly. For deposits, this is money considered to be within the account. A charge back on a deposit means money would be in egress from the account, and the money is considered held. A charge back on a withdrawal means money would return to the account, I.E. there is no money present to hold. A charged back withdrawal is a beneficial outcome for the account provider, not the target institution.

## Running

The contents of the provided CSV are read into memory after being deserialized into individual transactions. The standard output presents the final state of each account which deposited at least 1 dollar.

Execute the following to run

```sh
cargo run -- ./example.csv
```


