#!/usr/bin/bash

# Define the keys | https://lab.stellar.org/xdr/view | XDR type ScVal
key1="AAAAAwAAAAE=" # u32: '1'
key2="AAAAAwAAAAI=" # u32: '2'
key3="AAAAAwAAAAM=" # u32: '3'
# keystring="AAAADwAAAAhEU0xJUEJQUw==" # symbol: "DSLIPBPS"

# Validate input arguments
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <CONTRACT> <ACCOUNT>"
    exit 1
fi

# Assign arguments to constants
CONTRACT=$1
ACCOUNT=$2

echo "Restore..."
    restored=$(soroban contract restore \
      --id "$CONTRACT" \
      --source-account "$ACCOUNT" \
      --rpc-url https://mainnet.sorobanrpc.com \
      --network-passphrase "Public Global Stellar Network ; September 2015" \
      --build-only )

    echo "Simulate..."
    simulated=$(soroban tx simulate \
      --source-account "$ACCOUNT" \
      --rpc-url https://mainnet.sorobanrpc.com \
      --network-passphrase "Public Global Stellar Network ; September 2015" \
      "$restored")

    echo "Sign..."
    signed=$(soroban tx sign \
      --rpc-url https://mainnet.sorobanrpc.com \
      --network-passphrase "Public Global Stellar Network ; September 2015" \
      --sign-with-key "$ACCOUNT" \
      "$simulated")

    echo "Send!"
    soroban tx send --quiet \
      --rpc-url https://mainnet.sorobanrpc.com \
      --network-passphrase "Public Global Stellar Network ; September 2015" \
      "$signed"



# Loop through the keys
for key in "$key1" "$key2"; do
    echo "Processing key: $key"

    echo "Restore..."
    restored=$(soroban contract restore \
      --id "$CONTRACT" \
      --source-account "$ACCOUNT" \
      --rpc-url https://mainnet.sorobanrpc.com \
      --network-passphrase "Public Global Stellar Network ; September 2015" \
      --build-only \
      --key-xdr "$key")

    echo "Simulate..."
    simulated=$(soroban tx simulate \
      --source-account "$ACCOUNT" \
      --rpc-url https://mainnet.sorobanrpc.com \
      --network-passphrase "Public Global Stellar Network ; September 2015" \
      "$restored")

    echo "Sign..."
    signed=$(soroban tx sign \
      --rpc-url https://mainnet.sorobanrpc.com \
      --network-passphrase "Public Global Stellar Network ; September 2015" \
      --sign-with-key "$ACCOUNT" \
      "$simulated")

    echo "Send!"
    soroban tx send --quiet \
      --rpc-url https://mainnet.sorobanrpc.com \
      --network-passphrase "Public Global Stellar Network ; September 2015" \
      "$signed"

    echo "-----------------------------"
done

