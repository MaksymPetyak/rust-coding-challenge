use crate::account::{BasicAccount, ClientAccount, ClientId};
use crate::transaction::{Transaction, TransactionType};
use std::collections::HashMap;

pub struct TransactionEngine {
    /// State of client accounts. Will create a new account if the mentioned client id
    /// isn't present.
    pub accounts: HashMap<ClientId, Box<dyn ClientAccount>>,
}

impl TransactionEngine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }
}

impl TransactionEngine {
    pub fn execute(&mut self, transaction: Transaction) {
        let account = self
            .accounts
            .entry(transaction.client_id)
            .or_insert_with(|| Box::new(BasicAccount::new(transaction.client_id)));

        match transaction.transaction_type {
            TransactionType::Deposit => {
                if let Some(amount) = transaction.amount {
                    account.deposit(transaction.transaction_id, amount)
                }
                // TODO: log bad transaction type if there is not amount for deposit/withdrawal
            }
            TransactionType::Withdrawal => {
                if let Some(amount) = transaction.amount {
                    account.withdraw(transaction.transaction_id, amount)
                }
            }
            TransactionType::Dispute => account.dispute(transaction.transaction_id),
            TransactionType::Resolve => account.resolve(transaction.transaction_id),
            TransactionType::Chargeback => account.chargeback(transaction.transaction_id),
        }
    }
}
