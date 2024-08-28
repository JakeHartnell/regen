use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid input")]
    InvalidInput {},

    #[error("Insufficient bid price")]
    InsufficientBidPrice,

    #[error("Max fee exceeded")]
    MaxFeeExceeded,

    #[error("Insufficient sell order quantity")]
    InsufficientSellOrderQuantity,
    // Add any other custom errors you need
}
