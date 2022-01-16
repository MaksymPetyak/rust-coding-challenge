use crate::transaction::TransactionId;
use std::collections::HashMap;

pub type ClientId = u16;

/// Trait defining available operations on client account
/// TODO: make operation methods return Result<(), UpdateError> in case something goes wrong
pub trait ClientAccount {
    fn deposit(&mut self, transaction_id: TransactionId, amount: f64);

    /// Does nothing if there are not enough available funds
    fn withdraw(&mut self, transaction_id: TransactionId, amount: f64);

    fn dispute(&mut self, transaction_id: TransactionId);

    fn resolve(&mut self, transaction_id: TransactionId);

    fn chargeback(&mut self, transaction_id: TransactionId);

    fn get_client_id(&self) -> ClientId;

    /// Total funds are available + held funds held by the client
    fn get_total_funds(&self) -> f64;

    fn get_available_funds(&self) -> f64;

    fn get_held_funds(&self) -> f64;

    fn is_locked(&self) -> bool;
}

#[derive(Debug)]
pub struct BasicAccount {
    client_id: ClientId,
    // TODO: switch to working with Decimal
    available: f64,
    held: f64,
    locked: bool,

    /// Keeps the amount by which the available funds have changed (-amount in withdrawals) in a
    /// transaction.
    /// Used to handle dispute transactions rather than to keep history of all transactions
    transaction_log: HashMap<TransactionId, f64>,
    /// Keeps the active disputes with the respective amount under dispute until it's resolved or
    /// chargebacked
    active_disputes: HashMap<TransactionId, f64>,
}

impl BasicAccount {
    pub fn new(client_id: ClientId) -> Self {
        BasicAccount {
            client_id,
            available: 0.0,
            held: 0.0,
            locked: false,

            transaction_log: HashMap::new(),
            active_disputes: HashMap::new(),
        }
    }
}

impl ClientAccount for BasicAccount {
    fn deposit(&mut self, transaction_id: TransactionId, amount: f64) {
        self.available += amount;
        self.transaction_log.insert(transaction_id, amount);
    }

    /// Does nothing if there are not enough available funds
    fn withdraw(&mut self, transaction_id: TransactionId, amount: f64) {
        if self.available >= amount {
            self.available -= amount;
            // It's actually a bit unclear to me how disputing a withdrawal would work.
            // Imagining an ATM, when the account holder withdraws the funds you can't really put
            // those funds on hold anymore.
            // I will assume that what we aim for is an ability to reverse a transaction in dispute
            // so here we store the amount by which the available funds decreased, but this also
            // means that when you put this transaction on dispute the held funds can be
            // negative, which might not make sense
            self.transaction_log.insert(transaction_id, -amount);
        }
    }

    fn dispute(&mut self, transaction_id: TransactionId) {
        // remove transaction from the log so that it cannot be disputed twice
        if let Some(amount) = self.transaction_log.remove(&transaction_id) {
            self.active_disputes.insert(transaction_id, amount);
            self.available -= amount;
            self.held += amount;
        }
    }

    fn resolve(&mut self, transaction_id: TransactionId) {
        // remove transaction from disputes so that it cannot be resolved twice
        if let Some(amount) = self.active_disputes.remove(&transaction_id) {
            self.held -= amount;
            self.available += amount;
        }
    }

    fn chargeback(&mut self, transaction_id: TransactionId) {
        // remove transaction from disputes so that it cannot be chargebacked twice
        if let Some(amount) = self.active_disputes.remove(&transaction_id) {
            self.held -= amount;
            self.locked = true;
        }
    }

    fn get_client_id(&self) -> ClientId {
        self.client_id
    }

    fn get_total_funds(&self) -> f64 {
        self.available + self.held
    }

    fn get_available_funds(&self) -> f64 {
        self.available
    }

    fn get_held_funds(&self) -> f64 {
        self.held
    }

    fn is_locked(&self) -> bool {
        self.locked
    }
}

#[cfg(test)]
mod tests {
    mod unit {
        use crate::account::{BasicAccount, ClientAccount};

        fn approx_eq(a: f64, b: f64) -> bool {
            (a - b).abs() < f64::EPSILON
        }

        #[test]
        fn deposit_and_withdraw_works() {
            let mut account = BasicAccount::new(0);

            account.deposit(0, 2.0);
            account.withdraw(1, 1.0);

            assert!(approx_eq(account.get_available_funds(), 1.0));
        }

        #[test]
        fn dispute_increases_held_funds() {
            let mut account = BasicAccount::new(0);

            account.deposit(0, 2.0);
            account.dispute(0);

            assert!(approx_eq(account.get_available_funds(), 0.0));
            assert!(approx_eq(account.get_held_funds(), 2.0));
        }

        #[test]
        fn resolving_dispute_brings_back_available_funds() {
            let mut account = BasicAccount::new(0);

            account.deposit(0, 2.0);
            account.dispute(0);
            account.resolve(0);

            assert!(approx_eq(account.get_available_funds(), 2.0));
            assert!(approx_eq(account.get_held_funds(), 0.0));
        }

        #[test]
        fn chargeback_removes_funds_and_locks_account() {
            let mut account = BasicAccount::new(0);

            account.deposit(0, 2.0);
            account.dispute(0);
            account.chargeback(0);

            assert!(approx_eq(account.get_available_funds(), 0.0));
            assert!(approx_eq(account.get_held_funds(), 0.0));
            assert!(account.is_locked());
        }

        #[test]
        fn withdrawing_with_not_enough_funds_has_no_effect() {
            let mut account = BasicAccount::new(0);

            account.deposit(0, 2.0);
            account.withdraw(1, 3.0);

            // Also check that disputing and resolving withdraw transaction does nothing
            account.dispute(1);
            account.resolve(1);

            assert!(approx_eq(account.get_available_funds(), 2.0));
        }

        #[test]
        fn disputing_withdrawal_and_resolving_withdrawal_works() {
            let mut account = BasicAccount::new(0);

            account.deposit(0, 5.0);
            account.withdraw(1, 3.0);

            // Also check that disputing and resolving withdraw transaction does nothing
            account.dispute(1);
            assert!(approx_eq(account.get_available_funds(), 5.0));
            assert!(approx_eq(account.get_held_funds(), -3.0));

            account.resolve(1);
            assert!(approx_eq(account.get_available_funds(), 2.0));
            assert!(approx_eq(account.get_held_funds(), 0.0));
        }

        // TODO: How to handle the case when you deposit, withdraw, and then dispute deposit. Could
        // bring available funds to a negative value

        #[test]
        fn transaction_cannot_be_disputed_twice() {
            let mut account = BasicAccount::new(0);
            let deposit_amount = 2.0;

            account.deposit(0, deposit_amount);

            account.dispute(0);
            account.dispute(0);
            assert!(approx_eq(account.get_held_funds(), deposit_amount));
            assert!(approx_eq(account.get_available_funds(), 0.0));

            account.resolve(0);
            assert!(approx_eq(account.get_available_funds(), deposit_amount));
            assert!(approx_eq(account.get_held_funds(), 0.0));

            account.chargeback(0);
            assert!(approx_eq(account.get_available_funds(), deposit_amount));
            assert!(approx_eq(account.get_held_funds(), 0.0));
        }
    }
}
