#!/bin/bash
set -e

# Check if both arguments are provided: identity string and second wallet address
if [ -z "$1" ] || [ -z "$2" ]; then
    echo "Usage: $0 <identity_string> <second_wallet_address>"
    exit 1
fi

IDENTITY_STRING=$1
SECOND_IDENTITY_STRING=$2

NETWORK="testnet"

cd target/wasm32-unknown-unknown/release

echo "Optimize contracts..."

soroban contract optimize --wasm soroban_token_contract.wasm
soroban contract optimize --wasm phoenix_vesting.wasm

echo "Optimized contracts..."

ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)
SECOND_WALLET=$(soroban keys address $SECOND_IDENTITY_STRING)

echo "Admin address: $ADMIN_ADDRESS"
echo "Second wallet (vesting recipient): $SECOND_WALLET"

echo "Deploying Vesting contract..."
VESTING_ADDR=$(soroban contract deploy \
    --wasm phoenix_vesting.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)
echo "Vesting contract deployed at: $VESTING_ADDR"

echo "Deploying Vesting Token contract..."
VESTING_TOKEN_ADDR=$(soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    --admin $ADMIN_ADDRESS \
    --decimal 7 \
    --name VESTING \
    --symbol VEST
)

echo "Vesting Token contract deployed at: $VESTING_TOKEN_ADDR"
echo "Minting additional vesting tokens to admin..."
soroban contract invoke \
    --id $VESTING_TOKEN_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    mint --to $ADMIN_ADDRESS --amount 50000000000000 # 5_000_000 tokens

echo "Initializing Vesting contract..."
VESTING_TOKEN_JSON=$(cat <<EOF #learned new trick today
{"name": "VESTING", "symbol": "VEST", "decimals": 7, "address": "$VESTING_TOKEN_ADDR"}
EOF
)
soroban contract invoke \
    --id $VESTING_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --vesting_token "$VESTING_TOKEN_JSON" \
    --max_vesting_complexity 10

# within a 3-hour vesting period, we will:
# - let the vesting start 60 seconds from $now (min_x as a UNIX timestamp)
# - end after 3 hours (10800 seconds) later.
CURRENT_TIMESTAMP=$(date +%s)
START_TIMESTAMP=$((CURRENT_TIMESTAMP + 60))
END_TIMESTAMP=$((START_TIMESTAMP + 10800))
VESTING_AMOUNT=50000000000000

VESTING_SCHEDULE_JSON=$(cat <<EOF
[
  {
    "recipient": "$SECOND_WALLET",
    "curve": {
      "SaturatingLinear": {
        "min_x": $START_TIMESTAMP,
        "min_y": "$VESTING_AMOUNT",
        "max_x": $END_TIMESTAMP,
        "max_y": "0"
      }
    }
  }
]
EOF
)

echo "Creating vesting schedule for wallet $SECOND_WALLET:"
echo "  - Vesting amount: $VESTING_AMOUNT"
echo "  - Start timestamp (min_x): $START_TIMESTAMP"
echo "  - End timestamp (max_x):   $END_TIMESTAMP"

soroban contract invoke \
    --id $VESTING_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    create_vesting_schedules \
    --vesting_schedules "$VESTING_SCHEDULE_JSON"

echo "Vesting schedule successfully created."
