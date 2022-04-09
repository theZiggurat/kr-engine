use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
    #[serde(skip_deserializing)]
    pub status: TransactionStatus,
}

#[derive(Debug, Deserialize, Clone)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

#[derive(Debug, Clone)]
pub enum TransactionStatus {
    Nominal,
    Disputed,
    ChargedBack,
}

impl Default for TransactionStatus {
    fn default() -> Self {
        Self::Nominal
    }
}
