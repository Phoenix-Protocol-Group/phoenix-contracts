#!/bin/bash

# Check if source account and send is provided
if [ -z "$2" ]; then
  echo "Error: Source account and Send is required as an argument."
  echo "Usage: $0 <source_account> $1 <send_tx>"
  exit 1
fi

# Input variables
SOURCE_ACCOUNT=$1
SEND_TX=$2
FACTORY_ID="CB4SVAWJA6TSRNOJZ7W2AWFW46D5VR4ZMFZKDIKXEINZCZEGZCJZCKMI"
RPC_URL="https://mainnet.sorobanrpc.com"
NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"

# Function to invoke Stellar contract
invoke_contract() {
  local FUNCTION_NAME=$1
  local ARGS=${2:-}

  stellar contract invoke \
    --id $FACTORY_ID \
    --source-account $SOURCE_ACCOUNT \
    --rpc-url $RPC_URL \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --send $SEND_TX \
    -- \
    $FUNCTION_NAME $ARGS | jq
}

# Query all pools
POOLS=$(invoke_contract query_pools | jq -r '.[]')

# Iterate over pools and query details
echo "Iterating over pool addresses"
for POOL in $POOLS; do
  invoke_contract query_pool_details "--pool_address $POOL"
done

# Query all pools details and save to variable
ALL_POOLS_DETAILS=$(invoke_contract query_all_pools_details)

# Iterate over all pools details and query token pairs
echo "Iterating over token_a and token_b"
echo $ALL_POOLS_DETAILS | jq -c '.[]' | while read -r POOL_DETAIL; do
  TOKEN_A=$(echo $POOL_DETAIL | jq -r '.pool_response.asset_a.address')
  TOKEN_B=$(echo $POOL_DETAIL | jq -r '.pool_response.asset_b.address')

  invoke_contract query_for_pool_by_token_pair "--token_a $TOKEN_A --token_b $TOKEN_B"
done

# Query admin address
echo "Querying admin"
invoke_contract get_admin

# Query config
echo "Querying config"
invoke_contract get_config
