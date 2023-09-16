use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::{Cw20Coin, TokenInfoResponse, BalanceResponse};

#[cw_serde]
#[cfg_attr(test, derive(Default))]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Register { id: String, amount: u128, denom: String },
    Apply { id: String },
    Reject { id: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(BalanceResponse)]
    Balance { address: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(TokenInfoResponse)]
    TokenInfo {},
}
