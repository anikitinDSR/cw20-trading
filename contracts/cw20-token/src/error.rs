use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Token is not registered")]
    TokenNotRegistered { denom: String},
    #[error("Token is already registered")]
    TokenAlreadyRegistered { denom: String},
    #[error("Such bank id is not registered")]
    BankNotRegistered {id: String },
    #[error("Bank already exists")]
    BankAlreadyExists { id: String },
    #[error("Account does not exist")]
    AccountDoesNotExist { account: String },
    #[error("Account already has an assigned token")]
    TokenAlreadyAssigned { denom: String },
    #[error("Not enough balance")]
    NotEnoughBalance { required: Uint128, available: Uint128 },
    #[error("Transaction already exists")]
    TransactionAlreadyExists { id: String },
    #[error("Transaction does not exist")]
    TransactionDoesNotExist { id: String },
    #[error("Exchange rate does not exist")]
    ExchangeRateDoesNotExist { id: String },
}
