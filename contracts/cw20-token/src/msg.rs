use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::{Cw20Coin, BalanceResponse};

use crate::state::{ApplicationInfo, BankInfo, TransactionInfo, TransactionStatus, ExchangeRateInfo, TokenInfo};

#[cw_serde]
#[cfg_attr(test, derive(Default))]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    // // Register Application
    // Register(ApplicationInfo),
    // // Apply an application (send to recipient)
    // Apply { id: String },
    // // Cancel the application
    // Reject { id: String },
    // Create token
    CreateToken(TokenInfo),
    // CreateBank
    CreateBank(BankInfo),
    // Send Transaction to Bank
    SendToBank(TransactionInfo),
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
    TokenInfo {symbol: String},
    // Returns ApplicationInfo
    #[returns(ApplicationInfoResponse)]
    ApplicationInfo { id: String },
    // Returns list of ApplicationInfo
    #[returns(ApplicationListResponse)]
    ApplicationList {},
    #[returns(BankInfoResponse)]
    BankInfo { id: String },
    #[returns(TransactionInfoResponse)]
    TransactionInfo { id: String },
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
    pub symbol: String,
    pub decimals: u8,
    pub cw20coin: Cw20Coin,
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
    pub rate: f64,
}

#[cw_serde]
pub struct BankInfoResponse {
    pub id: String,
    pub name: String,
    pub balance: Uint128,
}

#[cw_serde]
pub struct ApplicationInfoResponse {
    pub id: String,
    pub from: Addr,
    pub to: Addr,
    pub amount: Uint128,
    pub denom: String,
}

#[cw_serde]
pub struct ApplicationListResponse {
    pub applications: Vec<ApplicationInfo>,
}
