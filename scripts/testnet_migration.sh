#!/bin/bash

# Testnet deployment script for Phoenix Protocol upgrade testing
# This script simulates the mainnet migration path:
# 1. Deploy all contracts from the earliest commit (factory commit 77742b01)
# 2. Upgrade pools and stakes to their intermediate versions (matching current network)
# 3. Final migration to latest main branch versions

set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

IDENTITY_STRING=$1
ADMIN_ADDRESS=$(stellar keys address $IDENTITY_STRING)
NETWORK="testnet"

echo "=== Phoenix Protocol Testnet Migration Testing ==="
echo "Admin Address: $ADMIN_ADDRESS"
echo "Network: $NETWORK"

# Build current main branch first (for final migration)
echo "Building current main branch contracts..."
make build > /dev/null
echo "Main branch contracts compiled."

# Store current branch to return to
CURRENT_BRANCH=$(git branch --show-current)

# === PHASE 1: Deploy all contracts from earliest commit ===
echo ""
echo "=== PHASE 1: DEPLOYING FROM EARLIEST COMMIT (77742b01) ==="

# All contracts from the same commit (77742b01) for initial deployment
echo "Building all contracts from commit 77742b01..."
git checkout 77742b01fd65cce26cbe1ee47f229fb52d889a43

make build > /dev/null
echo "Old contracts compiled from commit 77742b01."

# Copy the old versions
cp target/wasm32-unknown-unknown/release/phoenix_factory.wasm .temp_old_factory.wasm
cp target/wasm32-unknown-unknown/release/phoenix_multihop.wasm .temp_old_multihop.wasm
cp target/wasm32-unknown-unknown/release/phoenix_pool.wasm .temp_old_pool.wasm
cp target/wasm32-unknown-unknown/release/phoenix_stake.wasm .temp_old_stake.wasm

# Now build intermediate versions for phase 2 upgrades
echo "Building intermediate contract versions..."

# Pool version 4a0b6e6c (most pools)
git checkout 4a0b6e6c8d8a25a7f2956799bea8a1eddaa5dcb6
make -C contracts/pool build > /dev/null
cp target/wasm32-unknown-unknown/release/phoenix_pool.wasm .temp_pool_4a0b6e6c.wasm

# Pool version 0e811ce4 (PHO-USDC hotfix)
git checkout 0e811ce4ef41185bfb9718c798ecc6505aa224d1
make -C contracts/pool build > /dev/null
cp target/wasm32-unknown-unknown/release/phoenix_pool.wasm .temp_pool_0e811ce4.wasm

# Stake version bc01344 (most stakes)
git checkout bc01344
make -C contracts/stake build > /dev/null
cp target/wasm32-unknown-unknown/release/phoenix_stake.wasm .temp_stake_bc01344.wasm

# Stake version 612f44f (March 2025 stakes)
git checkout 612f44f
make -C contracts/stake build > /dev/null
cp target/wasm32-unknown-unknown/release/phoenix_stake.wasm .temp_stake_612f44f.wasm

# Stake version e4f767d (PHO-USDC stake)
git checkout e4f767d
make -C contracts/stake build > /dev/null
cp target/wasm32-unknown-unknown/release/phoenix_stake.wasm .temp_stake_e4f767d.wasm

# Return to current branch
git checkout $CURRENT_BRANCH

# Optimize all WASM files
echo "Optimizing WASM files..."
stellar contract optimize --wasm .temp_old_factory.wasm
stellar contract optimize --wasm .temp_old_multihop.wasm
stellar contract optimize --wasm .temp_old_pool.wasm
stellar contract optimize --wasm .temp_old_stake.wasm
stellar contract optimize --wasm .temp_pool_4a0b6e6c.wasm
stellar contract optimize --wasm .temp_pool_0e811ce4.wasm
stellar contract optimize --wasm .temp_stake_bc01344.wasm
stellar contract optimize --wasm .temp_stake_612f44f.wasm
stellar contract optimize --wasm .temp_stake_e4f767d.wasm

# Optimize current versions for final migration
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_factory.wasm
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_multihop.wasm
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_pool.wasm
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_stake.wasm
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.wasm

echo "Uploading WASM hashes..."

# Upload old versions (phase 1)
OLD_FACTORY_HASH=$(stellar contract upload --wasm .temp_old_factory.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
OLD_MULTIHOP_HASH=$(stellar contract upload --wasm .temp_old_multihop.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
OLD_POOL_HASH=$(stellar contract upload --wasm .temp_old_pool.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
OLD_STAKE_HASH=$(stellar contract upload --wasm .temp_old_stake.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)

# Upload intermediate versions (phase 2)
POOL_4A0B6E6C_HASH=$(stellar contract upload --wasm .temp_pool_4a0b6e6c.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
POOL_0E811CE4_HASH=$(stellar contract upload --wasm .temp_pool_0e811ce4.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
STAKE_BC01344_HASH=$(stellar contract upload --wasm .temp_stake_bc01344.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
STAKE_612F44F_HASH=$(stellar contract upload --wasm .temp_stake_612f44f.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
STAKE_E4F767D_HASH=$(stellar contract upload --wasm .temp_stake_e4f767d.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)

# Upload latest versions (phase 3)
LATEST_FACTORY_HASH=$(stellar contract upload --wasm target/wasm32-unknown-unknown/release/phoenix_factory.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
LATEST_MULTIHOP_HASH=$(stellar contract upload --wasm target/wasm32-unknown-unknown/release/phoenix_multihop.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
LATEST_POOL_HASH=$(stellar contract upload --wasm target/wasm32-unknown-unknown/release/phoenix_pool.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
LATEST_STAKE_HASH=$(stellar contract upload --wasm target/wasm32-unknown-unknown/release/phoenix_stake.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)
TOKEN_HASH=$(stellar contract upload --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.optimized.wasm --source $IDENTITY_STRING --network $NETWORK 2>/dev/null)

echo "Phase 1 - Old Contract Hashes (commit 77742b01):"
echo "Factory: $OLD_FACTORY_HASH"
echo "Multihop: $OLD_MULTIHOP_HASH"
echo "Pool: $OLD_POOL_HASH"
echo "Stake: $OLD_STAKE_HASH"
echo ""
echo "Phase 2 - Intermediate Hashes:"
echo "Pool 4a0b6e6c: $POOL_4A0B6E6C_HASH"
echo "Pool 0e811ce4: $POOL_0E811CE4_HASH"
echo "Stake bc01344: $STAKE_BC01344_HASH"
echo "Stake 612f44f: $STAKE_612F44F_HASH"
echo "Stake e4f767d: $STAKE_E4F767D_HASH"
echo ""
echo "Phase 3 - Latest Hashes:"
echo "Factory: $LATEST_FACTORY_HASH"
echo "Multihop: $LATEST_MULTIHOP_HASH"
echo "Pool: $LATEST_POOL_HASH"
echo "Stake: $LATEST_STAKE_HASH"
echo "Token: $TOKEN_HASH"

# Deploy factory with old version
echo ""
echo "Deploying factory with old version..."
FACTORY_ADDR=$(stellar contract deploy --wasm-hash $OLD_FACTORY_HASH --source $IDENTITY_STRING --network $NETWORK)

echo "Initializing factory..."
stellar contract invoke \
  --id $FACTORY_ADDR \
  --source $IDENTITY_STRING \
  --network $NETWORK \
  -- \
  initialize \
  --admin $ADMIN_ADDRESS \
  --multihop_wasm_hash $OLD_MULTIHOP_HASH \
  --lp_wasm_hash $OLD_POOL_HASH \
  --stable_wasm_hash $OLD_POOL_HASH \
  --stake_wasm_hash $OLD_STAKE_HASH \
  --token_wasm_hash $TOKEN_HASH \
  --whitelisted_accounts "[ \"$ADMIN_ADDRESS\" ]" \
  --lp_token_decimals 7

echo "Factory deployed and initialized at: $FACTORY_ADDR"

# Deploy multihop with old version
echo "Deploying multihop with old version..."
MULTIHOP_ADDR=$(stellar contract deploy --wasm-hash $OLD_MULTIHOP_HASH --source $IDENTITY_STRING --network $NETWORK)

stellar contract invoke \
  --id $MULTIHOP_ADDR \
  --source $IDENTITY_STRING \
  --network $NETWORK \
  -- \
  initialize \
  --admin $ADMIN_ADDRESS \
  --factory $FACTORY_ADDR

echo "Multihop deployed and initialized at: $MULTIHOP_ADDR"

# Create test tokens
echo "Creating test tokens..."

# Create tokens using mainnet-like addresses from CLAUDE.md
XLM_TOKEN=$(stellar contract deploy --wasm-hash $TOKEN_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke --id $XLM_TOKEN --source $IDENTITY_STRING --network $NETWORK -- initialize --admin $ADMIN_ADDRESS --decimal 7 --name "XLM" --symbol "XLM"

USDC_TOKEN=$(stellar contract deploy --wasm-hash $TOKEN_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke --id $USDC_TOKEN --source $IDENTITY_STRING --network $NETWORK -- initialize --admin $ADMIN_ADDRESS --decimal 7 --name "USDC" --symbol "USDC"

PHO_TOKEN=$(stellar contract deploy --wasm-hash $TOKEN_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke --id $PHO_TOKEN --source $IDENTITY_STRING --network $NETWORK -- initialize --admin $ADMIN_ADDRESS --decimal 7 --name "PHO" --symbol "PHO"

EURC_TOKEN=$(stellar contract deploy --wasm-hash $TOKEN_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke --id $EURC_TOKEN --source $IDENTITY_STRING --network $NETWORK -- initialize --admin $ADMIN_ADDRESS --decimal 7 --name "EURC" --symbol "EURC"

VEUR_TOKEN=$(stellar contract deploy --wasm-hash $TOKEN_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke --id $VEUR_TOKEN --source $IDENTITY_STRING --network $NETWORK -- initialize --admin $ADMIN_ADDRESS --decimal 7 --name "VEUR" --symbol "VEUR"

echo "Tokens created:"
echo "XLM: $XLM_TOKEN"
echo "USDC: $USDC_TOKEN"
echo "PHO: $PHO_TOKEN"
echo "EURC: $EURC_TOKEN"
echo "VEUR: $VEUR_TOKEN"

# Generate test users or use existing
if ! stellar keys address pool_user > /dev/null 2>&1; then
    stellar keys generate pool_user --network $NETWORK --fund
fi
POOL_USER=$(stellar keys address pool_user)

echo "Test user: $POOL_USER"

# Mint tokens to user
echo "Minting tokens to test user..."
stellar contract invoke --id $XLM_TOKEN --source $IDENTITY_STRING --network $NETWORK -- mint --to $POOL_USER --amount 1000000000000000
stellar contract invoke --id $USDC_TOKEN --source $IDENTITY_STRING --network $NETWORK -- mint --to $POOL_USER --amount 1000000000000000
stellar contract invoke --id $PHO_TOKEN --source $IDENTITY_STRING --network $NETWORK -- mint --to $POOL_USER --amount 1000000000000000
stellar contract invoke --id $EURC_TOKEN --source $IDENTITY_STRING --network $NETWORK -- mint --to $POOL_USER --amount 1000000000000000
stellar contract invoke --id $VEUR_TOKEN --source $IDENTITY_STRING --network $NETWORK -- mint --to $POOL_USER --amount 1000000000000000

# Deploy pools with old versions and specified liquidity amounts
echo ""
echo "=== DEPLOYING POOLS WITH OLD VERSIONS ==="

# Helper function to sort tokens
get_sorted_tokens() {
    local token1=$1
    local token2=$2
    if [[ "$token1" < "$token2" ]]; then
        echo "$token1 $token2"
    else
        echo "$token2 $token1"
    fi
}

# Pool 1: XLM-USDC (will upgrade to 4a0b6e6c, stake to 612f44f)
echo "Creating XLM-USDC pool..."
SORTED_TOKENS_1=$(get_sorted_tokens $XLM_TOKEN $USDC_TOKEN)
TOKEN_A_1=$(echo $SORTED_TOKENS_1 | cut -d' ' -f1)
TOKEN_B_1=$(echo $SORTED_TOKENS_1 | cut -d' ' -f2)

POOL_1=$(stellar contract deploy --wasm-hash $OLD_POOL_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke \
  --id $POOL_1 \
  --source $IDENTITY_STRING \
  --network $NETWORK \
  -- \
  initialize \
  --stake_wasm_hash $OLD_STAKE_HASH \
  --token_wasm_hash $TOKEN_HASH \
  --lp_init_info "{
    \"admin\": \"$ADMIN_ADDRESS\",
    \"swap_fee_bps\": 100,
    \"fee_recipient\": \"$ADMIN_ADDRESS\",
    \"max_allowed_slippage_bps\": 5000,
    \"default_slippage_bps\": 2500,
    \"max_allowed_spread_bps\": 10000,
    \"max_referral_bps\": 5000,
    \"token_init_info\": {
      \"token_a\": \"$TOKEN_A_1\",
      \"token_b\": \"$TOKEN_B_1\"
    },
    \"stake_init_info\": {
      \"min_bond\": \"100\",
      \"min_reward\": \"50\",
      \"manager\": \"$ADMIN_ADDRESS\",
      \"max_complexity\": 7
    }
  }" \
  --factory_addr $FACTORY_ADDR \
  --share_token_decimals 7 \
  --share_token_name "XLM-USDC LP" \
  --share_token_symbol "XLMUSDC" \
  --default_slippage_bps 100 \
  --max_allowed_fee_bps 1000

# Provide liquidity: 64139988753895 XLM, 17449955004640 USDC
if [[ "$TOKEN_A_1" == "$XLM_TOKEN" ]]; then
    DESIRED_A=64139988753895
    DESIRED_B=17449955004640
else
    DESIRED_A=17449955004640
    DESIRED_B=64139988753895
fi

stellar contract invoke \
  --id $POOL_1 \
  --source pool_user \
  --network $NETWORK \
  -- \
  provide_liquidity \
  --sender $POOL_USER \
  --desired_a $DESIRED_A \
  --min_a $DESIRED_A \
  --desired_b $DESIRED_B \
  --min_b $DESIRED_B

echo "XLM-USDC Pool: $POOL_1"

# Pool 2: XLM-PHO (will upgrade to 4a0b6e6c, stake to 612f44f)
echo "Creating XLM-PHO pool..."
SORTED_TOKENS_2=$(get_sorted_tokens $XLM_TOKEN $PHO_TOKEN)
TOKEN_A_2=$(echo $SORTED_TOKENS_2 | cut -d' ' -f1)
TOKEN_B_2=$(echo $SORTED_TOKENS_2 | cut -d' ' -f2)

POOL_2=$(stellar contract deploy --wasm-hash $OLD_POOL_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke \
  --id $POOL_2 \
  --source $IDENTITY_STRING \
  --network $NETWORK \
  -- \
  initialize \
  --stake_wasm_hash $OLD_STAKE_HASH \
  --token_wasm_hash $TOKEN_HASH \
  --lp_init_info "{
    \"admin\": \"$ADMIN_ADDRESS\",
    \"swap_fee_bps\": 100,
    \"fee_recipient\": \"$ADMIN_ADDRESS\",
    \"max_allowed_slippage_bps\": 5000,
    \"default_slippage_bps\": 2500,
    \"max_allowed_spread_bps\": 10000,
    \"max_referral_bps\": 5000,
    \"token_init_info\": {
      \"token_a\": \"$TOKEN_A_2\",
      \"token_b\": \"$TOKEN_B_2\"
    },
    \"stake_init_info\": {
      \"min_bond\": \"100\",
      \"min_reward\": \"50\",
      \"manager\": \"$ADMIN_ADDRESS\",
      \"max_complexity\": 7
    }
  }" \
  --factory_addr $FACTORY_ADDR \
  --share_token_decimals 7 \
  --share_token_name "XLM-PHO LP" \
  --share_token_symbol "XLMPHO" \
  --default_slippage_bps 100 \
  --max_allowed_fee_bps 1000

# Provide liquidity: 757675338772 XLM, 1125936632179 PHO
if [[ "$TOKEN_A_2" == "$XLM_TOKEN" ]]; then
    DESIRED_A=757675338772
    DESIRED_B=1125936632179
else
    DESIRED_A=1125936632179
    DESIRED_B=757675338772
fi

stellar contract invoke \
  --id $POOL_2 \
  --source pool_user \
  --network $NETWORK \
  -- \
  provide_liquidity \
  --sender $POOL_USER \
  --desired_a $DESIRED_A \
  --min_a $DESIRED_A \
  --desired_b $DESIRED_B \
  --min_b $DESIRED_B

echo "XLM-PHO Pool: $POOL_2"

# Pool 3: XLM-EURC (will upgrade to 4a0b6e6c, stake to bc01344)
echo "Creating XLM-EURC pool..."
SORTED_TOKENS_3=$(get_sorted_tokens $XLM_TOKEN $EURC_TOKEN)
TOKEN_A_3=$(echo $SORTED_TOKENS_3 | cut -d' ' -f1)
TOKEN_B_3=$(echo $SORTED_TOKENS_3 | cut -d' ' -f2)

POOL_3=$(stellar contract deploy --wasm-hash $OLD_POOL_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke \
  --id $POOL_3 \
  --source $IDENTITY_STRING \
  --network $NETWORK \
  -- \
  initialize \
  --stake_wasm_hash $OLD_STAKE_HASH \
  --token_wasm_hash $TOKEN_HASH \
  --lp_init_info "{
    \"admin\": \"$ADMIN_ADDRESS\",
    \"swap_fee_bps\": 100,
    \"fee_recipient\": \"$ADMIN_ADDRESS\",
    \"max_allowed_slippage_bps\": 5000,
    \"default_slippage_bps\": 2500,
    \"max_allowed_spread_bps\": 10000,
    \"max_referral_bps\": 5000,
    \"token_init_info\": {
      \"token_a\": \"$TOKEN_A_3\",
      \"token_b\": \"$TOKEN_B_3\"
    },
    \"stake_init_info\": {
      \"min_bond\": \"100\",
      \"min_reward\": \"50\",
      \"manager\": \"$ADMIN_ADDRESS\",
      \"max_complexity\": 7
    }
  }" \
  --factory_addr $FACTORY_ADDR \
  --share_token_decimals 7 \
  --share_token_name "XLM-EURC LP" \
  --share_token_symbol "XLMEURC" \
  --default_slippage_bps 100 \
  --max_allowed_fee_bps 1000

# Provide liquidity: 115142998193 XLM, 27598017336 EURC
if [[ "$TOKEN_A_3" == "$XLM_TOKEN" ]]; then
    DESIRED_A=115142998193
    DESIRED_B=27598017336
else
    DESIRED_A=27598017336
    DESIRED_B=115142998193
fi

stellar contract invoke \
  --id $POOL_3 \
  --source pool_user \
  --network $NETWORK \
  -- \
  provide_liquidity \
  --sender $POOL_USER \
  --desired_a $DESIRED_A \
  --min_a $DESIRED_A \
  --desired_b $DESIRED_B \
  --min_b $DESIRED_B

echo "XLM-EURC Pool: $POOL_3"

# Pool 4: USDC-VEUR (will upgrade to 4a0b6e6c, stake to bc01344)
echo "Creating USDC-VEUR pool..."
SORTED_TOKENS_4=$(get_sorted_tokens $USDC_TOKEN $VEUR_TOKEN)
TOKEN_A_4=$(echo $SORTED_TOKENS_4 | cut -d' ' -f1)
TOKEN_B_4=$(echo $SORTED_TOKENS_4 | cut -d' ' -f2)

POOL_4=$(stellar contract deploy --wasm-hash $OLD_POOL_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke \
  --id $POOL_4 \
  --source $IDENTITY_STRING \
  --network $NETWORK \
  -- \
  initialize \
  --stake_wasm_hash $OLD_STAKE_HASH \
  --token_wasm_hash $TOKEN_HASH \
  --lp_init_info "{
    \"admin\": \"$ADMIN_ADDRESS\",
    \"swap_fee_bps\": 100,
    \"fee_recipient\": \"$ADMIN_ADDRESS\",
    \"max_allowed_slippage_bps\": 5000,
    \"default_slippage_bps\": 2500,
    \"max_allowed_spread_bps\": 10000,
    \"max_referral_bps\": 5000,
    \"token_init_info\": {
      \"token_a\": \"$TOKEN_A_4\",
      \"token_b\": \"$TOKEN_B_4\"
    },
    \"stake_init_info\": {
      \"min_bond\": \"100\",
      \"min_reward\": \"50\",
      \"manager\": \"$ADMIN_ADDRESS\",
      \"max_complexity\": 7
    }
  }" \
  --factory_addr $FACTORY_ADDR \
  --share_token_decimals 7 \
  --share_token_name "USDC-VEUR LP" \
  --share_token_symbol "USDCVEUR" \
  --default_slippage_bps 100 \
  --max_allowed_fee_bps 1000

# Provide liquidity: 49791689351 USDC, 44946917323 VEUR
if [[ "$TOKEN_A_4" == "$USDC_TOKEN" ]]; then
    DESIRED_A=49791689351
    DESIRED_B=44946917323
else
    DESIRED_A=44946917323
    DESIRED_B=49791689351
fi

stellar contract invoke \
  --id $POOL_4 \
  --source pool_user \
  --network $NETWORK \
  -- \
  provide_liquidity \
  --sender $POOL_USER \
  --desired_a $DESIRED_A \
  --min_a $DESIRED_A \
  --desired_b $DESIRED_B \
  --min_b $DESIRED_B

echo "USDC-VEUR Pool: $POOL_4"

# Pool 5: PHO-USDC (will upgrade to 0e811ce4 hotfix, stake to e4f767d)
echo "Creating PHO-USDC pool..."
SORTED_TOKENS_5=$(get_sorted_tokens $PHO_TOKEN $USDC_TOKEN)
TOKEN_A_5=$(echo $SORTED_TOKENS_5 | cut -d' ' -f1)
TOKEN_B_5=$(echo $SORTED_TOKENS_5 | cut -d' ' -f2)

POOL_5=$(stellar contract deploy --wasm-hash $OLD_POOL_HASH --source $IDENTITY_STRING --network $NETWORK)
stellar contract invoke \
  --id $POOL_5 \
  --source $IDENTITY_STRING \
  --network $NETWORK \
  -- \
  initialize \
  --stake_wasm_hash $OLD_STAKE_HASH \
  --token_wasm_hash $TOKEN_HASH \
  --lp_init_info "{
    \"admin\": \"$ADMIN_ADDRESS\",
    \"swap_fee_bps\": 100,
    \"fee_recipient\": \"$ADMIN_ADDRESS\",
    \"max_allowed_slippage_bps\": 5000,
    \"default_slippage_bps\": 2500,
    \"max_allowed_spread_bps\": 10000,
    \"max_referral_bps\": 5000,
    \"token_init_info\": {
      \"token_a\": \"$TOKEN_A_5\",
      \"token_b\": \"$TOKEN_B_5\"
    },
    \"stake_init_info\": {
      \"min_bond\": \"100\",
      \"min_reward\": \"50\",
      \"manager\": \"$ADMIN_ADDRESS\",
      \"max_complexity\": 7
    }
  }" \
  --factory_addr $FACTORY_ADDR \
  --share_token_decimals 7 \
  --share_token_name "PHO-USDC LP" \
  --share_token_symbol "PHOUSDC" \
  --default_slippage_bps 100 \
  --max_allowed_fee_bps 1000

# Add reasonable liquidity for PHO-USDC
stellar contract invoke \
  --id $POOL_5 \
  --source pool_user \
  --network $NETWORK \
  -- \
  provide_liquidity \
  --sender $POOL_USER \
  --desired_a 1000000000000 \
  --min_a 1000000000000 \
  --desired_b 1000000000000 \
  --min_b 1000000000000 > /dev/null 2>&1

echo "PHO-USDC Pool: $POOL_5"

# Get stake addresses from pools
echo "Getting stake addresses from pools..."
POOL_1_INFO=$(stellar contract invoke --id $POOL_1 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory)
STAKE_1=$(echo "$POOL_1_INFO" | jq -r '.pool_response.stake_address')

POOL_2_INFO=$(stellar contract invoke --id $POOL_2 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory)
STAKE_2=$(echo "$POOL_2_INFO" | jq -r '.pool_response.stake_address')

POOL_3_INFO=$(stellar contract invoke --id $POOL_3 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory)
STAKE_3=$(echo "$POOL_3_INFO" | jq -r '.pool_response.stake_address')

POOL_4_INFO=$(stellar contract invoke --id $POOL_4 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory)
STAKE_4=$(echo "$POOL_4_INFO" | jq -r '.pool_response.stake_address')

POOL_5_INFO=$(stellar contract invoke --id $POOL_5 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory)
STAKE_5=$(echo "$POOL_5_INFO" | jq -r '.pool_response.stake_address')

echo "Stake addresses:"
echo "XLM-USDC Stake: $STAKE_1"
echo "XLM-PHO Stake: $STAKE_2"
echo "XLM-EURC Stake: $STAKE_3"
echo "USDC-VEUR Stake: $STAKE_4"
echo "PHO-USDC Stake: $STAKE_5"

# Stake some LP tokens to simulate real conditions
echo ""
echo "=== STAKING LP TOKENS ==="

LP_1=$(echo "$POOL_1_INFO" | jq -r '.pool_response.asset_lp_share.address')
LP_2=$(echo "$POOL_2_INFO" | jq -r '.pool_response.asset_lp_share.address')
LP_3=$(echo "$POOL_3_INFO" | jq -r '.pool_response.asset_lp_share.address')
LP_4=$(echo "$POOL_4_INFO" | jq -r '.pool_response.asset_lp_share.address')
LP_5=$(echo "$POOL_5_INFO" | jq -r '.pool_response.asset_lp_share.address')

# Stake 50% of LP tokens
echo "Staking LP tokens..."
LP_1_BALANCE=$(stellar contract invoke --id $LP_1 --source $IDENTITY_STRING --network $NETWORK -- balance --id $POOL_USER | tr -d '"')
LP_1_STAKE_AMOUNT=$((LP_1_BALANCE / 2))
stellar contract invoke --id $STAKE_1 --source pool_user --network $NETWORK -- bond --sender $POOL_USER --tokens $LP_1_STAKE_AMOUNT

LP_2_BALANCE=$(stellar contract invoke --id $LP_2 --source $IDENTITY_STRING --network $NETWORK -- balance --id $POOL_USER | tr -d '"')
LP_2_STAKE_AMOUNT=$((LP_2_BALANCE / 2))
stellar contract invoke --id $STAKE_2 --source pool_user --network $NETWORK -- bond --sender $POOL_USER --tokens $LP_2_STAKE_AMOUNT

LP_3_BALANCE=$(stellar contract invoke --id $LP_3 --source $IDENTITY_STRING --network $NETWORK -- balance --id $POOL_USER | tr -d '"')
LP_3_STAKE_AMOUNT=$((LP_3_BALANCE / 2))
stellar contract invoke --id $STAKE_3 --source pool_user --network $NETWORK -- bond --sender $POOL_USER --tokens $LP_3_STAKE_AMOUNT

LP_4_BALANCE=$(stellar contract invoke --id $LP_4 --source $IDENTITY_STRING --network $NETWORK -- balance --id $POOL_USER | tr -d '"')
LP_4_STAKE_AMOUNT=$((LP_4_BALANCE / 2))
stellar contract invoke --id $STAKE_4 --source pool_user --network $NETWORK -- bond --sender $POOL_USER --tokens $LP_4_STAKE_AMOUNT

LP_5_BALANCE=$(stellar contract invoke --id $LP_5 --source $IDENTITY_STRING --network $NETWORK -- balance --id $POOL_USER | tr -d '"')
LP_5_STAKE_AMOUNT=$((LP_5_BALANCE / 2))
stellar contract invoke --id $STAKE_5 --source pool_user --network $NETWORK -- bond --sender $POOL_USER --tokens $LP_5_STAKE_AMOUNT

echo "LP tokens staked successfully"

# === PHASE 2: Upgrade to intermediate versions ===
echo ""
echo "=== PHASE 2: UPGRADING TO INTERMEDIATE VERSIONS (SIMULATING NETWORK STATE) ==="

echo "Upgrading pools to their intermediate versions..."

# Pool 1 (XLM-USDC): upgrade to 4a0b6e6c
echo "Upgrading XLM-USDC pool to hash 4a0b6e6c..."
stellar contract invoke --id $POOL_1 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $POOL_4A0B6E6C_HASH

# Pool 2 (XLM-PHO): upgrade to 4a0b6e6c
echo "Upgrading XLM-PHO pool to hash 4a0b6e6c..."
stellar contract invoke --id $POOL_2 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $POOL_4A0B6E6C_HASH

# Pool 3 (XLM-EURC): upgrade to 4a0b6e6c
echo "Upgrading XLM-EURC pool to hash 4a0b6e6c..."
stellar contract invoke --id $POOL_3 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $POOL_4A0B6E6C_HASH

# Pool 4 (USDC-VEUR): upgrade to 4a0b6e6c
echo "Upgrading USDC-VEUR pool to hash 4a0b6e6c..."
stellar contract invoke --id $POOL_4 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $POOL_4A0B6E6C_HASH

# Pool 5 (PHO-USDC): upgrade to 0e811ce4 (hotfix)
echo "Upgrading PHO-USDC pool to hash 0e811ce4 (hotfix)..."
stellar contract invoke --id $POOL_5 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $POOL_0E811CE4_HASH

echo "Upgrading stakes to their intermediate versions..."

# Stakes 1&2 (XLM-USDC, XLM-PHO): upgrade to 612f44f (March 2025)
echo "Upgrading XLM-USDC and XLM-PHO stakes to hash 612f44f..."
stellar contract invoke --id $STAKE_1 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $STAKE_612F44F_HASH
stellar contract invoke --id $STAKE_2 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $STAKE_612F44F_HASH

# Stakes 3&4 (XLM-EURC, USDC-VEUR): upgrade to bc01344
echo "Upgrading XLM-EURC and USDC-VEUR stakes to hash bc01344..."
stellar contract invoke --id $STAKE_3 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $STAKE_BC01344_HASH
stellar contract invoke --id $STAKE_4 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $STAKE_BC01344_HASH

# Stake 5 (PHO-USDC): upgrade to e4f767d
echo "Upgrading PHO-USDC stake to hash e4f767d..."
stellar contract invoke --id $STAKE_5 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $STAKE_E4F767D_HASH

echo "Phase 2 upgrade complete - contracts now match current network state"

# Verify phase 2 upgrades
echo ""
echo "=== VERIFYING PHASE 2 UPGRADES ==="
echo "Testing pool queries after intermediate upgrade..."
stellar contract invoke --id $POOL_1 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 1 (XLM-USDC) config query successful"
stellar contract invoke --id $POOL_2 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 2 (XLM-PHO) config query successful"
stellar contract invoke --id $POOL_3 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 3 (XLM-EURC) config query successful"
stellar contract invoke --id $POOL_4 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 4 (USDC-VEUR) config query successful"
stellar contract invoke --id $POOL_5 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 5 (PHO-USDC) config query successful"

echo "Testing stake queries after intermediate upgrade..."
stellar contract invoke --id $STAKE_1 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 1 admin query successful"
stellar contract invoke --id $STAKE_2 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 2 admin query successful"
stellar contract invoke --id $STAKE_3 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 3 admin query successful"
stellar contract invoke --id $STAKE_4 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 4 admin query successful"
stellar contract invoke --id $STAKE_5 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 5 admin query successful"

# === PHASE 3: Final migration to latest versions ===
echo ""
echo "=== PHASE 3: FINAL MIGRATION TO LATEST MAIN BRANCH VERSIONS ==="

echo "=== PHASE 3 UPGRADE DEBUGGING ==="
echo "Latest Factory Hash: $LATEST_FACTORY_HASH"
echo "Latest Multihop Hash: $LATEST_MULTIHOP_HASH"
echo "Latest Pool Hash: $LATEST_POOL_HASH"
echo "Latest Stake Hash: $LATEST_STAKE_HASH"
echo ""

echo "Upgrading factory to latest version..."
echo "Command: stellar contract invoke --id $FACTORY_ADDR --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_FACTORY_HASH --new_stable_pool_hash $LATEST_POOL_HASH"
stellar contract invoke --id $FACTORY_ADDR --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_FACTORY_HASH --new_stable_pool_hash $LATEST_POOL_HASH
echo "✓ Factory upgrade completed"
echo ""

echo "Upgrading multihop to latest version..."
echo "Command: stellar contract invoke --id $MULTIHOP_ADDR --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_MULTIHOP_HASH"
stellar contract invoke --id $MULTIHOP_ADDR --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_MULTIHOP_HASH
echo "✓ Multihop upgrade completed"
echo ""

echo "Upgrading all pools to latest version..."
echo "Latest Pool Hash being used: $LATEST_POOL_HASH"

echo "Upgrading Pool 1 (XLM-USDC): $POOL_1"
stellar contract invoke --id $POOL_1 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_POOL_HASH
echo "✓ Pool 1 upgrade completed"

echo "Upgrading Pool 2 (XLM-PHO): $POOL_2"
stellar contract invoke --id $POOL_2 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_POOL_HASH
echo "✓ Pool 2 upgrade completed"

echo "Upgrading Pool 3 (XLM-EURC): $POOL_3"
stellar contract invoke --id $POOL_3 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_POOL_HASH
echo "✓ Pool 3 upgrade completed"

echo "Upgrading Pool 4 (USDC-VEUR): $POOL_4"
stellar contract invoke --id $POOL_4 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_POOL_HASH
echo "✓ Pool 4 upgrade completed"

echo "Upgrading Pool 5 (PHO-USDC): $POOL_5"
stellar contract invoke --id $POOL_5 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_POOL_HASH
echo "✓ Pool 5 upgrade completed"
echo ""

echo "Upgrading all stakes to latest version..."
echo "Latest Stake Hash being used: $LATEST_STAKE_HASH"

echo "Upgrading Stake 1 (XLM-USDC): $STAKE_1"
stellar contract invoke --id $STAKE_1 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_STAKE_HASH
echo "✓ Stake 1 upgrade completed"

echo "Upgrading Stake 2 (XLM-PHO): $STAKE_2"
stellar contract invoke --id $STAKE_2 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_STAKE_HASH
echo "✓ Stake 2 upgrade completed"

echo "Upgrading Stake 3 (XLM-EURC): $STAKE_3"
stellar contract invoke --id $STAKE_3 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_STAKE_HASH
echo "✓ Stake 3 upgrade completed"

echo "Upgrading Stake 4 (USDC-VEUR): $STAKE_4"
stellar contract invoke --id $STAKE_4 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_STAKE_HASH
echo "✓ Stake 4 upgrade completed"

echo "Upgrading Stake 5 (PHO-USDC): $STAKE_5"
stellar contract invoke --id $STAKE_5 --source $IDENTITY_STRING --network $NETWORK -- update --new_wasm_hash $LATEST_STAKE_HASH
echo "✓ Stake 5 upgrade completed"

echo ""
echo "🎉 All Phase 3 upgrades completed!"

# Verify phase 3 upgrades
echo ""
echo "=== VERIFYING PHASE 3 UPGRADES ==="
echo "Testing factory query after final upgrade..."
stellar contract invoke --id $FACTORY_ADDR --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Factory config query successful"

echo "Testing multihop query after final upgrade..."
stellar contract invoke --id $MULTIHOP_ADDR --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Multihop config query successful"

echo "Testing pool queries after final upgrade..."
stellar contract invoke --id $POOL_1 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 1 (XLM-USDC) config query successful"
stellar contract invoke --id $POOL_2 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 2 (XLM-PHO) config query successful"
stellar contract invoke --id $POOL_3 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 3 (XLM-EURC) config query successful"
stellar contract invoke --id $POOL_4 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 4 (USDC-VEUR) config query successful"
stellar contract invoke --id $POOL_5 --source $IDENTITY_STRING --network $NETWORK -- query_config && echo "✓ Pool 5 (PHO-USDC) config query successful"

echo "Testing stake queries after final upgrade..."
stellar contract invoke --id $STAKE_1 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 1 admin query successful"
stellar contract invoke --id $STAKE_2 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 2 admin query successful"
stellar contract invoke --id $STAKE_3 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 3 admin query successful"
stellar contract invoke --id $STAKE_4 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 4 admin query successful"
stellar contract invoke --id $STAKE_5 --source $IDENTITY_STRING --network $NETWORK -- query_admin && echo "✓ Stake 5 admin query successful"

echo "Testing pool functionality after final upgrade..."
stellar contract invoke --id $POOL_1 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory && echo "✓ Pool 1 factory query successful"
stellar contract invoke --id $POOL_2 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory && echo "✓ Pool 2 factory query successful"
stellar contract invoke --id $POOL_3 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory && echo "✓ Pool 3 factory query successful"
stellar contract invoke --id $POOL_4 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory && echo "✓ Pool 4 factory query successful"
stellar contract invoke --id $POOL_5 --source $IDENTITY_STRING --network $NETWORK -- query_pool_info_for_factory && echo "✓ Pool 5 factory query successful"

echo "Testing stake functionality after final upgrade..."
stellar contract invoke --id $STAKE_1 --source $IDENTITY_STRING --network $NETWORK -- query_staked --address $POOL_USER && echo "✓ Stake 1 staked query successful"
stellar contract invoke --id $STAKE_2 --source $IDENTITY_STRING --network $NETWORK -- query_staked --address $POOL_USER && echo "✓ Stake 2 staked query successful"
stellar contract invoke --id $STAKE_3 --source $IDENTITY_STRING --network $NETWORK -- query_staked --address $POOL_USER && echo "✓ Stake 3 staked query successful"
stellar contract invoke --id $STAKE_4 --source $IDENTITY_STRING --network $NETWORK -- query_staked --address $POOL_USER && echo "✓ Stake 4 staked query successful"
stellar contract invoke --id $STAKE_5 --source $IDENTITY_STRING --network $NETWORK -- query_staked --address $POOL_USER && echo "✓ Stake 5 staked query successful"

# Cleanup temporary files
echo ""
echo "Cleaning up temporary files..."
rm -f .temp_*.wasm .temp_*.optimized.wasm

echo ""
echo "=== MIGRATION TESTING COMPLETE ==="
echo ""
echo "✅ Phase 1: All contracts deployed from earliest commit (77742b01)"
echo "✅ Phase 2: Contracts upgraded to intermediate versions matching network state"
echo "✅ Phase 3: Final migration to latest main branch versions completed successfully"
echo ""
echo "Contract Addresses:"
echo "Factory: $FACTORY_ADDR"
echo "Multihop: $MULTIHOP_ADDR"
echo ""
echo "Pools:"
echo "XLM-USDC Pool: $POOL_1 (Stake: $STAKE_1)"
echo "XLM-PHO Pool: $POOL_2 (Stake: $STAKE_2)"
echo "XLM-EURC Pool: $POOL_3 (Stake: $STAKE_3)"
echo "USDC-VEUR Pool: $POOL_4 (Stake: $STAKE_4)"
echo "PHO-USDC Pool: $POOL_5 (Stake: $STAKE_5)"
echo ""
echo "Tokens:"
echo "XLM: $XLM_TOKEN"
echo "USDC: $USDC_TOKEN"
echo "PHO: $PHO_TOKEN"
echo "EURC: $EURC_TOKEN"
echo "VEUR: $VEUR_TOKEN"
echo ""
echo "Test User: $POOL_USER"
echo ""
echo "🎉 All migration phases completed successfully! The testnet now simulates the"
echo "   complete upgrade path from the original deployment through current network"
echo "   state to the latest main branch version."
