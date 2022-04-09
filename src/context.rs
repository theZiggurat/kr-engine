use crate::client::Client;
use crate::errors::*;
use crate::transaction::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context {
    batch_idx: usize,
    clients: HashMap<u16, Client>,
    transactions: Vec<Transaction>,
    transaction_cache: HashMap<u32, usize>,
}

impl Context {
    /// Builds context from csv file path.
    ///
    /// Example: "path/to/file.csv"
    ///
    pub fn from_csv(path: &std::path::Path) -> Result<Self, ParseError> {
        let mut context = Context::default();

        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_path(path)?;

        // validate headers of csv
        if reader
            .headers()?
            .into_iter()
            .map(|header| header.trim().to_lowercase())
            .zip(["type", "client", "tx", "amount"].into_iter())
            .map(|(actual, expected)| !actual.eq(expected))
            .any(|b| b)
        {
            return Err(ParseError::HeaderError);
        }

        let txns: Result<Vec<Transaction>, _> = reader.deserialize::<Transaction>().collect();
        context.transactions = txns?;

        Ok(context)
    }

    /// Processes transactions in chronological order.
    /// Starts at self.batch_index and finishes at end of transaction list.
    ///
    /// Returns tuple of # of successfully processes and # of failed processes.
    ///
    pub fn batch(&mut self) -> (i32, i32) {
        let mut successes = 0;
        let mut failures = 0;
        while self.batch_idx < self.transactions.len() {
            match self.process() {
                Ok(_) => successes += 1,
                Err(e) => {
                    failures += 1;
                    eprintln!("Error on transaction record #{}: {}", self.batch_idx, e);
                }
            }
            self.batch_idx += 1;
        }
        (successes, failures)
    }

    /// Writes contents of client table to writer
    /// in csv file format.
    ///
    pub fn write_as_csv<W>(&self, writer: W)
    where
        W: std::io::Write,
    {
        let mut writer = csv::Writer::from_writer(writer);
        let mut clients = self.clients.values().collect::<Vec<_>>();
        clients.sort_by(|a, b| a.id.cmp(&b.id));
        for client in clients {
            if let Err(e) = writer.serialize(client) {
                eprintln!("{}", e);
            }
        }
    }

    /// Obtain mutable reference to deposit transaction with id = tx.
    /// Returns None if no transaction with id = tx at the time of this function invocation.
    ///
    fn get_txn_mut(&mut self, tx: u32) -> Option<&mut Transaction> {
        self.transaction_cache
            .get(&tx)
            .map(|&idx| &mut self.transactions[idx])
    }

    /// Processes transaction entry at batch index.
    ///
    /// **NOTE**: Private function only called from batch to ensure chronological ordering.
    fn process(&mut self) -> Result<(), ProcessError> {
        // retrieve current transaction
        let transaction = &self.transactions[self.batch_idx];
        let tx = transaction.tx;

        // retrieve target client state before transaction
        // create new client if one does not exist at txn.client
        let client = match self.clients.get(&transaction.client) {
            Some(client) => client.clone(),
            None => Client::new(transaction.client),
        };

        // transform client immutably using transaction
        let client = match transaction.tx_type {
            TransactionType::Deposit => {
                self.transaction_cache
                    .insert(transaction.tx, self.batch_idx);
                let amount = transaction.amount.unwrap_or(0.);
                Client {
                    available: client.available + amount,
                    total: client.total + amount,
                    ..client
                }
            }

            TransactionType::Withdrawal => {
                let amount = transaction.amount.unwrap_or(0.);
                if client.available < amount {
                    return Err(ProcessError::InsufficientFunds);
                } else {
                    Client {
                        available: client.available - amount,
                        total: client.total - amount,
                        ..client
                    }
                }
            }

            TransactionType::Dispute => {
                if let Some(txn) = self.get_txn_mut(tx) {
                    match (&txn.tx_type, &txn.status) {
                        (TransactionType::Deposit, TransactionStatus::Nominal) => {
                            let amount = txn.amount.unwrap_or(0.);
                            if client.available < amount {
                                return Err(ProcessError::DisputeAfterWithdrawal);
                            }
                            txn.status = TransactionStatus::Disputed;
                            Client {
                                available: client.available - amount,
                                held: client.held + amount,
                                ..client
                            }
                        }
                        _ => return Err(ProcessError::InvalidDispute),
                    }
                } else {
                    return Err(ProcessError::TransactionNotFound);
                }
            }

            TransactionType::Resolve => {
                if let Some(txn) = self.get_txn_mut(tx) {
                    match (&txn.tx_type, &txn.status) {
                        (TransactionType::Deposit, TransactionStatus::Disputed) => {
                            let amount = txn.amount.unwrap_or(0.);
                            txn.status = TransactionStatus::Nominal;
                            Client {
                                available: client.available + amount,
                                held: client.held - amount,
                                ..client
                            }
                        }
                        _ => return Err(ProcessError::InvalidResolve),
                    }
                } else {
                    return Err(ProcessError::TransactionNotFound);
                }
            }

            TransactionType::Chargeback => {
                if let Some(txn) = self.get_txn_mut(tx) {
                    match (&txn.tx_type, &txn.status) {
                        (TransactionType::Deposit, TransactionStatus::Disputed) => {
                            let amount = txn.amount.unwrap_or(0.);
                            txn.status = TransactionStatus::ChargedBack;
                            Client {
                                held: client.held - amount,
                                total: client.total - amount,
                                locked: true,
                                ..client
                            }
                        }
                        _ => return Err(ProcessError::InvalidChargeback),
                    }
                } else {
                    return Err(ProcessError::TransactionNotFound);
                }
            }
        };

        // apply transformation
        self.clients.insert(client.id, client);

        Ok(())
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            batch_idx: 0,
            clients: HashMap::new(),
            transactions: Vec::new(),
            transaction_cache: HashMap::new(),
        }
    }
}
