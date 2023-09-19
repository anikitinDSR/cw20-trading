use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20Coin;

use crate::state::{ BankInfo, TransactionStatus};

#[cw_serde]
#[cfg_attr(test, derive(Default))]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    // Create token
    CreateToken(TokenInfoMsg),
    // CreateBank
    CreateBank(BankInfo),
    // Send Transaction to Bank
    SendToBank(TransactionMsg),
    // Send Transaction to Recipient
    SendToRecipient { transaction_id: String},
    // Set exchange rate
    SetExchangeRate(ExchangeRateMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(BalanceResponse)]
    Balance { address: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(TokenInfoResponse)]
    TokenInfo { denom: String},
    #[returns(ExchangeRateInfoResponse)]
    ExchangeRateInfo { id: String },
    #[returns(BankInfoResponse)]
    BankInfo { id: String },
    #[returns(TransactionInfoResponse)]
    TransactionInfo { id: String },
}

#[cw_serde]
pub struct BalanceResponse {
    pub amount: Uint128,
    pub denom: String
}

#[cw_serde]
pub struct TokenInfoMsg {
    // PK
    pub denom: String,
    pub name: String,
    pub initial_balances: Vec<Cw20Coin>
}

#[cw_serde]
pub struct ExchangeRateMsg {
    pub denom_from: String,
    pub denom_to: String,
    pub precision: u32,
    pub rate: u64,
}

#[cw_serde]
pub struct TokenInfoResponse {
    pub name: String,
    pub denom: String,
    pub total_supply: Uint128,
    pub initial_balances: Vec<Cw20Coin>,
}

#[cw_serde]
pub struct TransactionMsg {
    pub id: String,
    pub bank_id: String,
    pub from: Addr,
    pub to: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub struct TransactionInfoResponse {
    pub id: String,
    pub bank_id: String,
    pub from: Addr,
    pub to: Addr,
    pub amount: Uint128,
    pub denom_from: String,
    pub denom_to: String,
    pub status: TransactionStatus,
}

#[cw_serde]
pub struct ExchangeRateInfoResponse {
    pub id: String,
    pub denom_from: String,
    pub denom_to: String,
    pub rate: u64,
    pub precision: u32,
}

#[cw_serde]
pub struct BankInfoResponse {
    pub id: String,
    pub name: String,
    pub balance: Uint128,
}
