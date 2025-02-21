#!/bin/bash
set -e

# Check for identity string argument
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

# Configuration
NETWORK="testnet"
IDENTITY="$1"
ADMIN_ADDR=$(soroban keys address $IDENTITY)
DAY_SECONDS=86400

# Cleanup previous deployment
rm -rf .stellar

echo "1. Building and optimizing contracts..."
make build > /dev/null
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_stake.wasm
soroban contract optimize --wasm .artifacts_stake_migration_test/old_phoenix_stake.wasm
echo "Contracts optimized"

echo "2. Deploying old stake contract..."
OLD_STAKE_WASM_HASH=$(soroban contract install \
    --wasm .artifacts_stake_migration_test/old_phoenix_stake.wasm \
    --source $IDENTITY \
    --network $NETWORK)
STAKE_ADDR=$(soroban contract deploy \
    --wasm-hash $OLD_STAKE_WASM_HASH \
    --source $IDENTITY \
    --network $NETWORK)
echo "Old Stake deployed at: $STAKE_ADDR"

echo "3. Deploying LP and Reward tokens..."
LP_TOKEN_ADDR=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.wasm \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    --admin $ADMIN_ADDR \
    --decimal 7 \
    --name LPToken \
    --symbol LPT)
echo "LP TOKEN ADDRESS: $LP_TOKEN_ADDR"

REWARD_TOKEN_ADDR=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.wasm \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    --admin $ADMIN_ADDR \
    --decimal 7 \
    --name RewardToken \
    --symbol RWT
)
echo "REWARD TOKEN ADDRESS : $REWARD_TOKEN_ADDR"

echo "Minting rewards tokens to manager"
soroban contract invoke \
    --id $REWARD_TOKEN_ADDR \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    mint --to $ADMIN_ADDR --amount 100000000000000

echo "4. Initializing old stake contract..."
soroban contract invoke \
    --id $STAKE_ADDR \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    initialize \
    --admin $ADMIN_ADDR \
    --lp_token $LP_TOKEN_ADDR \
    --min_bond 100 \
    --min_reward 50 \
    --manager $ADMIN_ADDR \
    --owner $ADMIN_ADDR \
    --max_complexity 7


echo "5. Creating test users..."
USER1=$(soroban keys address user1 2>/dev/null || { soroban keys generate user1 --network $NETWORK --fund >/dev/null 2>&1; soroban keys address user1; })
USER2=$(soroban keys address user2 2>/dev/null || { soroban keys generate user2 --network $NETWORK --fund >/dev/null 2>&1; soroban keys address user2; })
USER3=$(soroban keys address user3 2>/dev/null || { soroban keys generate user3 --network $NETWORK --fund >/dev/null 2>&1; soroban keys address user3; })
NEW_USER=$(soroban keys address new_user 2>/dev/null || { soroban keys generate new_user --network $NETWORK --fund >/dev/null 2>&1; soroban keys address new_user; })

echo "USER1: $USER1"
echo "USER2: $USER2"
echo "USER3: $USER3"
echo "NEW_USER: $NEW_USER"

for user in $USER1 $USER2 $USER3 $NEW_USER; do
    echo "🤑 Will mint to $user"
    soroban contract invoke \
        --id $LP_TOKEN_ADDR \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        mint --to $user --amount 10000000000000
done

echo "6. Creating distribution flow..."
soroban contract invoke \
    --id $STAKE_ADDR \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    create_distribution_flow \
    --sender $ADMIN_ADDR \
    --asset $REWARD_TOKEN_ADDR

echo "7. Bonding tokens..."
bond_tokens() {
    local user=$1
    local amount=$2
    soroban contract invoke \
        --id $STAKE_ADDR \
        --source $user \
        --network $NETWORK \
        -- \
        bond \
        --sender $(soroban keys secret $user) \
        --tokens $amount
}

bond_tokens user1 10000000000 # 1_000 tokens
bond_tokens user2 20000000000 # 2_000 tokens
bond_tokens user3 15000000000 # 1_500 tokens

echo "8. Verifying initial stakes..."
verify_stake() {
    local user=$1
    local expected=$2
    local stakes=$(soroban contract invoke \
        --id $STAKE_ADDR \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        query_staked \
        --address $user | jq -r '.stakes[0].stake')
    
    if [ $((stakes)) -ne $((expected)) ]; then
        echo "Stake verification failed for $user: expected $expected, got $stakes"
        exit 1
    fi
}

verify_stake user1 10000000000
verify_stake user2 20000000000
verify_stake user3 15000000000

echo "9. Distributing initial rewards..."
# soroban contract invoke \
#     --id $REWARD_TOKEN_ADDR \
#     --source $IDENTITY \
#     --network $NETWORK \
#     -- \
#     mint --to $STAKE_ADDR --amount 100000000000000

soroban contract invoke \
    --id $STAKE_ADDR \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    distribute_rewards \
    --sender $ADMIN_ADDR \
    --amount 10000000 \
    --reward_token $REWARD_TOKEN_ADDR

echo "10. Verifying initial rewards..."
verify_rewards() {
    local user=$1
    local expected=$2
    local rewards=$(soroban contract invoke \
        --id $STAKE_ADDR \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        query_withdrawable_rewards \
        --user $user | jq '.rewards[0].reward_amount')
    
    if [ $((rewards)) -ne $((expected)) ]; then
        echo "Reward verification failed for $user: expected $expected, got $rewards"
        exit 1
    fi
}

verify_rewards user1 1111111
verify_rewards user2 2222222
verify_rewards user3 1666666

echo "11. Upgrading stake contract..."
NEW_STAKE_WASM_HASH=$(soroban contract install \
    --wasm target/wasm32-unknown-unknown/release/phoenix_stake.wasm \
    --source $IDENTITY \
    --network $NETWORK)

soroban contract invoke \
    --id $STAKE_ADDR \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    update \
    --new_wasm_hash $NEW_STAKE_WASM_HASH

echo "12. Migrating distributions..."
soroban contract invoke \
    --id $STAKE_ADDR \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    migrate_distributions

echo "13. Withdrawing rewards..."
withdraw_rewards() {
    local user=$1
    soroban contract invoke \
        --id $STAKE_ADDR \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        withdraw_rewards_deprecated \
        --sender $(soroban keys secret $user)
}

withdraw_rewards user1
withdraw_rewards user2
withdraw_rewards user3

echo "14. Verifying withdrawn balances..."
verify_balance() {
    local user=$1
    local expected=$2
    local balance=$(soroban contract invoke \
        --id $REWARD_TOKEN_ADDR \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        balance --id $user | jq - r '.')
    
    if [ "$balance" -ne $expected ]; then
        echo "Balance verification failed for $user: expected $expected, got $balance"
        exit 1
    fi
}

verify_balance user1 1111111
verify_balance user2 2222222
verify_balance user3 1666666

echo "15. Unbonding tokens with deprecated API..."
unbond_tokens() {
    local user=$1
    local amount=$2

    STAKE_TIMESTAMP=$(soroban contract invoke \
        --id $STAKE_ADDR \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        query_staked --address $user | jq -r '.stakes[0].stake_timestamp')

    soroban contract invoke \
        --id $STAKE_ADDR \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        unbond_deprecated \
        --sender $(soroban keys secret $user) \
        --stake_amount $amount \
        --stake_timestamp $STAKE_TIMESTAMP
}


unbond_tokens user1 10000000000
unbond_tokens user2 20000000000
unbond_tokens user3 15000000000

echo "16. Verifying empty stakes..."
verify_empty_stakes() {
    local user=$1
    local stakes=$(soroban contract invoke \
        --id $STAKE_ADDR \
        --source $IDENTITY \
        --network $NETWORK \
        -- \
        query_staked --address $user | jq -r '.stakes | length')
    
    if [ "$stakes" -ne 0 ]; then
        echo "Unbond failed for $user, stakes remaining: $stakes"
        exit 1
    fi
}

verify_empty_stakes user1
verify_empty_stakes user2
verify_empty_stakes user3

echo "17. New user interaction..."
bond_tokens new_user 10000000000

echo "18. Final rewards check..."
soroban contract invoke \
    --id $STAKE_ADDR \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    distribute_rewards

final_rewards=$(soroban contract invoke \
    --id $REWARD_TOKEN_ADDR \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    balance --id $NEW_USER | jq -r '.')

## since we're in testnet and we cannot forward time to generate rewards we can just assume that there are 0 rewards after bonding
if [ $((final_rewards)) -ne 0 ]; then
    echo "Final rewards check failed: expected 0 got $final_rewards"
    exit 1
fi

echo "All tests completed successfully!"
