#!/bin/bash

# Script to build, simulate and sign a Soroban contract invocation
# Usage: ./sign_invoke.sh <contract_id> "<function and arguments>"
# Example: ./sign_invoke.sh CBISULYO5ZGS32WTNCBMEFCNKNSLFXCQ4Z3XHVDP4X4FLPSEALGSY3PS "update --new_wasm_hash 167ab414a226427de34c19947ef9c5cf38c6c0ed91ecf9392f7cef3278ff506c"

set +xe

# Check if contract and arguments are provided
if [ $# -lt 2 ]; then
    echo "Usage: $0 <contract_id> \"<function and arguments>\""
    echo "Example: $0 CBISULYO5ZGS32WTNCBMEFCNKNSLFXCQ4Z3XHVDP4X4FLPSEALGSY3PS \"update --new_wasm_hash 167ab414a226427de34c19947ef9c5cf38c6c0ed91ecf9392f7cef3278ff506c\""
    exit 1
fi

CONTRACT=$1
ARGUMENTS="$2"

ACCOUNT="futurenetacc"

echo "Building transaction..."
built=$(soroban contract invoke \
    --id "$CONTRACT" \
    --source "$ACCOUNT" \
    --network mainnet \
    --fee 10000000 \
    --build-only \
    -- \
    $ARGUMENTS)

echo "Simulating..."
simulated=$(soroban tx simulate \
    --source-account "$ACCOUNT" \
    --network mainnet \
    "$built")

echo "Signing..."
signed=$(soroban tx sign \
    --network mainnet \
    --sign-with-key "$ACCOUNT" \
    "$simulated")

echo "$signed"
