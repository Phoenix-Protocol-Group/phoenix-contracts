# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

IDENTITY_STRING=$1

# Build and optimize the contracts
make build > /dev/null
cd target/wasm32-unknown-unknown/release

echo "Contracts compiled..."

soroban contract optimize --wasm soroban_token_contract.wasm
soroban contract optimize --wasm phoenix_pair.wasm
soroban contract optimize --wasm phoenix_stake.wasm

echo "Contracts optimized..."

# Fetch the admin's address
ADMIN_ADDRESS=$(soroban config identity address $IDENTITY_STRING)

# Deploy the soroban_token_contract and capture its contract ID hash
TOKEN_ADDR1=$(soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

TOKEN_ADDR2=$(soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

echo "Tokens deployed..."

# Sort the token addresses alphabetically
if [[ "$TOKEN_ADDR1" < "$TOKEN_ADDR2" ]]; then
    TOKEN_ID1=$TOKEN_ADDR1
    TOKEN_ID2=$TOKEN_ADDR2
else
    TOKEN_ID1=$TOKEN_ADDR2
    TOKEN_ID2=$TOKEN_ADDR1
fi

# Initialize the contracts
soroban contract invoke \
    --id $TOKEN_ID1 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --decimal 7 \
    --name TOKEN \
    --symbol TOK

soroban contract invoke \
    --id $TOKEN_ID2 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --decimal 7 \
    --name PHOENIX \
    --symbol PHO

echo "Tokens initialized..."

# Install the soroban_token_contract and capture its hash
TOKEN_WASM_HASH=$(soroban contract install \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

# Continue with the rest of the deployments
PAIR_CONTRACT=$(soroban contract deploy \
    --wasm phoenix_pair.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

STAKE_WASM_HASH=$(soroban contract install \
    --wasm phoenix_stake.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

echo "Pair and stake contracts deployed..."


# Initialize pair using the previously fetched hashes
soroban contract invoke \
    --id $PAIR_CONTRACT \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --share_token_decimals 7 \
    --swap_fee_bps 1000 \
    --fee_recipient $ADMIN_ADDRESS \
    --max_allowed_slippage_bps 10000 \
    --max_allowed_spread_bps 10000 \
    --token_init_info "{\"token_a\": \"${TOKEN_ID1}\", \"token_b\": \"${TOKEN_ID2}\", \"token_wasm_hash\": \"${TOKEN_WASM_HASH}\"}" \
    --stake_init_info "{\"stake_wasm_hash\": \"${STAKE_WASM_HASH}\", \"min_bond\": \"100\", \"min_reward\": \"100\", \"max_distributions\": 3}"

    # --token_init_info '{"token_a": \"${TOKEN_ID1}\", "token_b": \"${TOKEN_ID2}\", "token_wasm_hash": \"${TOKEN_WASM_HASH}\"}' \
    # --stake_init_info '{"stake_wasm_hash": \"${STAKE_WASM_HASH}\", "min_bond": "100", "min_reward": "100", "max_distributions": 3}'

    # --token_init_info '{ "token_a": "$(TOKEN_ID1)", "token_b": "$(TOKEN_ID2)", "token_wasm_hash": "$(TOKEN_WASM_HASH)" }' \
    # --stake_init_info '{ "stake_wasm_hash": "$(STAKE_WASM_HASH)", "min_bond": "100", "min_reward": "100", "max_distributions": "3" }'

echo "Pair contract initialized..."

# Mint both tokens to the admin
soroban contract invoke \
    --id $TOKEN_ID1 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $ADMIN_ADDRESS --amount 100000000000

soroban contract invoke \
    --id $TOKEN_ID2 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $ADMIN_ADDRESS --amount 100000000000

# Provide liquidity in 2:1 ratio to the pool
soroban contract invoke \
    --id $PAIR_CONTRACT \
    --source $IDENTITY_STRING \
    --network futurenet --fee 10000000 \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 100000000000 --desired_b 50000000000

echo "Liquidity provided..."

# Continue with the rest of the commands
STAKE_CONTRACT=$(soroban contract invoke \
    --id $PAIR_CONTRACT \
    --source $IDENTITY_STRING \
    --network futurenet --fee 10000000 \
    -- \
    query_stake_contract_address)

# Bond token in stake contract
soroban contract invoke \
    --id $STAKE_CONTRACT \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    bond --sender $ADMIN_ADDRESS --tokens 70000000000

echo "Tokens bonded..."

echo "Initialization complete!"
echo "Token Contract 1 Hash: $TOKEN_ID1"
echo "Token Contract 2 Hash: $TOKEN_ID2"
echo "Pair Contract Hash: $PAIR_CONTRACT"
echo "Stake Contract Hash: $STAKE_CONTRACT"
