#!/bin/bash
# shellcheck disable=SC2086

set -euox pipefail

# sed in macos requires extra argument

# sed in MacOS requires extra argument
# if [[ "$OSTYPE" == "darwin"* ]]; then
#   SED_EXT='.orig'
# else 
#   SED_EXT=''
# fi

CHAIN_ID="osmosis"

apk add jq

# Node
osmosisd init --chain-id "$CHAIN_ID" testing

# User
osmosisd keys add osmosis-user --keyring-backend=test

# Genesis
GENESIS="$HOME/.osmosisd/config/genesis.json"
sed -i 's/"stake"/"uosmo"/' "$GENESIS"

# Config
sed -i 's|minimum-gas-prices = ""|minimum-gas-prices = "50uosmo"|g' "$HOME/.osmosisd/config/app.toml"

osmosisd add-genesis-account "$(osmosisd keys show osmosis-user -a --keyring-backend=test)" 2000000000uosmo
osmosisd gentx osmosis-user 500000000uosmo --keyring-backend=test --chain-id "$CHAIN_ID"
osmosisd collect-gentxs

jq '.app_state["gov"]["voting_params"]["voting_period"]="10s"' "$HOME/.osmosisd/config/genesis.json" > "$HOME/.osmosisd/config/tmp_genesis.json" && \
  mv "$HOME/.osmosisd/config/tmp_genesis.json" "$HOME/.osmosisd/config/genesis.json"

# Config
sed -i 's|laddr = "tcp://127.0.0.1:26657"|laddr = "tcp://0.0.0.0:26657"|g' "$HOME/.osmosisd/config/config.toml"
