use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Token is not registered")]
    TokenNotRegistered {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Application already exists")]
    ApplicationAlreadyExists {},
    #[error("Application does not exist")]
    ApplicationDoesNotExist {},
    #[error("Such bank id is not registered")]
    BankNotRegistered {},
    #[error("Bank already exists")]
    BankAlreadyExists {},
    #[error("Account does not exist")]
    AccountDoesNotExist {},
    #[error("Not enough balance")]
    NotEnoughBalance {},
    #[error("Transaction already exists")]
    TransactionAlreadyExists {},
    #[error("Transaction does not exist")]
    TransactionDoesNotExist {},
    #[error("Exchange rate already exists")]
    ExchangeRateAlreadyExists {},
    #[error("Exchange rate does not exist")]
    ExchangeRateDoesNotExist {},
}
