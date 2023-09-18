extern crate serde;

use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20Coin;
use cw_storage_plus::Map;

#[cw_serde]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub cw20coin: Cw20Coin,
}

#[cw_serde]
pub struct ApplicationInfo {
    pub id: String,
    pub from: Addr,
    pub to: Addr,
    pub amount: Uint128,
    pub denom: String,
}

#[cw_serde]
pub struct BankInfo {
    pub id: String,
    pub name: String,
    pub balance: Uint128,
}

#[cw_serde]
pub enum TransactionStatus {
    Initial,
    SentToBank,
    SentToRecipient,
    RejectedByBank,
}

// Float rate = rate / 10^precision
#[cw_serde]
pub struct ExchangeRateInfo {
    pub id: String,
    pub denom_from: String,
    pub denom_to: String,
    pub precision: u32,
    pub rate: u64,
}


#[cw_serde]
pub struct TransactionInfo {
    pub id: String,
    pub bank_id: String,
    pub from: Addr,
    pub to: Addr,
    pub amount: Uint128,
    pub denom_from: String,
    pub denom_to: String,
    pub status: TransactionStatus,
}

impl TransactionInfo {
    pub fn update_status(&mut self, status: TransactionStatus) -> TransactionInfo {
        self.status = status;
        self.clone()
    }
}

impl BankInfo {
    pub fn income(&mut self, amount: Uint128) -> BankInfo {
        self.balance += amount;
        self.clone()
    }

    pub fn outcome(&mut self, amount: Uint128) -> BankInfo {
        self.balance -= amount;
        self.clone()
    }
}


pub const TOKENS: Map<String, TokenInfo> = Map::new("tokens");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
pub const APPLICATIONS: Map<String, ApplicationInfo> = Map::new("applications");
pub const BANKS: Map<String, BankInfo> = Map::new("banks");
pub const TRANSACTIONS: Map<String, TransactionInfo> = Map::new("transactions");
pub const EXCHANGE_RATES: Map<String, ExchangeRateInfo> = Map::new("exchange_rates");
