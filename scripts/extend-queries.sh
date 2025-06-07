#!/bin/bash

set +x

# validate the input arguments
if [ -z "$2" ]; then
  echo "Error: Source account and Send is required as an argument." >&2
  echo "Usage: $0 <source_account> <send_tx> [output_file]" >&2
  exit 1
fi

# vars
SOURCE_ACCOUNT=$1
SEND_TX=$2
OUTPUT_FILE=${3:-/dev/null} # Redirect output to a file or /dev/null by default
FACTORY_ID="CB4SVAWJA6TSRNOJZ7W2AWFW46D5VR4ZMFZKDIKXEINZCZEGZCJZCKMI"
RPC_URL="https://mainnet.sorobanrpc.com"
NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"

# helper to invoke the stellar network
invoke_contract() {
  local CONTRACT_ID=$1
  local FUNCTION_NAME=$2
  local ARGS=${3:-}

  echo "Calling function: $FUNCTION_NAME on contract: $CONTRACT_ID with args: $ARGS" >&2

  # capture raw output and errors
  RAW_OUTPUT=$(stellar contract invoke \
    --id $CONTRACT_ID \
    --source-account $SOURCE_ACCOUNT \
    --rpc-url $RPC_URL \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --send $SEND_TX \
    -- \
    $FUNCTION_NAME $ARGS 2>&1)

  # check for archived entry error
  if echo "$RAW_OUTPUT" | grep -q "EntryArchived"; then
    echo "Error: EntryArchived for function: $FUNCTION_NAME on contract: $CONTRACT_ID" >&2
    return 1
  fi

  # parse JSON output with jq
  echo "$RAW_OUTPUT" | jq
}

# redirect all output to the specified file or /dev/null
exec >"$OUTPUT_FILE" 2>&1

# all pools of factory
POOLS=$(invoke_contract $FACTORY_ID query_pools | jq -r '.[]')

# query the pools and get the details
echo "Iterating over pool addresses" >&2
for POOL in $POOLS; do
  invoke_contract $FACTORY_ID query_pool_details "--pool_address $POOL"
done

# same like before, just another storage key
ALL_POOLS_DETAILS=$(invoke_contract $FACTORY_ID query_all_pools_details)

# arrays for staking and lp_share token
STAKE_ADDRESSES=()
LP_SHARE_ADDRESSES=()

# fill in the arrays
echo "Extracting stake addresses and LP share addresses" >&2
while read -r POOL_DETAIL; do
  STAKE_ADDRESS=$(echo "$POOL_DETAIL" | jq -r '.pool_response.stake_address')
  LP_SHARE_ADDRESS=$(echo "$POOL_DETAIL" | jq -r '.pool_response.asset_lp_share.address')

  STAKE_ADDRESSES+=("$STAKE_ADDRESS")
  LP_SHARE_ADDRESSES+=("$LP_SHARE_ADDRESS")
done < <(echo "$ALL_POOLS_DETAILS" | jq -c '.[]')

echo "DONE WITH FACTORY QUERIES" >&2

echo "STARTING WITH QUERIES IN POOL" >&2

# call the queries in pool contract
for POOL in $POOLS; do
  invoke_contract $POOL query_config
  invoke_contract $POOL query_share_token_address
  invoke_contract $POOL query_stake_contract_address
  invoke_contract $POOL query_pool_info
  invoke_contract $POOL query_pool_info_for_factory
done

echo "DONE WITH QUERIES IN POOL CONTRACTS" >&2

echo "STARTING WITH STAKE CONTRACT QUERIES" >&2

# call the queries in stake contract
for STAKE in "${STAKE_ADDRESSES[@]}"; do
  echo "Querying stake contract: $STAKE" >&2
  invoke_contract $STAKE query_config || continue
  invoke_contract $STAKE query_admin || continue
  invoke_contract $STAKE query_total_staked || continue
  # invoke_contract $STAKE query_annualized_rewards TODO - not present in the current version
done

echo "DONE WITH STAKE CONTRACT QUERIES" >&2

echo "STARTING WITH LP SHARE QUERIES" >&2

# call the queries in token contract
for LP_SHARE in "${LP_SHARE_ADDRESSES[@]}"; do
  echo "Querying LP share name: $LP_SHARE" >&2
  invoke_contract $LP_SHARE name || continue
done

echo "DONE WITH ALL QUERIES" >&2
