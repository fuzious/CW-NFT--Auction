use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Insufficient funds sent")]
    InsufficientFundsSend {},

    #[error("Auction Ended")]
    AuctionEnded {},

    #[error("Auction Not Ended Yet")]
    AuctionNotEnded {},
}
