use crate::engine::TransactionEngine;
use crate::transaction::Transaction;
use csv::{ReaderBuilder, Trim};

mod account;
mod engine;
mod transaction;

fn main() {
    // TODO: use clap for better CLI interface
    let path = std::env::args().nth(1).expect("No file path provided");

    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        // Require flexible since the "amount" field may sometimes be unspecified
        .flexible(true)
        .from_path(path)
        .expect("Failed to build file reader");

    let mut transaction_engine = TransactionEngine::new();

    for result in reader.deserialize() {
        let transaction: Transaction = result.expect("Failed to deserialize");
        transaction_engine.execute(transaction);
    }

    // TODO: Could move to a special writer object or use csv writer
    println!("client, available, held, total, locked");
    for account in transaction_engine.accounts.values() {
        println!(
            "{}",
            format!(
                "{}, {:.4}, {:.4}, {:.4}, {}",
                account.get_client_id(),
                account.get_available_funds(),
                account.get_held_funds(),
                account.get_total_funds(),
                account.is_locked(),
            ),
        )
    }
}
