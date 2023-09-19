use cosmwasm_std::{DepsMut, Uint128};
use cw20::Cw20Coin;

use crate::{ContractError, state::{BalanceInfo, BALANCES}};

pub fn create_accounts(
    deps: &mut DepsMut,
    accounts: &[Cw20Coin],
    denom: String,
) -> Result<Uint128, ContractError> {

    let mut total_supply = Uint128::zero();
    for account in accounts {
        let address = deps.api.addr_validate(&account.address)?;
        let balance = BalanceInfo {
            amount: account.amount,
            denom: denom.clone(),
        };
        BALANCES.save(deps.storage, &address, &balance)?;
        total_supply += account.amount;
    }

    Ok(total_supply)
}