# Overview

Here is the first stage of the project for emulating the trading procedure but with bank as a man-in-the-middle between 2 accounts.
Currently we have all-in-one contract but I hope it will be moved to multi-contract architecture with dedicated responsibilities, like `account` and `bank`

# Compiling

From the project root:

```bash
docker run --rm -v "$(pwd)":/code \
          --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
          --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
          cosmwasm/workspace-optimizer:0.14.0
```

# Actions

## Test Mnemonic

```text
stock error strategy program tribe identify dune current kiss oil brisk improve carbon brick sausage vital cradle alien illegal resist lawsuit seed purpose pear
```

Account, associated with this mnemonic has some tokens for testing

Recover:

```bash
osmosisd keys add osmosis --recover
```
P.S. All the next `osmosisd` commands will be with assumption that key alias if `osmosis`

## Store contract

```bash
osmosisd tx wasm store artifacts/cw20_token.wasm --from osmosis --gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```

It returns the `code_id` in response and might be used for instantiating a contract.

## Instantiate

For instantiating you need only send the message like:

```bash
osmosisd tx wasm instantiate <code_id> '{}' --from osmosis --label "cw20-trading" --gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5 --no-admin
```

this command returns the contract address which can be used as a reference for all the next commands.

## Execute msgs

### CreateToken

It creates the `TokenInfo` state structure with next fields:

- `name` - human-readable name
- `denom`- short name of the currency, like USD or RUB or BTC
- `initial_balances` - list of `CW20Coin` structure with:
    - `address` - address with bech32 prefix
    - `amount` - amount of tokens

Command is:

```bash
osmosisd tx wasm execute <contract-address> '{"create_token": {"name": "ruble", "denom": "RUB", "initial_balances": [{"address": "osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6", "amount": "1000"}, {"address": "osmo19n8knfdas6xxqyya7e46dnx9lqjwalgagf8u4w", "amount": "2000"}]}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```

And for example another token:

```bash
osmosisd tx wasm execute <contract-address> '{"create_token": {"name": "dollar", "denom": "USD", "initial_balances": [{"address": "osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu", "amount": "1000"}, {"address": "osmo1am7n67uvmg03e04tjm3a96zer3d89jnw30676z", "amount": "2000"}]}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```

### CreateBank

Create the Bank actor which is responsible for converting the tokens using Exchange rules
`BankInfo` :

- `id` - bank identifier
- `name` - human-readable name
- `balance` - amount of tokens without denomination (Needs only for checking that transfer process was correct cause it should be the same after sending to recipient)

command:

```bash
osmosisd tx wasm execute <contract-address> '{"create_bank": {"id": "bank00001", "name": "Universal Bank", "balance": "100000000000"}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```

### Set exchange rate

Create the rule for converting one token to another. Cause cosmwasm doesn't allow to store float in state I chose to store exchange rate in the structure:

- `precision` - power of 10
- `rate` - rate number

#### Example

Let's assume that we wanna to use exchange coeff as `0.25` for converting from `D1` denom to `D2` . So in that case we can use 10^3 as a threshold and `rate` value `250` as a base.
Verse converting could be `1.0 / 0.25` and rate will be `4000`. In other words it shows that `D2` more then `D1` in `4000/10^3 = 4` times.

The setup message:

- `denom_from`
- `denom_to`
- `precision`
- `rate`


Command:

```bash
osmosisd tx wasm execute <contract-address> '{"set_exchange_rate": {"denom_from": "RUB", "denom_to": "USD", "precision": 3, "rate": 200}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```

### Send transaction to bank

It sends the transaction for sending tokens from one account address to another
Fields:
- `id` - transaction id
- `bank_id` - id of bank
- `from` - sender address
- `to` - recipient address
- `amount` - amount of tokens for sending from sender address.
- Recipient will get the amount of tokens according to exchange rules


Command:

```bash
osmosisd tx wasm execute <contract-address> '{"send_to_bank": {"id": "txn00001", "bank_id": "bank00001", "from": "osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6", "to": "osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu", "amount": "1000"}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```

### Send to recipient

This command is needed for applying previously sent txn.

It accepts only transaction id which already should be placed in the state

Command:

```bash
osmosisd tx wasm execute <contract-address> '{"send_to_recipient": {"transaction_id": "txn00001"}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```

## Query commands

### Get the balance

`{"balance": {"address": "<address>"}}` allows to get the balance for the particular address. This address should be assigned with token

Returns the structure like:
```
  amount: "6000"
  denom: USD
```

### Get the TokenInfo

`{"token_info": {"denom": "RUB"}}`

`denom` is a short name of token.
Returns the structure like:

```text
  denom: RUB
  initial_balances:
  - address: osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6
    amount: "1000"
  - address: osmo19n8knfdas6xxqyya7e46dnx9lqjwalgagf8u4w
    amount: "2000"
  name: ruble
  total_supply: "3000"
```

### Get the exchange rate

`{"exchange_rate_info": {"id": "USDRUB"}}`

Returns the structure like:
```text
  denom_from: USD
  denom_to: RUB
  id: USDRUB
  rate: 200
  precision: 3
```

### Get the Bank info

`{"bank_info": {"id": "bank00001"}}`

Returns the structure like:
```text
  balance: "100000000000"
  id: bank00001
  name: Universal Bank
```

### Get transaction info

`{"transaction_info": {"id": "txn00001"}}`

Returns the structure like:
```text
  amount: "1000"
  bank_id: bank00001
  denom_from: RUB
  denom_to: USD
  from: osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6
  id: txn00001
  status: sent_to_recipient
  to: osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu
```

# Example flow

1. Store contract
```
osmosisd tx wasm store artifacts/cw20_token.wasm --from osmosis --gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```
2. Instantiate
```
osmosisd tx wasm instantiate 4347 '{}' --from osmosis --label "cw20-trading" --gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5 --no-admin
```
3. Create a `RUB` token
```
osmosisd tx wasm execute osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"create_token": {"name": "ruble", "denom": "RUB", "initial_balances": [{"address": "osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6", "amount": "1000"}, {"address": "osmo19n8knfdas6xxqyya7e46dnx9lqjwalgagf8u4w", "amount": "2000"}]}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```
4. Create `USD` token
```
osmosisd tx wasm execute osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"create_token": {"name": "dollar", "denom": "USD", "initial_balances": [{"address": "osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu", "amount": "1000"}, {"address": "osmo1am7n67uvmg03e04tjm3a96zer3d89jnw30676z", "amount": "2000"}]}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```
5. Create a bank
```
osmosisd tx wasm execute osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"create_bank": {"id": "bank00001", "name": "Universal Bank", "balance": "100000000000"}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```
6. Setup exchange rate
```
osmosisd tx wasm execute osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"set_exchange_rate": {"denom_from": "RUB", "denom_to": "USD", "precision": 3, "rate": 200}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```
7. Get the TokenInfo
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"token_info": {"denom": "RUB"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  denom: RUB
  initial_balances:
  - address: osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6
    amount: "1000"
  - address: osmo19n8knfdas6xxqyya7e46dnx9lqjwalgagf8u4w
    amount: "2000"
  name: ruble
  total_supply: "3000"
```
And `USD`:
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"token_info": {"denom": "USD"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  denom: USD
  initial_balances:
  - address: osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu
    amount: "1000"
  - address: osmo1am7n67uvmg03e04tjm3a96zer3d89jnw30676z
    amount: "2000"
  name: dollar
  total_supply: "3000"
```
8. Get Balances
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"balance": {"address": "osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  amount: "1000"
  denom: RUB
```
And for another
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"balance": {"address": "osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  amount: "1000"
  denom: USD
```
9. Get BankInfo
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"bank_info": {"id": "bank00001"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  balance: "100000000000"
  id: bank00001
  name: Universal Bank
```
10. Get ExchangeRateInfo
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"exchange_rate_info": {"id": "RUBUSD"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  denom_from: RUB
  denom_to: USD
  id: RUBUSD
  precision: 3
  rate: 200
```
And Verse exchange rate:
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"exchange_rate_info": {"id": "USDRUB"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  denom_from: USD
  denom_to: RUB
  id: USDRUB
  precision: 3
  rate: 5000
```
11. Send txn to bank
```
osmosisd tx wasm execute osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"send_to_bank": {"id": "txn00001", "bank_id": "bank00001", "from": "osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6", "to": "osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu", "amount": "1000"}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```
12. Get TransactionInfo
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"transaction_info": {"id": "txn00001"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  amount: "1000"
  bank_id: bank00001
  denom_from: RUB
  denom_to: USD
  from: osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6
  id: txn00001
  status: sent_to_bank
  to: osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu
```
13. Send to recipient
```
osmosisd tx wasm execute osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"send_to_recipient": {"transaction_id": "txn00001"}}' --from osmosis \
--gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
```
14.  Get the TransactionInfo again
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"transaction_info": {"id": "txn00001"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  amount: "1000"
  bank_id: bank00001
  denom_from: RUB
  denom_to: USD
  from: osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6
  id: txn00001
  status: sent_to_recipient
  to: osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu
```
The status was changed to "SentToRecipient"
15.  Get balances again
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"balance": {"address": "osmo138cvlfj0j7rgn9jsj428kxrnauqgytr7ej0vp6"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  amount: "0"
  denom: RUB
```
And for recipient:
```
osmosisd query wasm contract-state smart osmo10dcwtvjqzsmsgq9kjk76ls5s67z02dhuesx2qqf8hqft97g2hzrsegncr8 '{"balance": {"address": "osmo1zr4d5vkwmuhtrh58dq0r28wp29z2r4mtp9mhxu"}}' --node https://rpc.osmotest5.osmosis.zone:443 --chain-id osmo-test-5
data:
  amount: "1200"
  denom: USD
```

So, the exchange rate was `0.2`, so the balance is as expected
