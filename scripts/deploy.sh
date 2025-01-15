#!/bin/bash
# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK="testnet"

echo "Build and optimize the contracts...";

make build > /dev/null
cd target/wasm32-unknown-unknown/release

echo "Contracts compiled."
echo "Optimize contracts..."

soroban contract optimize --wasm soroban_token_contract.wasm
soroban contract optimize --wasm phoenix_factory.wasm
soroban contract optimize --wasm phoenix_pool.wasm
soroban contract optimize --wasm phoenix_pool_stable.wasm
soroban contract optimize --wasm phoenix_stake.wasm
soroban contract optimize --wasm phoenix_stake_rewards.wasm
soroban contract optimize --wasm phoenix_multihop.wasm
soroban contract optimize --wasm phoenix_stake_rewards.wasm

echo "Contracts optimized."

# Fetch the admin's address
ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)

echo "Deploy the soroban_token_contract and capture its contract ID hash..."

XLM="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"

TOKEN_ADDR1=$XLM

TOKEN_ADDR2=$(soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    --admin $ADMIN_ADDRESS \
    --decimal 7 \
    --name PHOENIX \
    --symbol PHO
)

echo "PHO Token initialized."

# Sort the token addresses alphabetically
if [[ "$TOKEN_ADDR1" < "$TOKEN_ADDR2" ]]; then
    TOKEN_ID1=$TOKEN_ADDR1
    TOKEN_ID2=$TOKEN_ADDR2
else
    TOKEN_ID1=$TOKEN_ADDR2
    TOKEN_ID2=$TOKEN_ADDR1
fi

echo "Install the soroban_token, multihop, phoenix_pool and phoenix_stake contracts..."

TOKEN_WASM_HASH=$(soroban contract install \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

# Continue with the rest of the deployments
PAIR_WASM_HASH=$(soroban contract install \
    --wasm phoenix_pool.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

STABLE_PAIR_WASM_HASH=$(soroban contract install \
    --wasm phoenix_pool_stable.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

STAKE_WASM_HASH=$(soroban contract install \
    --wasm phoenix_stake.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

STAKE_REWARDS_WASM_HASH=$(soroban contract install \
    --wasm phoenix_stake_rewards.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

MULTIHOP_WASM_HASH=$(soroban contract install \
    --wasm phoenix_multihop.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

echo "Token, pair and stake contracts deployed."

echo "Initialize factory..."

FACTORY_ADDR=$(soroban contract deploy \
    --wasm phoenix_factory.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    --admin $ADMIN_ADDRESS \
    --multihop_wasm_hash $MULTIHOP_WASM_HASH \
    --lp_wasm_hash $PAIR_WASM_HASH \
    --stable_wasm_hash $STABLE_PAIR_WASM_HASH \
    --stake_wasm_hash $STAKE_WASM_HASH \
    --token_wasm_hash $TOKEN_WASM_HASH \
    --whitelisted_accounts "[ \"${ADMIN_ADDRESS}\" ]" \
    --lp_token_decimals 7
)

echo "Factory initialized: " $FACTORY_ADDR

echo "Initialize Multihop..."
MULTIHOP_ADDR=$(soroban contract deploy \
    --wasm phoenix_multihop.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    --admin $ADMIN_ADDRESS \
    --factory $FACTORY_ADDR
)

echo "Multihop initialized: " $MULTIHOP_ADDR


echo "Initialize pair using the previously fetched hashes through factory..."

soroban contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    create_liquidity_pool \
    --sender $ADMIN_ADDRESS \
    --lp_init_info "{ \"admin\": \"${ADMIN_ADDRESS}\", \"swap_fee_bps\": 1000, \"fee_recipient\": \"${ADMIN_ADDRESS}\", \"max_allowed_slippage_bps\": 10000, \"default_slippage_bps\": 3000, \"max_allowed_spread_bps\": 10000, \"max_referral_bps\": 5000, \"token_init_info\": { \"token_a\": \"${TOKEN_ID1}\", \"token_b\": \"${TOKEN_ID2}\" }, \"stake_init_info\": { \"min_bond\": \"100\", \"min_reward\": \"100\", \"max_distributions\": 3, \"manager\": \"${ADMIN_ADDRESS}\", \"max_complexity\": 7 } }" \
    --share_token_name "XLMPHOST" \
    --share_token_symbol "XPST" \
    --pool_type 0 \
    --default_slippage_bps 3000 \
    --max_allowed_fee_bps 10000 

echo "Query XLM/PHO pair address..."

PAIR_ADDR=$(soroban contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 100 \
    -- \
    query_pools | jq -r '.[0]')

echo "Pair contract initialized."

echo "Mint PHO token to the admin and provide liquidity..."
soroban contract invoke \
    --id $TOKEN_ADDR2 \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    mint --to $ADMIN_ADDRESS --amount 10000000000 # 7 decimals, 10k tokens

# Provide liquidity in 2:1 ratio to the pool
soroban contract invoke \
    --id $PAIR_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 10000000 \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 2000000000 --desired_b 2000000000

echo "Liquidity provided."

# Continue with the rest of the commands
echo "Query stake contract address..."

STAKE_ADDR=$(soroban contract invoke \
    --id $PAIR_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 10000000 \
    -- \
    query_stake_contract_address | jq -r '.')

echo "Bond tokens to stake contract..."
# Bond token in stake contract
soroban contract invoke \
    --id $STAKE_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    bond --sender $ADMIN_ADDRESS --tokens 70000000

echo "Tokens bonded."

echo "#############################"

# TOKEN_ADDR2 stays the same - $PHO
TOKEN_ADDR1=$(soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    --admin $ADMIN_ADDRESS \
    --decimal 7 \
    --name USDC \
    --symbol USDC
)


echo "USDC Token initialized."

# Sort the token addresses alphabetically
if [[ "$TOKEN_ADDR1" < "$TOKEN_ADDR2" ]]; then
    TOKEN_ID1=$TOKEN_ADDR1
    TOKEN_ID2=$TOKEN_ADDR2
else
    TOKEN_ID1=$TOKEN_ADDR2
    TOKEN_ID2=$TOKEN_ADDR1
fi

echo "Initialize pair through factory..."

soroban contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    create_liquidity_pool \
    --sender $ADMIN_ADDRESS \
    --lp_init_info "{ \"admin\": \"${ADMIN_ADDRESS}\", \"swap_fee_bps\": 1000, \"fee_recipient\": \"${ADMIN_ADDRESS}\", \"max_allowed_slippage_bps\": 10000, \"default_slippage_bps\": 3000, \"max_allowed_spread_bps\": 10000, \"max_referral_bps\": 5000, \"token_init_info\": { \"token_a\": \"${TOKEN_ID1}\", \"token_b\": \"${TOKEN_ID2}\" }, \"stake_init_info\": { \"min_bond\": \"100\", \"min_reward\": \"100\", \"max_distributions\": 3, \"manager\": \"${ADMIN_ADDRESS}\", \"max_complexity\": 7 } }" \
    --default_slippage_bps 3000 \
    --max_allowed_fee_bps 10000 \
    --share_token_name "XLMPHOST" \
    --share_token_symbol "XPST" \
    --pool_type 0

echo "Query PHO/USDC pair address..."

PAIR_ADDR2=$(soroban contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 100 \
    -- \
    query_pools | jq -r '.[1]')

echo "Pair contract initialized."

echo "Mint PHO & USDC token to the admin and provide liquidity..."
soroban contract invoke \
    --id $TOKEN_ADDR1 \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    mint --to $ADMIN_ADDRESS --amount 10000000000 # 7 decimals, 10k tokens

soroban contract invoke \
    --id $TOKEN_ADDR2 \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    mint --to $ADMIN_ADDRESS --amount 10000000000 # 7 decimals, 10k tokens

# Provide liquidity in 2:1 ratio to the pool
soroban contract invoke \
    --id $PAIR_ADDR2 \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 10000000 \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 6000000000 --desired_b 2000000000

echo "Liquidity provided."

# Continue with the rest of the commands
echo "Query stake contract address..."

STAKE_ADDR2=$(soroban contract invoke \
    --id $PAIR_ADDR2 \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 10000000 \
    -- \
    query_stake_contract_address | jq -r '.')

echo "Bond tokens to stake contract..."
# Bond token in stake contract
soroban contract invoke \
    --id $STAKE_ADDR2 \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    bond --sender $ADMIN_ADDRESS --tokens 70000000

echo "Tokens bonded."

echo "Starting the deployment of stable pool..."

echo "Deploying GBPx and EURc ..."

STABLE_TOKEN_A=$(
soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    --admin $ADMIN_ADDRESS \
    --decimal 7 \
    --name GBPCoin \
    --symbol GBPx
)

STABLE_TOKEN_B=$(
soroban contract deploy \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    --admin $ADMIN_ADDRESS \
    --decimal 7 \
    --name EuroCoin \
    --symbol EURc
)

if [[ "$STABLE_TOKEN_A" < "$STABLE_TOKEN_B" ]]; then
    STABLE_TOKEN_ID1=$STABLE_TOKEN_A
    STABLE_TOKEN_ID2=$STABLE_TOKEN_B
else
    STABLE_TOKEN_ID1=$STABLE_TOKEN_B
    STABLE_TOKEN_ID2=$STABLE_TOKEN_A
fi

echo "Minting GBPx and EURc..."

soroban contract invoke \
    --id $STABLE_TOKEN_ID1 \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    mint --to $ADMIN_ADDRESS --amount 100000000000

soroban contract invoke \
    --id $STABLE_TOKEN_ID2 \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    mint --to $ADMIN_ADDRESS --amount 100000000000

echo "Deploy GBPx/EURc stable pool ..."

soroban contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    create_liquidity_pool \
    --sender $ADMIN_ADDRESS  \
    --lp_init_info "{ \
      \"admin\": \"${ADMIN_ADDRESS}\", \
      \"swap_fee_bps\": 1000, \
      \"fee_recipient\": \"${ADMIN_ADDRESS}\", \
      \"max_allowed_slippage_bps\": 10000, \
      \"default_slippage_bps\": 3000, \
      \"max_allowed_spread_bps\": 10000, \
      \"max_referral_bps\": 5000, \
      \"token_init_info\": { \
        \"token_a\": \"${STABLE_TOKEN_ID1}\", \
        \"token_b\": \"${STABLE_TOKEN_ID2}\" \
      }, \
      \"stake_init_info\": { \
        \"min_bond\": \"100\", \
        \"min_reward\": \"100\", \
        \"max_distributions\": \"3\", \
        \"manager\": \"${ADMIN_ADDRESS}\", \
        \"max_complexity\": 7 \
      } \
    }" \
    --default_slippage_bps 3000 \
    --max_allowed_fee_bps 10000 \
    --share_token_name "GBPEURCST" \
    --share_token_symbol "GEST" \
    --pool_type 1 \
    --amp 50 

echo "Query GBPx/EURc pair address..."

STABLE_PAIR_ADDR=$(soroban contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 100 \
    -- \
    query_pools | jq -r '.[2]')

echo "Providing liquidity to stable pool: " $STABLE_PAIR_ADDR

# temporary using 2 decimals zeros less, when the liquidity pool is fixed we can use regular numbers again
soroban contract invoke \
    --id $STABLE_PAIR_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 10000000 \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 20000000 --desired_b 20000000

echo "Liquidity provided."

echo "#############################"

echo "Deploy and initialize stake_rewards contracts..."

MAX_COMPLEXITY=7
MIN_REWARD=100
MIN_BOND=100

echo "Deploying stake_rewards for the XLM/PHO Stake Contract ($STAKE_ADDR)..."
STAKING_REWARDS_XLM_PHO_ADDR=$(soroban contract deploy \
    --wasm phoenix_stake_rewards.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    --admin "$ADMIN_ADDRESS" \
    --staking_contract "$STAKE_ADDR" \
    --reward_token "$TOKEN_ADDR2" \
    --max_complexity "$MAX_COMPLEXITY" \
    --min_reward "$MIN_REWARD" \
    --min_bond "$MIN_BOND"
)

echo "Staking Rewards Contract for XLM/PHO deployed at address: $STAKING_REWARDS_XLM_PHO_ADDR"

echo "Deploying staking_rewards for the PHO/USDC Stake Contract ($STAKE_ADDR2)..."
STAKING_REWARDS_PHO_USDC_ADDR=$(soroban contract deploy \
    --wasm phoenix_stake_rewards.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    --admin "$ADMIN_ADDRESS" \
    --staking_contract "$STAKE_ADDR2" \
    --reward_token "$TOKEN_ADDR2" \
    --max_complexity "$MAX_COMPLEXITY" \
    --min_reward "$MIN_REWARD" \
    --min_bond "$MIN_BOND"
)

echo "Staking Rewards Contract for PHO/USDC deployed at address: $STAKING_REWARDS_PHO_USDC_ADDR"

echo "#############################"

echo "Initialization complete!"
echo "XLM address: $XLM"
echo "PHO address: $TOKEN_ADDR2"
echo "USDC address: $TOKEN_ADDR1"
echo "XLM/PHO Pair Contract address: $PAIR_ADDR"
echo "XLM/PHO Stake Contract address: $STAKE_ADDR"
echo "PHO/USDC Pair Contract address: $PAIR_ADDR2"
echo "PHO/USDC Stake Contract address: $STAKE_ADDR2"
echo "GBPx/EURc Pair Contract address: $STABLE_PAIR_ADDR"
echo "Factory Contract address: $FACTORY_ADDR"
echo "Multihop Contract address: $MULTIHOP_ADDR"
echo "Staking Rewards Contract for XLM/PHO address: $STAKING_REWARDS_XLM_PHO_ADDR"
echo "Staking Rewards Contract for PHO/USDC address: $STAKING_REWARDS_PHO_USDC_ADDR"
