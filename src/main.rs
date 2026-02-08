use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter},
};

use ::csv::ReaderBuilder;

use crate::{
    csv::{CsvTransaction, write_balances_to_file},
    ledger::{Ledger, Transaction},
};

mod csv;
mod ledger;
mod string;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // The filename we're looking for is the first argument which isn't '--'
    let filename = {
        let mut i = 1; // Skip over the name of the binary
        while args[i] == "--" {
            i += 1;
        }
        args[i].clone()
    };

    let mut f = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .truncate(false)
        .open(filename)
        .expect("File should be available");

    // Track all transactions in this file.
    let mut ledger = Ledger::new();

    // Need to read the file in
    let reader = BufReader::new(&f);

    // Parse the contents with our CSV library
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let headers = csv_reader.headers().expect("headers to be present").clone();

    for record in csv_reader.records() {
        match record {
            Ok(r) => {
                let tx: CsvTransaction = match r.deserialize(Some(&headers)) {
                    Ok(tx) => tx,
                    Err(_) => continue,
                };

                let tx = match tx.try_into() {
                    Ok(tx) => tx,
                    Err(_) => continue,
                };

                let _ = ledger.process_transaction(tx);
            }
            Err(_) => (), // Do nothing on bad entries in the CSV
        }
    }

    let snapshots = ledger.get_client_snapshots();

    let writer = std::io::stdout();

    let _ = write_balances_to_file(&snapshots, writer);
}
