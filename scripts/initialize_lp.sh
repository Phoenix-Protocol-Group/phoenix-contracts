#!/bin/bash

# Initialize variables with default values
IDENTITY_STRING=""
FACTORY_ADDR=""
TOKEN_ID1=""
TOKEN_ID2=""
WASM_DIR="target/wasm32-unknown-unknown/release"

# Default TOKEN_RATIO
TOKEN_RATIO=1.0

# Function to check if a directory exists
check_directory() {
  if [ -d "$1" ]; then
    WASM_DIR="$1"
  fi
}

# Function to install a contract and return the hash
install_contract() {
  local CONTRACT_NAME="$1"
  local CONTRACT_WASM_FILE="$2"

  local CONTRACT_WASM_HASH=$(soroban contract install \
    --wasm "$WASM_DIR/$CONTRACT_WASM_FILE" \
    --source $IDENTITY_STRING \
    --network futurenet)

  echo "$CONTRACT_NAME contract installed."
  echo "$CONTRACT_WASM_HASH"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --identity)
      IDENTITY_STRING="$2"
      shift 2
      ;;
    --factory)
      FACTORY_ADDR="$2"
      shift 2
      ;;
    --token1)
      TOKEN_ID1="$2"
      shift 2
      ;;
    --token2)
      TOKEN_ID2="$2"
      shift 2
      ;;
    --token_ratio)
      TOKEN_RATIO=$(bc <<< "$2")
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Check if required parameters are provided
if [[ -z $FACTORY_ADDR || -z $TOKEN_ID1 || -z $TOKEN_ID2 || -z $IDENTITY_STRING ]]; then
  echo "Initialization of liquidity pool failed: Required parameters missing."
  exit 1
fi

# Check if the WASM directory exists
check_directory "../$WASM_DIR"
check_directory "$WASM_DIR"

echo "Install the soroban_token, phoenix_pair and phoenix_stake contracts..."

# Install contracts and store the hashes
TOKEN_WASM_HASH=$(install_contract "TOKEN" "soroban_token_contract.optimized.wasm")
PAIR_WASM_HASH=$(install_contract "PAIR" "phoenix_pair.optimized.wasm")
STAKE_WASM_HASH=$(install_contract "STAKE" "phoenix_stake.optimized.wasm")

echo "Token, pair and stake contracts deployed."

echo "Initialize pair using the previously fetched hashes through factory..."

if [[ "$TOKEN_ID1" < "$TOKEN_ID2" ]]; then
    TOKEN_ID1=$TOKEN_ID1
    TOKEN_ID2=$TOKEN_ID2
else
    TOKEN_ID1=$TOKEN_ID2
    TOKEN_ID2=$TOKEN_ID1
fi

soroban contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    create_liquidity_pool \
    --lp_init_info "{ \"admin\": \"${ADMIN_ADDRESS}\", \"lp_wasm_hash\": \"${PAIR_WASM_HASH}\", \"share_token_decimals\": 7, \"swap_fee_bps\": 1000, \"fee_recipient\": \"${ADMIN_ADDRESS}\", \"max_allowed_slippage_bps\": 10000, \"max_allowed_spread_bps\": 10000, \"token_init_info\": { \"token_wasm_hash\": \"${TOKEN_WASM_HASH}\", \"token_a\": \"${TOKEN_ID1}\", \"token_b\": \"${TOKEN_ID2}\" }, \"stake_init_info\": { \"stake_wasm_hash\": \"${STAKE_WASM_HASH}\", \"min_bond\": \"100\", \"min_reward\": \"100\", \"max_distributions\": 3 } }"

PAIR_ADDR=$(soroban contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet --fee 100 \
    -- \
    query_pools | jq -r '.[0]')

echo "Pair contract initialized."

echo "Mint both tokens to the admin and provide liquidity..."
if (( $(echo "$TOKEN_RATIO < 1.0" | bc -l) )); then
  # If TOKEN_RATIO is less than 1.0, swap desired_a and desired_b
  desired_a=$((50000000000 / $TOKEN_RATIO))
  desired_b=$((100000000000 / $TOKEN_RATIO))
else
  desired_a=$((100000000000 * $TOKEN_RATIO))
  desired_b=$((100000000000 / $TOKEN_RATIO))
fi

soroban contract invoke \
    --id $TOKEN_ID1 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $ADMIN_ADDRESS --amount $desired_a

soroban contract invoke \
    --id $TOKEN_ID2 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $ADMIN_ADDRESS --amount $desired_b

# Provide liquidity in 2:1 ratio to the pool
soroban contract invoke \
    --id $PAIR_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet --fee 10000000 \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a $desired_a --desired_b $desired_b

echo "Liquidity provided."

# Continue with the rest of the commands
echo "Bond tokens to stake contract..."

STAKE_ADDR=$(soroban contract invoke \
    --id $PAIR_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet --fee 10000000 \
    -- \
    query_stake_contract_address | jq -r '.')

# Bond token in stake contract
soroban contract invoke \
    --id $STAKE_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    bond --sender $ADMIN_ADDRESS --tokens 70000000000


