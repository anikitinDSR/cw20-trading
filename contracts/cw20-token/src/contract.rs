#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, to_binary, Addr};
use cw2::set_contract_version;
use cw20::{Cw20Coin, BalanceResponse};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ApplicationListResponse, TokenInfoResponse};
use crate::state::{TOKENS, TokenInfo, BALANCES, APPLICATIONS, ApplicationInfo, BANKS, TRANSACTIONS, TransactionStatus, ExchangeRateInfo, EXCHANGE_RATES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateToken(token_info) => execute::execute_create_token(deps, token_info),
        ExecuteMsg::CreateBank(bank_info) => execute::execute_create_bank(deps, bank_info),

        ExecuteMsg::SetExchangeRate(exchange_rate) => execute::execute_set_exchange_rate(deps, exchange_rate),

        ExecuteMsg::SendToBank(transaction_info) => execute::execute_send_to_bank(deps, transaction_info),
        ExecuteMsg::SendToRecipient { transaction_id } => execute::execute_send_to_recipient(deps, transaction_id),
}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Balance { address } => query::query_balance(deps, address),
        QueryMsg::TokenInfo {symbol} => query::query_token_info(deps, symbol),
        QueryMsg::ApplicationInfo { id } => query::query_application_info(deps, id),
        QueryMsg::ApplicationList {} => query::query_application_list(deps),
        QueryMsg::BankInfo { id } => query::query_bank_info(deps, id),
        QueryMsg::TransactionInfo { id } => query::query_transaction_info(deps, id),
    }
}

pub mod execute {

    use crate::{state::{TransactionInfo, BankInfo}, msg::ExchangeRateMsg};

    use super::*;

    pub fn execute_create_token(deps: DepsMut, token_info: TokenInfo) -> Result<Response, ContractError> {
        let symbol = token_info.symbol.clone();
        if TOKENS.has(deps.storage, symbol.clone()) {
            return Err(ContractError::TokenNotRegistered {});
        }
        if BALANCES.has(deps.storage, &Addr::unchecked(token_info.cw20coin.address.as_str())) {
            return Err(ContractError::AccountDoesNotExist {});
        }
        TOKENS.save(deps.storage, symbol,  &token_info)?;
        create_accounts(deps, &token_info.cw20coin)?;
        
        Ok(Response::default())
    }

    pub fn execute_create_bank(deps: DepsMut, bank_info: BankInfo) -> Result<Response, ContractError> {
        let bank_id = bank_info.id.clone();
        if BANKS.has(deps.storage, bank_id.clone()) {
            return Err(ContractError::BankAlreadyExists {});
        }
        BANKS.save(deps.storage, bank_id,  &bank_info)?;
        Ok(Response::default())
    }

    pub fn execute_set_exchange_rate(deps: DepsMut, exchange_rate: ExchangeRateMsg) -> Result<Response, ContractError> {
        let exchange_rate_id = exchange_rate.denom_from.to_owned() + exchange_rate.denom_to.as_str();
        let exchange_rate_id_verse = exchange_rate.denom_to.to_owned() + exchange_rate.denom_from.as_str();

        if EXCHANGE_RATES.has(deps.storage, exchange_rate_id.clone()) {
            return Err(ContractError::ExchangeRateAlreadyExists {});
        }
        // We cannot store f64 in the state
        let base = 10_u128.pow(exchange_rate.precision) as f64;
        let coeff: f64 = exchange_rate.rate as f64 / base;
        let verse_rate = base / coeff;

        let exchange_rate_state = ExchangeRateInfo {
            id: exchange_rate_id_verse.clone(),
            denom_from: exchange_rate.denom_to.clone(),
            denom_to: exchange_rate.denom_from.clone(),
            precision: exchange_rate.precision,
            rate: exchange_rate.rate,
        };

        let exchange_rate_verse_state = ExchangeRateInfo {
            id: exchange_rate_id_verse.clone(),
            denom_from: exchange_rate.denom_to.clone(),
            denom_to: exchange_rate.denom_from.clone(),
            precision: exchange_rate.precision,
            rate: verse_rate as u64,
        };
        EXCHANGE_RATES.save(deps.storage, exchange_rate_id,  &exchange_rate_state)?;
        EXCHANGE_RATES.save(deps.storage, exchange_rate_id_verse,  &exchange_rate_verse_state)?;
        Ok(Response::default())
    }

    pub fn execute_send_to_bank(deps: DepsMut, transaction_info: TransactionInfo) -> Result<Response, ContractError> {
        let bank_id = transaction_info.bank_id.clone();
        let transaction_id = transaction_info.id.clone();
        // Check if bank exists
        if !BANKS.has(deps.storage, bank_id.clone()) {
            return Err(ContractError::BankNotRegistered {});
        }
        // Check if transaction exists
        if TRANSACTIONS.has(deps.storage, transaction_id.clone()) {
            return Err(ContractError::TransactionAlreadyExists {});
        }
        // Check if sender has enough balance
        let balance = BALANCES.load(deps.storage, &transaction_info.from)?;
        if balance < transaction_info.amount {
            return Err(ContractError::NotEnoughBalance {});
        }
        // decrease sender balance
        BALANCES.update(deps.storage, &transaction_info.from, |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default() - transaction_info.amount)
        })?;

        // increase bank balance
        BANKS.update(deps.storage, bank_id, |bank| -> StdResult<_> {
            Ok(bank.unwrap().income(transaction_info.amount))
        })?;

        let mut transaction = transaction_info;
        // Update transaction status
        TRANSACTIONS.save(deps.storage, transaction_id, &transaction.update_status(TransactionStatus::SentToBank))?;
        Ok(Response::default())
    }

    pub fn execute_send_to_recipient(deps: DepsMut, transaction_id: String) -> Result<Response, ContractError> {
        // Check if transaction exists

        if !TRANSACTIONS.has(deps.storage, transaction_id.clone()) {
            return Err(ContractError::TransactionDoesNotExist {});
        }

        let transaction_info = TRANSACTIONS.load(deps.storage, transaction_id.clone())?;

        // Check that exchange rate exists
        let exchange_rate_id = transaction_info.denom_from.to_owned() + transaction_info.denom_to.as_str();

        if !EXCHANGE_RATES.has(deps.storage, exchange_rate_id.clone()) {
            return Err(ContractError::ExchangeRateDoesNotExist {});
        }

        // Check if bank exists
        let bank_id = transaction_info.bank_id.clone();
        if !BANKS.has(deps.storage, bank_id.clone()) {
            return Err(ContractError::BankNotRegistered {});
        }

        let exchange_rate = EXCHANGE_RATES.load(deps.storage, exchange_rate_id)?; // TODO: check if it is correct

        let bank = BANKS.load(deps.storage, bank_id.clone())?; // TODO: check if it is correct

        if bank.balance < transaction_info.amount {
            return Err(ContractError::NotEnoughBalance {});
        }
        
        // decrease bank balance
        BANKS.update(deps.storage, bank_id, |bank| -> StdResult<_> {
            Ok(bank.unwrap().outcome(transaction_info.amount))
        })?;

        // Calculate balance due to exchange rate
        let coeff = exchange_rate.rate as f64 / 10_u128.pow(exchange_rate.precision) as f64;
        let amount = (transaction_info.amount.u128() as f64 * coeff) as u128;

        BALANCES.update(deps.storage, &transaction_info.to, |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + Uint128::from(amount))
        })?;

        let mut transaction = transaction_info;
        // Update transaction status
        TRANSACTIONS.save(deps.storage, transaction_id, &transaction.update_status(TransactionStatus::SentToRecipient))?;
        Ok(Response::default())
    }
}
pub mod query {
    use crate::msg::{ApplicationInfoResponse, BankInfoResponse, TransactionInfoResponse};

    use super::*;

    pub fn query_balance(deps: Deps, address: String) -> Result<Binary, ContractError> {
        let address = deps.api.addr_validate(&address)?;
        let balance = BALANCES
            .may_load(deps.storage, &address)?
            .unwrap_or_default();
        Ok(to_binary(&BalanceResponse { balance })?)
    }

    pub fn query_token_info(deps: Deps, symbol: String) -> Result<Binary, ContractError> {
        match TOKENS.load(deps.storage, symbol) {
            Ok(info) => {
                let res = TokenInfoResponse {
                    name: info.name,
                    symbol: info.symbol,
                    decimals: info.decimals,
                    cw20coin: info.cw20coin,
                };
                Ok(to_binary(&res)?)
            },
            Err(_) => return Err(ContractError::TokenNotRegistered {}),
        }
    }

    pub fn query_application_info(deps: Deps, id: String) -> Result<Binary, ContractError> {
        match APPLICATIONS.load(deps.storage, id) {
            Ok(item) => {
                let res = ApplicationInfoResponse {
                id: item.id,
                from: item.from,
                to: item.to,
                amount: item.amount,
                denom: item.denom,
            };
            Ok(to_binary(&res)?)},
            Err(_) => return Err(ContractError::ApplicationDoesNotExist {}), 
        }
        
    }

    pub fn query_application_list(deps: Deps) -> Result<Binary, ContractError> {
        let list: Vec<ApplicationInfo> = APPLICATIONS
                .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
                .map(|item| {
                    let (_, v) = item?;
                    Ok(v)
                })
                .collect::<StdResult<Vec<ApplicationInfo>>>()?;
        let res = ApplicationListResponse { applications: list };
        Ok(to_binary(&res)?)
    }

    pub fn query_bank_info(deps: Deps, id: String) -> Result<Binary, ContractError> {
        match BANKS.load(deps.storage, id) {
            Ok(item) => {
                let res = BankInfoResponse {
                    id: item.id,
                    name: item.name,
                    balance: item.balance,
                };
                Ok(to_binary(&res)?)
            },
            Err(_) => return Err(ContractError::BankNotRegistered {}), 
        }
        
    }

    pub fn query_transaction_info(deps: Deps, id: String) -> Result<Binary, ContractError> {
        match TRANSACTIONS.load(deps.storage, id) {
            Ok(item) => {
                let res = TransactionInfoResponse {
                    id: item.id,
                    bank_id: item.bank_id,
                    from: item.from,
                    to: item.to,
                    amount: item.amount,
                    denom_from: item.denom_from,
                    denom_to: item.denom_to,
                    status: item.status,
                };
                Ok(to_binary(&res)?)
            },
            Err(_) => Err(ContractError::TransactionDoesNotExist {}), 
        }
    }
}

pub fn create_accounts(
    deps: DepsMut,
    account: &Cw20Coin,
) -> Result<(), ContractError> {
        let address = deps.api.addr_validate(&account.address)?;
        BALANCES.save(deps.storage, &address, &account.amount)?;
        Ok(())
}

#[cfg(test)]
mod tests {
    use crate::msg::{TokenInfoResponse, ExchangeRateMsg, BankInfoResponse, TransactionInfoResponse};
    use crate::state::{BankInfo, TransactionInfo, TokenInfo};

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn register_bank() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
            
        let info = mock_info("addr0000", &coins(1000, "earth"));
        let msg = ExecuteMsg::CreateBank(BankInfo{ 
            id: "bank0000".to_string(),
            name: "Bank".to_string(),
            balance: Uint128::from(1000000u128),
        });
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::BankInfo { id: "bank0000".to_string() }
        ).unwrap();

        let value: BankInfoResponse = from_binary(&res).unwrap();
        assert_eq!("bank0000", value.id);
        assert_eq!("Bank", value.name);
        assert_eq!(Uint128::from(1000000u128), value.balance);
    }

    #[test]
    fn create_token() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
            
        let info = mock_info("addr0000", &coins(1000, "earth"));
        let msg = ExecuteMsg::CreateToken(TokenInfo {
            name: "Test".to_string(),
            symbol: "TEST".to_string(),
            decimals: 6,
            cw20coin: Cw20Coin {
                address: "addr0000".to_string(),
                amount: Uint128::from(1000000u128),
            },
        });
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenInfo { symbol: "TEST".to_string() }
        ).unwrap();

        let value: TokenInfoResponse = from_binary(&res).unwrap();
        assert_eq!("Test", value.name);
        assert_eq!("TEST", value.symbol);
        assert_eq!(6, value.decimals);
        assert_eq!("addr0000", value.cw20coin.address);
        assert_eq!(Uint128::from(1000000u128), value.cw20coin.amount);
    }

    #[test]
    fn create_bank() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
            
        let info = mock_info("addr0000", &coins(1000, "earth"));
        let msg = ExecuteMsg::CreateBank(BankInfo{ 
            id: "bank0000".to_string(),
            name: "Bank".to_string(),
            balance: Uint128::from(1000000u128),
        });
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::BankInfo { id: "bank0000".to_string() }
        ).unwrap();

        let value: BankInfoResponse = from_binary(&res).unwrap();
        assert_eq!("bank0000", value.id);
        assert_eq!("Bank", value.name);
        assert_eq!(Uint128::from(1000000u128), value.balance);
    }

    #[test]
    fn send_tokens() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "RUB"));

        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let rub_token = TokenInfo {
            name: "RUB".to_string(),
            symbol: "RUB".to_string(),
            decimals: 6,
            cw20coin: Cw20Coin {
                address: "addr0000".to_string(),
                amount: Uint128::from(1000000u128),
            },
        };

        let usd_token = TokenInfo {
            name: "USD".to_string(),
            symbol: "USD".to_string(),
            decimals: 6,
            cw20coin: Cw20Coin {
                address: "addr0001".to_string(),
                amount: Uint128::from(2000000u128),
            },
        };

        let msg = ExecuteMsg::CreateToken(rub_token);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::CreateToken(usd_token);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
    
        let msg = ExecuteMsg::CreateBank(BankInfo{ 
            id: "bank0000".to_string(),
            name: "Bank".to_string(),
            balance: Uint128::from(1000000u128),
        });
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        
        // means that real rate is 0.2
        let exchange_rate = ExecuteMsg::SetExchangeRate(ExchangeRateMsg {
            denom_from: "RUB".to_string(),
            denom_to: "USD".to_string(),
            precision: 3,
            rate: 200,
        });

        let res = execute(deps.as_mut(), mock_env(), info.clone(), exchange_rate).unwrap();
        assert_eq!(0, res.messages.len());

        let transaction = TransactionInfo {
            id: "transaction0000".to_string(),
            bank_id: "bank0000".to_string(),
            from: Addr::unchecked("addr0000"),
            to: Addr::unchecked("addr0001"),
            amount: Uint128::from(1000000u128),
            denom_from: "RUB".to_string(),
            denom_to: "USD".to_string(),
            status: TransactionStatus::Initial,
        };
        let msg = ExecuteMsg::SendToBank(transaction);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TransactionInfo { id: ("transaction0000".to_string()) } 
        ).unwrap();
        let tinfo: TransactionInfoResponse = from_binary(&res).unwrap();
        assert_eq!("transaction0000", tinfo.id);
        assert_eq!("bank0000", tinfo.bank_id);
        assert_eq!("addr0000", tinfo.from);
        assert_eq!("addr0001", tinfo.to);
        assert_eq!(Uint128::from(1000000u128), tinfo.amount);
        assert_eq!("RUB", tinfo.denom_from);
        assert_eq!("USD", tinfo.denom_to);
        assert_eq!(TransactionStatus::SentToBank, tinfo.status);

        let msg = ExecuteMsg::SendToRecipient { transaction_id: "transaction0000".to_string() };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TransactionInfo { id: ("transaction0000".to_string()) } 
        ).unwrap();
        let tinfo: TransactionInfoResponse = from_binary(&res).unwrap();
        assert_eq!("transaction0000", tinfo.id);
        assert_eq!("bank0000", tinfo.bank_id);
        assert_eq!("addr0000", tinfo.from);
        assert_eq!("addr0001", tinfo.to);
        assert_eq!(Uint128::from(1000000u128), tinfo.amount);
        assert_eq!("RUB", tinfo.denom_from);
        assert_eq!("USD", tinfo.denom_to);
        assert_eq!(TransactionStatus::SentToRecipient, tinfo.status);


        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance { address: "addr0000".to_string() }
        ).unwrap();
        let value: BalanceResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(0u128), value.balance);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance { address: "addr0001".to_string() }
        ).unwrap();
        let value: BalanceResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(2200000u128), value.balance);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::BankInfo { id: "bank0000".to_string() }
        ).unwrap();
        let value: BankInfoResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(1000000u128), value.balance);
    }
}
