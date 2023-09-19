#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfoResponse, TokenInfoMsg};
use crate::state::{TOKENS, TokenInfo, BALANCES, BANKS, TRANSACTIONS, TransactionStatus, ExchangeRateInfo, EXCHANGE_RATES, BalanceInfo};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-trading";
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

        ExecuteMsg::SendToBank(transaction_msg) => execute::execute_send_to_bank(deps, transaction_msg),
        ExecuteMsg::SendToRecipient { transaction_id } => execute::execute_send_to_recipient(deps, transaction_id),
}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Balance { address } => query::query_balance(deps, address),
        QueryMsg::TokenInfo { denom } => query::query_token_info(deps, denom),
        QueryMsg::BankInfo { id } => query::query_bank_info(deps, id),
        QueryMsg::TransactionInfo { id } => query::query_transaction_info(deps, id),
        QueryMsg::ExchangeRateInfo { id } => query::query_exchange_rate_info(deps, id),
    }
}

pub mod execute {

    use crate::{state::{TransactionInfo, BankInfo}, msg::{ExchangeRateMsg, TransactionMsg}, helpers::create_accounts};

    use super::*;

    pub fn execute_create_token(mut deps: DepsMut, token_info: TokenInfoMsg) -> Result<Response, ContractError> {
        let denom = token_info.denom.clone();
        if TOKENS.has(deps.storage, denom.clone()) {
            return Err(ContractError::TokenAlreadyRegistered { denom });
        }
        
        let total_supply = create_accounts(&mut deps, &token_info.initial_balances, denom.clone())?;

        let token_info = TokenInfo { 
            denom: denom.clone(), 
            name: token_info.name, 
            total_supply,
            initial_balances: token_info.initial_balances, 
        };
        TOKENS.save(deps.storage, denom,  &token_info)?;

        Ok(Response::default())
    }

    pub fn execute_create_bank(deps: DepsMut, bank_info: BankInfo) -> Result<Response, ContractError> {
        let bank_id = bank_info.id.clone();
        if BANKS.has(deps.storage, bank_id.clone()) {
            return Err(ContractError::BankAlreadyExists { id: bank_id.clone() });
        }
        BANKS.save(deps.storage, bank_id,  &bank_info)?;
        Ok(Response::default())
    }

    pub fn execute_set_exchange_rate(deps: DepsMut, exchange_rate: ExchangeRateMsg) -> Result<Response, ContractError> {
        let exchange_rate_id = exchange_rate.denom_from.to_owned() + exchange_rate.denom_to.as_str();
        let exchange_rate_id_verse = exchange_rate.denom_to.to_owned() + exchange_rate.denom_from.as_str();

        // We cannot store f64 in the state
        let base = 10_u128.pow(exchange_rate.precision);
        let verse_rate = base * base / exchange_rate.rate as u128;

        let exchange_rate_state = ExchangeRateInfo {
            id: exchange_rate_id.clone(),
            denom_from: exchange_rate.denom_from.clone(),
            denom_to: exchange_rate.denom_to.clone(),
            precision: exchange_rate.precision,
            rate: exchange_rate.rate,
        };

        // Verse case
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

    pub fn execute_send_to_bank(deps: DepsMut, transaction_info: TransactionMsg) -> Result<Response, ContractError> {
        let bank_id = transaction_info.bank_id.clone();
        let transaction_id = transaction_info.id.clone();
        // Validations
        // Check if bank exists
        if !BANKS.has(deps.storage, bank_id.clone()) {
            return Err(ContractError::BankNotRegistered { id: bank_id.clone() });
        }
        // Check if transaction exists
        if TRANSACTIONS.has(deps.storage, transaction_id.clone()) {
            return Err(ContractError::TransactionAlreadyExists { id: transaction_id.clone()});
        }

        if !BALANCES.has(deps.storage, &transaction_info.from) {
            return Err(ContractError::AccountDoesNotExist{account: transaction_info.from.to_string()});
        }

        if !BALANCES.has(deps.storage, &transaction_info.to) {
            return Err(ContractError::AccountDoesNotExist{account: transaction_info.to.to_string()});
        }
        // Check if sender has enough balance
        let balance_from = BALANCES.load(deps.storage, &transaction_info.from)?;
        if balance_from.amount < transaction_info.amount {
            return Err(ContractError::NotEnoughBalance { required: transaction_info.amount, available: balance_from.amount});
        }
        // decrease sender balance
        BALANCES.update(deps.storage, &transaction_info.from, |balance| -> StdResult<_> {
            Ok(
                BalanceInfo { 
                    amount: balance.clone().unwrap().amount - transaction_info.amount, 
                    denom: balance.unwrap().denom,})
        })?;

        // increase bank balance
        BANKS.update(deps.storage, bank_id.clone(), |bank| -> StdResult<_> {
            Ok(bank.unwrap().income(transaction_info.amount))
        })?;

        let balance_to = BALANCES.load(deps.storage, &transaction_info.to)?;

        let transaction = TransactionInfo {
            id: transaction_id.clone(),
            bank_id: bank_id.clone(),
            from: transaction_info.from.clone(),
            to: transaction_info.to.clone(),
            amount: transaction_info.amount.clone(),
            denom_from: balance_from.denom.clone(),
            denom_to: balance_to.denom.clone(),
            status: TransactionStatus::SentToBank,
        };
        // Update transaction status
        TRANSACTIONS.save(deps.storage, transaction_id, &transaction)?;
        Ok(Response::default())
    }

    pub fn execute_send_to_recipient(deps: DepsMut, transaction_id: String) -> Result<Response, ContractError> {
        // Check if transaction exists

        if !TRANSACTIONS.has(deps.storage, transaction_id.clone()) {
            return Err(ContractError::TransactionDoesNotExist {id: transaction_id.clone()});
        }

        let transaction_info = TRANSACTIONS.load(deps.storage, transaction_id.clone())?;

        // Check that exchange rate exists
        let exchange_rate_id = transaction_info.denom_from.to_owned() + transaction_info.denom_to.as_str();

        if !EXCHANGE_RATES.has(deps.storage, exchange_rate_id.clone()) {
            return Err(ContractError::ExchangeRateDoesNotExist { id: exchange_rate_id.clone()});
        }

        // Check if bank exists
        let bank_id = transaction_info.bank_id.clone();
        if !BANKS.has(deps.storage, bank_id.clone()) {
            return Err(ContractError::BankNotRegistered { id: bank_id.clone() });
        }

        let exchange_rate = EXCHANGE_RATES.load(deps.storage, exchange_rate_id)?; // TODO: check if it is correct

        let bank = BANKS.load(deps.storage, bank_id.clone())?; // TODO: check if it is correct

        if bank.balance < transaction_info.amount {
            return Err(ContractError::NotEnoughBalance {available: bank.balance, required: transaction_info.amount});
        }
        
        // decrease bank balance
        BANKS.update(deps.storage, bank_id, |bank| -> StdResult<_> {
            Ok(bank.unwrap().outcome(transaction_info.amount))
        })?;

        // Calculate balance due to exchange rate
        let amount = transaction_info.amount.u128() * exchange_rate.rate as u128 / 10_u128.pow(exchange_rate.precision);

        BALANCES.update(deps.storage, &transaction_info.to, |balance| -> StdResult<_> {
            Ok(
                BalanceInfo {
                    amount: balance.unwrap().amount + Uint128::from(amount),
                    denom: transaction_info.denom_to.clone(),
            })
        })?;

        let mut transaction = transaction_info;
        // Update transaction status
        TRANSACTIONS.save(deps.storage, transaction_id, &transaction.update_status(TransactionStatus::SentToRecipient))?;
        Ok(Response::default())
    }
}
pub mod query {
    use crate::msg::{BankInfoResponse, TransactionInfoResponse, BalanceResponse, ExchangeRateInfoResponse};

    use super::*;

    pub fn query_balance(deps: Deps, address: String) -> Result<Binary, ContractError> {
        let address = deps.api.addr_validate(&address)?;
        if !BALANCES.has(deps.storage, &address) {
            return Err(ContractError::AccountDoesNotExist { account: address.to_string() });
        }
        let balance = BALANCES.load(deps.storage, &address)?;
            
        Ok(to_binary(&BalanceResponse { 
            amount: balance.amount, 
            denom: balance.denom
        })?)
    }

    pub fn query_token_info(deps: Deps, denom: String) -> Result<Binary, ContractError> {
        match TOKENS.load(deps.storage, denom.clone()) {
            Ok(info) => {
                let res = TokenInfoResponse {
                    name: info.name,
                    denom: info.denom,
                    total_supply: info.total_supply,
                    initial_balances: info.initial_balances,
                };
                Ok(to_binary(&res)?)
            },
            Err(_) => Err(ContractError::TokenNotRegistered { denom }),
        }
    }

    pub fn query_bank_info(deps: Deps, id: String) -> Result<Binary, ContractError> {
        match BANKS.load(deps.storage, id.clone()) {
            Ok(item) => {
                let res = BankInfoResponse {
                    id: item.id,
                    name: item.name,
                    balance: item.balance,
                };
                Ok(to_binary(&res)?)
            },
            Err(_) => return Err(ContractError::BankNotRegistered { id }), 
        }
        
    }

    pub fn query_transaction_info(deps: Deps, id: String) -> Result<Binary, ContractError> {
        match TRANSACTIONS.load(deps.storage, id.clone()) {
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
            Err(_) => Err(ContractError::TransactionDoesNotExist { id }), 
        }
    }

    pub fn query_exchange_rate_info(deps: Deps, id: String) -> Result<Binary, ContractError> {
        match EXCHANGE_RATES.load(deps.storage, id.clone()) {
            Ok(item) => {
                let res = ExchangeRateInfoResponse {
                    id: item.id,
                    denom_from: item.denom_from,
                    denom_to: item.denom_to,
                    rate: item.rate,
                    precision: item.precision,
                };
                Ok(to_binary(&res)?)
            },
            Err(_) => Err(ContractError::ExchangeRateDoesNotExist { id }), 
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{TransactionMsg, TokenInfoResponse, ExchangeRateMsg, BankInfoResponse, TransactionInfoResponse, BalanceResponse, ExchangeRateInfoResponse};
    use crate::state::BankInfo;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};
    use cw20::Cw20Coin;

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
    fn set_exchange_rate() {
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
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let exchange_rate = ExecuteMsg::SetExchangeRate(ExchangeRateMsg {
            denom_from: "RUB".to_string(),
            denom_to: "USD".to_string(),
            precision: 3,
            rate: 200,
        });

        let res = execute(deps.as_mut(), mock_env(), info, exchange_rate).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ExchangeRateInfo { id: "RUBUSD".to_string() }
        ).unwrap();

        let value: ExchangeRateInfoResponse = from_binary(&res).unwrap();
        assert_eq!("RUBUSD", value.id);
        assert_eq!("RUB", value.denom_from);
        assert_eq!("USD", value.denom_to);
        assert_eq!(200, value.rate);
        assert_eq!(3, value.precision);

        // Verse exchange rate

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ExchangeRateInfo { id: "USDRUB".to_string() }
        ).unwrap();

        let value: ExchangeRateInfoResponse = from_binary(&res).unwrap();
        assert_eq!("USDRUB", value.id);
        assert_eq!("USD", value.denom_from);
        assert_eq!("RUB", value.denom_to);
        assert_eq!(5000, value.rate);
    }

    #[test]
    fn create_token() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
            
        let info = mock_info("addr0000", &coins(1000, "earth"));
        let msg = ExecuteMsg::CreateToken(TokenInfoMsg {
            name: "Test".to_string(),
            denom: "TEST".to_string(),
            initial_balances: vec![Cw20Coin {
                address: "addr0000".to_string(),
                amount: Uint128::from(1000000u128),
            }, 
            Cw20Coin {
                address: "addr0001".to_string(),
                amount: Uint128::from(1000000u128),
            }],
        });
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenInfo { denom: "TEST".to_string() }
        ).unwrap();

        let value: TokenInfoResponse = from_binary(&res).unwrap();
        assert_eq!("Test", value.name);
        assert_eq!("TEST", value.denom);
        assert_eq!(Uint128::from(2000000u128), value.total_supply);
        assert_eq!("addr0000", value.initial_balances[0].address);
        assert_eq!(Uint128::from(1000000u128), value.initial_balances[0].amount);
        assert_eq!("addr0001", value.initial_balances[1].address);
        assert_eq!(Uint128::from(1000000u128), value.initial_balances[1].amount);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance { address: "addr0000".to_string() }
        ).unwrap();
        let value: BalanceResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(1000000u128), value.amount);
        assert_eq!("TEST", value.denom);
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

        let rub_token = TokenInfoMsg {
            name: "RUB".to_string(),
            denom: "RUB".to_string(),
            initial_balances: vec![Cw20Coin {
                address: "addr0000".to_string(),
                amount: Uint128::from(1000000u128),
            }],
        };

        let usd_token = TokenInfoMsg {
            name: "USD".to_string(),
            denom: "USD".to_string(),
            initial_balances: vec![Cw20Coin {
                address: "addr0001".to_string(),
                amount: Uint128::from(2000000u128),
            }],
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

        let transaction = TransactionMsg {
            id: "transaction0000".to_string(),
            bank_id: "bank0000".to_string(),
            from: Addr::unchecked("addr0000"),
            to: Addr::unchecked("addr0001"),
            amount: Uint128::from(1000000u128),
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
        assert_eq!(Uint128::from(0u128), value.amount);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance { address: "addr0001".to_string() }
        ).unwrap();
        let value: BalanceResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(2200000u128), value.amount);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::BankInfo { id: "bank0000".to_string() }
        ).unwrap();
        let value: BankInfoResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(1000000u128), value.balance);
    }
}
