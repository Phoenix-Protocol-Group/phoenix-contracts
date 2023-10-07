#!/bin/bash

# Initialize variables with default values
WASM_DIR="target/wasm32-unknown-unknown/release"
IDENTITY_STRING=""
ADMIN_ADDRESS=""
TOKEN_NAME=""
TOKEN_SYMBOL=""
TOKEN_ADDR=""

# Function to check if a directory exists
check_directory() {
  if [ -d "$1" ]; then
    WASM_DIR="$1"
  fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --identity)
      IDENTITY_STRING="$2"
      shift 2
      ;;
    --token_name)
      TOKEN_NAME="$2"
      shift 2
      ;;
    --token_symbol)
      TOKEN_SYMBOL="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Check if required parameters are provided
if [[ -z $TOKEN_NAME || -z $TOKEN_SYMBOL || -z $IDENTITY_STRING ]]; then
  echo "Initialization of token failed: Required parameters missing."
  exit 1
fi

ADMIN_ADDRESS=$(soroban config identity address $IDENTITY_STRING)

# Deploy token contracts and store the addresses
TOKEN_ADDR=$(soroban contract deploy \
    --wasm "$WASM_DIR/soroban_token_contract.optimized.wasm" \
    --source $IDENTITY_STRING \
    --network futurenet)

# Output the addresses in the required format
echo "TOKEN_ADDR=$TOKEN_ADDR"

echo "Initialize the token contracts..."

# Initialize the first token contract
soroban contract invoke \
    --id $TOKEN_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --decimal 7 \
    --name "$TOKEN_NAME" \
    --symbol "$TOKEN_SYMBOL"

echo "Token initialized."

