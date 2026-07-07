use alloy::transports::{RpcError, TransportErrorKind};
use thiserror::Error;

use crate::{
    postgres::PostgresError, provider::SendTransactionError, transaction::types::TransactionHash,
    SafeProxyError,
};

#[derive(Error, Debug)]
pub enum SendTransactionGasPriceError {
    #[error("Gas calculation error")]
    GasCalculationError,

    #[error("Blob gas calculation error")]
    BlobGasCalculationError,

    #[error("Transaction has no last sent gas price object")]
    NoLastSentGas,
}

#[derive(Error, Debug)]
pub enum TransactionQueueSendTransactionError {
    #[error("Gas price too high")]
    GasPriceTooHigh,

    #[error("Gas calculation error")]
    GasCalculationError,

    #[error("Transaction send error: {0}")]
    TransactionSendError(#[from] SendTransactionError),

    /// The broadcast failed with a nonce error but it could not be verified
    /// whether one of our own earlier broadcasts consumed the nonce, for example
    /// because the receipt lookup itself failed.
    ///
    /// The send must be retried later at the same nonce. Handlers must never
    /// route this into nonce recovery, re-assigning a new nonce to a payload
    /// that may already be on-chain executes it twice.
    #[error("Broadcast outcome unknown for transaction hash {hash}, will retry: {reason}")]
    BroadcastInconclusive { hash: TransactionHash, reason: String },

    #[error("Transaction could not be updated in DB: {0}")]
    CouldNotUpdateTransactionDb(#[from] PostgresError),

    #[error("{0}")]
    SendTransactionGasPriceError(#[from] SendTransactionGasPriceError),

    #[error("Transaction estimate gas error: {0}")]
    TransactionEstimateGasError(RpcError<TransportErrorKind>),

    #[error("Transaction conversion error: {0}")]
    TransactionConversionError(String),

    #[error("Safe proxy error: {0}")]
    SafeProxyError(#[from] SafeProxyError),

    #[error("No transaction found in queue")]
    NoTransactionInQueue,
}
