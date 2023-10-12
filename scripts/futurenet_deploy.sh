# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

IDENTITY_STRING=$1

# FAUCET ADDRESS
FAUCET=GAFNG7UOA2FDD745PVHFYHSZEIMJ6NYY2BY7ONJ74MRZGHSU2NEHBZ74

echo "Build and optimize the contracts...";

make build > /dev/null
cd target/wasm32-unknown-unknown/release

echo "Contracts compiled."
echo "Optimize contracts..."

soroban contract optimize --wasm soroban_token_contract.wasm
soroban contract optimize --wasm phoenix_factory.wasm
soroban contract optimize --wasm phoenix_pool.wasm
soroban contract optimize --wasm phoenix_stake.wasm
soroban contract optimize --wasm phoenix_multihop.wasm

echo "Contracts optimized."

# Fetch the admin's address
ADMIN_ADDRESS=$(soroban config identity address $IDENTITY_STRING)

echo "Deploy the soroban_token_contract and capture its contract ID hash..."

TOKEN_ADDR1=$(soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

TOKEN_ADDR2=$(soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

FACTORY_ADDR=$(soroban contract deploy \
    --wasm phoenix_factory.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

echo "Tokens and factory deployed."

# Sort the token addresses alphabetically
if [[ "$TOKEN_ADDR1" < "$TOKEN_ADDR2" ]]; then
    TOKEN_ID1=$TOKEN_ADDR1
    TOKEN_ID2=$TOKEN_ADDR2
else
    TOKEN_ID1=$TOKEN_ADDR2
    TOKEN_ID2=$TOKEN_ADDR1
fi

echo "Initialize factory..."

soroban contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS

echo "Factory initialized."

echo "Initialize the token contracts..."

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

echo "Tokens initialized."

echo "Install the soroban_token, phoenix_pool and phoenix_stake contracts..."

TOKEN_WASM_HASH=$(soroban contract install \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

# Continue with the rest of the deployments
PAIR_WASM_HASH=$(soroban contract install \
    --wasm phoenix_pool.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

STAKE_WASM_HASH=$(soroban contract install \
    --wasm phoenix_stake.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

echo "Token, pair and stake contracts deployed."


echo "Initialize pair using the previously fetched hashes through factory..."

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
soroban contract invoke \
    --id $TOKEN_ADDR1 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $ADMIN_ADDRESS --amount 100000000000

soroban contract invoke \
    --id $TOKEN_ADDR2 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $ADMIN_ADDRESS --amount 100000000000

# Provide liquidity in 2:1 ratio to the pool
soroban contract invoke \
    --id $PAIR_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet --fee 10000000 \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 100000000000 --desired_b 50000000000

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

echo "Tokens bonded."

echo "Deploy and initializa a multihop..."

MULTIHOP=$(soroban contract deploy \
    --wasm phoenix_multihop.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

soroban contract invoke \
    --id $MULTIHOP \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --factory $FACTORY_ADDR

echo "Multihop initialized."

echo "Mint tokens to the faucet..."

soroban contract invoke \
    --id $TOKEN_ADDR1 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $FAUCET --amount 100000000000

soroban contract invoke \
    --id $TOKEN_ADDR2 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $FAUCET --amount 100000000000

echo "Tokens minted."

echo "#############################"

echo "Deploy the soroban_token_contract and capture its contract ID hash..."

TOKEN_ADDR1=$(soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

echo "Tokens and factory deployed."

# Sort the token addresses alphabetically
if [[ "$TOKEN_ADDR1" < "$TOKEN_ADDR2" ]]; then
    TOKEN_ID1=$TOKEN_ADDR1
    TOKEN_ID2=$TOKEN_ADDR2
else
    TOKEN_ID1=$TOKEN_ADDR2
    TOKEN_ID2=$TOKEN_ADDR1
fi

echo "Initialize factory..."

echo "Initialize the token contracts..."

soroban contract invoke \
    --id $TOKEN_ADDR1 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --decimal 7 \
    --name Eurocoin \
    --symbol EURC

echo "Tokens initialized."

echo "Install the soroban_token, phoenix_pool and phoenix_stake contracts..."

TOKEN_WASM_HASH=$(soroban contract install \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

# Continue with the rest of the deployments
PAIR_WASM_HASH=$(soroban contract install \
    --wasm phoenix_pool.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

STAKE_WASM_HASH=$(soroban contract install \
    --wasm phoenix_stake.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

echo "Token, pair and stake contracts deployed."


echo "Initialize pair using the previously fetched hashes through factory..."

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
    query_pools | jq -r '.[1]')

echo "Pair contract initialized."

echo "Mint both tokens to the admin and provide liquidity..."
soroban contract invoke \
    --id $TOKEN_ADDR1 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $ADMIN_ADDRESS --amount 100000000000

soroban contract invoke \
    --id $TOKEN_ADDR2 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $ADMIN_ADDRESS --amount 100000000000

# Provide liquidity in 2:1 ratio to the pool
soroban contract invoke \
    --id $PAIR_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet --fee 10000000 \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 100000000000 --desired_b 50000000000

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

echo "Tokens bonded."

echo "Mint tokens to the faucet..."

soroban contract invoke \
    --id $TOKEN_ADDR1 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $FAUCET --amount 100000000000

echo "Tokens minted."

echo "#############################"

echo "now third time"

echo "Deploy the soroban_token_contract and capture its contract ID hash..."

TOKEN_ADDR1=CB64D3G7SM2RTH6JSGG34DDTFTQ5CFDKVDZJZSODMCX4NJ2HV2KN7OHT

echo "Tokens and factory deployed."

# Sort the token addresses alphabetically
if [[ "$TOKEN_ADDR1" < "$TOKEN_ADDR2" ]]; then
    TOKEN_ID1=$TOKEN_ADDR1
    TOKEN_ID2=$TOKEN_ADDR2
else
    TOKEN_ID1=$TOKEN_ADDR2
    TOKEN_ID2=$TOKEN_ADDR1
fi

echo "Initialize factory..."

echo "Install the soroban_token, phoenix_pool and phoenix_stake contracts..."

TOKEN_WASM_HASH=$(soroban contract install \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

# Continue with the rest of the deployments
PAIR_WASM_HASH=$(soroban contract install \
    --wasm phoenix_pool.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

STAKE_WASM_HASH=$(soroban contract install \
    --wasm phoenix_stake.optimized.wasm \
    --source $IDENTITY_STRING \
    --network futurenet)

echo "Token, pair and stake contracts deployed."


echo "Initialize pair using the previously fetched hashes through factory..."

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
    query_pools | jq -r '.[2]')

echo "Pair contract initialized."

echo "Mint both tokens to the admin and provide liquidity..."
soroban contract invoke \
    --id $TOKEN_ADDR2 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $ADMIN_ADDRESS --amount 100000000000

# Provide liquidity in 2:1 ratio to the pool
soroban contract invoke \
    --id $PAIR_ADDR \
    --source $IDENTITY_STRING \
    --network futurenet --fee 10000000 \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 10000000000 --desired_b 50000000000

echo "Liquidity provided."

# Continue with the rest of the commands
echo "Bond tokens to stake contract..."

# STAKE_ADDR=$(soroban contract invoke \
#     --id $PAIR_ADDR \
#     --source $IDENTITY_STRING \
#     --network futurenet --fee 10000000 \
#     -- \
#     query_stake_contract_address | jq -r '.')
#
# # Bond token in stake contract
# soroban contract invoke \
#     --id $STAKE_ADDR \
#     --source $IDENTITY_STRING \
#     --network futurenet \
#     -- \
#     bond --sender $ADMIN_ADDRESS --tokens 70000000000

echo "Tokens bonded."

echo "Mint tokens to the faucet..."

soroban contract invoke \
    --id $TOKEN_ADDR2 \
    --source $IDENTITY_STRING \
    --network futurenet \
    -- \
    mint --to $FAUCET --amount 100000000000

echo "Tokens minted."




echo "Initialization complete!"
echo "Token Contract 1 address: $TOKEN_ADDR1"
echo "Token Contract 2 address: $TOKEN_ADDR2"
echo "Pair Contract address: $PAIR_ADDR"
echo "Stake Contract address: $STAKE_ADDR"
echo "Factory Contract address: $FACTORY_ADDR"
echo "Multihop Contract address: $MULTIHOP"

