#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error occured while parsing: {0}")]
    ParseError(#[from] ParseError),
    #[error("Error occured while processing: {0}")]
    ProcessError(#[from] ProcessError),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("{0}")]
    CsvError(#[from] csv::Error),
    #[error("Header must be [type, client, tx, amount].")]
    HeaderError,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Insufficient funds for withdrawal.")]
    InsufficientFunds,
    #[error("Transaction not found for dispute/resolve/chargeback.")]
    TransactionNotFound,
    #[error("Dispute must target a deposit transaction.")]
    InvalidDispute,
    #[error("Resolve must target a deposit transaction that has been disputed.")]
    InvalidResolve,
    #[error("Chargeback must target a deposit transaction that has been disputed.")]
    InvalidChargeback,
    #[error("Funds already withdrawn cannot be disputed.")]
    DisputeAfterWithdrawal,
}
