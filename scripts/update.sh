# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

# Folder name to check and delete
FOLDER=".stellar"

# Check if the folder exists
if [ -d "$FOLDER" ]; then
    echo "Folder '$FOLDER' exists. Deleting it..."
    rm -rf "$FOLDER"
    echo "Folder '$FOLDER' has been deleted."
else
    echo "Folder '$FOLDER' does not exist."
fi

IDENTITY_STRING=$1
ADMIN_ADDRESS=$(stellar keys address $IDENTITY_STRING)
NETWORK="testnet"

echo "Build and optimize the contracts...";

make build > /dev/null

echo "Contracts compiled."
echo "Optimize contracts..."

stellar contract optimize --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.wasm
stellar contract optimize --wasm .wasm_binaries_mainnet/live_token_contract.wasm

stellar contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_factory.wasm
stellar contract optimize --wasm .wasm_binaries_mainnet/live_factory.wasm

stellar contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_pool.wasm
stellar contract optimize --wasm .wasm_binaries_mainnet/live_xlm_usdc_pool.wasm

stellar contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_pool_stable.wasm

stellar contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_stake.wasm
stellar contract optimize --wasm .wasm_binaries_mainnet/live_xlm_usdc_stake.wasm

stellar contract optimize --wasm .artifacts_sdk_update/old_phoenix_stake_rewards.wasm

stellar contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_multihop.wasm
stellar contract optimize --wasm .wasm_binaries_mainnet/live_multihop.wasm

echo "Contracts optimized."

echo "installing old and latest wasm hashes"

OLD_SOROBAN_TOKEN_WASM_HASH=$(stellar contract upload \
    --wasm .wasm_binaries_mainnet/live_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

LATEST_SOROBAN_TOKEN_WASM_HASH=$(stellar contract upload \
    --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

echo "Installed old token wasm: $OLD_SOROBAN_TOKEN_WASM_HASH"
echo "Installed latest token wasm: $LATEST_SOROBAN_TOKEN_WASM_HASH"

OLD_PHOENIX_FACTORY_WASM_HASH=$(stellar contract upload \
    --wasm .wasm_binaries_mainnet/live_factory.optimized.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_FACTORY_WASM_HASH=$(stellar contract upload \
    --wasm target/wasm32-unknown-unknown/release/phoenix_factory.optimized.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old factory wasm: $OLD_PHOENIX_FACTORY_WASM_HASH"
echo "Installed latest factory wasm: $LATEST_PHOENIX_FACTORY_WASM_HASH"

OLD_PHOENIX_POOL_WASM_HASH=$(stellar contract upload \
    --wasm .wasm_binaries_mainnet/live_xlm_usdc_pool.optimized.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_POOL_WASM_HASH=$(stellar contract upload \
    --wasm target/wasm32-unknown-unknown/release/phoenix_pool.optimized.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old pool wasm: $OLD_PHOENIX_POOL_WASM_HASH"
echo "Installed latest pool wasm: $LATEST_PHOENIX_POOL_WASM_HASH"

OLD_PHOENIX_POOL_STABLE_WASM_HASH=$(stellar contract upload \
    --wasm .artifacts_sdk_update/old_phoenix_pool_stable.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_POOL_STABLE_WASM_HASH=$(stellar contract upload \
    --wasm .artifacts_sdk_update/old_phoenix_pool_stable.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")

echo "Installed old stable pool wasm: $OLD_PHOENIX_POOL_STABLE_WASM_HASH"
echo "Installed latest stable pool wasm: $LATEST_PHOENIX_POOL_STABLE_WASM_HASH"

OLD_PHOENIX_STAKE_WASM_HASH=$(stellar contract upload \
    --wasm .wasm_binaries_mainnet/live_xlm_usdc_stake.optimized.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_STAKE_WASM_HASH=$(stellar contract upload \
    --wasm target/wasm32-unknown-unknown/release/phoenix_stake.optimized.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old stake wasm: $OLD_PHOENIX_STAKE_WASM_HASH"
echo "Installed latest stake wasm: $LATEST_PHOENIX_STAKE_WASM_HASH"

OLD_PHOENIX_STAKE_REWARDS_WASM_HASH=$(stellar contract upload \
    --wasm .artifacts_sdk_update/old_phoenix_stake_rewards.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_STAKE_REWARDS_WASM_HASH=$(stellar contract upload \
    --wasm .artifacts_sdk_update/old_phoenix_stake_rewards.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old stake rewards wasm: $OLD_PHOENIX_STAKE_REWARDS_WASM_HASH"
echo "Installed latest stake rewards wasm: $LATEST_PHOENIX_STAKE_REWARDS_WASM_HASH"

OLD_PHOENIX_MULTIHOP_WASM_HASH=$(stellar contract upload \
    --wasm .wasm_binaries_mainnet/live_multihop.optimized.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_MULTIHOP_WASM_HASH=$(stellar contract upload \
    --wasm target/wasm32-unknown-unknown/release/phoenix_multihop.optimized.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old multihop wasm: $OLD_PHOENIX_MULTIHOP_WASM_HASH"
echo "Installed latest multihop wasm: $LATEST_PHOENIX_MULTIHOP_WASM_HASH"

echo "All old and latest WASMs have been installed successfully."


echo "Deploying old factory contract..."
FACTORY_ADDR=$(stellar contract deploy \
  --wasm-hash "$OLD_PHOENIX_FACTORY_WASM_HASH" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK")
echo "Old factory deployed at: $FACTORY_ADDR"


echo "Initializing old factory..."
stellar contract invoke \
  --id "$FACTORY_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  initialize \
  --admin "$ADMIN_ADDRESS" \
  --multihop_wasm_hash "$OLD_PHOENIX_MULTIHOP_WASM_HASH" \
  --lp_wasm_hash "$OLD_PHOENIX_POOL_WASM_HASH" \
  --stable_wasm_hash "$OLD_PHOENIX_POOL_STABLE_WASM_HASH" \
  --stake_wasm_hash "$OLD_PHOENIX_STAKE_WASM_HASH" \
  --token_wasm_hash "$OLD_SOROBAN_TOKEN_WASM_HASH" \
  --whitelisted_accounts "[ \"$ADMIN_ADDRESS\" ]" \
  --lp_token_decimals 7

echo "Old factory initialized at $FACTORY_ADDR."


echo "Checking the admin of the old factory..."
FACTORY_ADMIN=$(stellar contract invoke \
  --id "$FACTORY_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  get_admin)

echo "Factory admin is: $FACTORY_ADMIN (expected \"${ADMIN_ADDRESS}\")"
# Typically the returned value is in quotes, e.g. "\"GA...\""
if [ "$FACTORY_ADMIN" != "\"${ADMIN_ADDRESS}\"" ]; then
  echo "ERROR: Admin does not match expected address."
  exit 1
else
  echo "Factory admin matches as expected."
fi


echo "Updating old factory to new factory code..."

stellar contract invoke \
  --id "$FACTORY_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  update \
  --new_wasm_hash "$LATEST_PHOENIX_FACTORY_WASM_HASH" \

echo "Factory updated to the latest code."


echo "Checking the admin of the updated factory..."
UPDATED_FACTORY_ADMIN=$(stellar contract invoke \
  --id "$FACTORY_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  get_admin)

echo "Updated factory admin is: $UPDATED_FACTORY_ADMIN (expected \"${ADMIN_ADDRESS}\")"
if [ "$UPDATED_FACTORY_ADMIN" != "\"${ADMIN_ADDRESS}\"" ]; then
  echo "ERROR: Admin changed after update."
  exit 1
else
  echo "Admin is still correct after factory update."
fi


echo "Updating wasm hashes on the updated factory..."

LATEST_LP_WASM_HASH="$LATEST_PHOENIX_POOL_WASM_HASH"
LATEST_STAKE_WASM_HASH="$LATEST_PHOENIX_STAKE_WASM_HASH"
LATEST_TOKEN_WASM_HASH="$LATEST_SOROBAN_TOKEN_WASM_HASH"

stellar contract invoke \
  --id "$FACTORY_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  update_config \
  --lp_wasm_hash "$LATEST_LP_WASM_HASH" \
  --stake_wasm_hash "$LATEST_STAKE_WASM_HASH" \
  --token_wasm_hash "$LATEST_TOKEN_WASM_HASH"

echo "WASM hashes updated on the factory."


echo "'update_factory' test have been replicated via shell."


echo "Deploying old multihop contract..."

OLD_MULTIHOP_ADDR=$(stellar contract deploy \
  --wasm-hash "$OLD_PHOENIX_MULTIHOP_WASM_HASH" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK")

echo "Old multihop contract deployed at: $OLD_MULTIHOP_ADDR"


echo "Initializing old multihop contract..."

stellar contract invoke \
  --id "$OLD_MULTIHOP_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  initialize \
  --admin "$ADMIN_ADDRESS" \
  --factory "$FACTORY_ADDR"

echo "Old multihop initialized at $OLD_MULTIHOP_ADDR"

echo "Updating old multihop contract to latest multihop code..."

stellar contract invoke \
  --id "$OLD_MULTIHOP_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  update \
  --new_wasm_hash "$LATEST_PHOENIX_MULTIHOP_WASM_HASH"

echo "Multihop contract updated to the latest code."

echo "'updapte_multihop' test have been replicated."
echo "Old -> Updated multihop contract address: $OLD_MULTIHOP_ADDR"

stellar keys generate luke --network $NETWORK --fund
stellar keys generate obiwan --network $NETWORK --fund
stellar keys generate jarjar --network $NETWORK --fund

ADMIN=$(stellar keys address luke)
USER=$(stellar keys address jarjar)


echo "Deploying token contract 1 under admin..."
TOKEN_ADDR1=$(stellar contract deploy \
  --wasm-hash "$OLD_SOROBAN_TOKEN_WASM_HASH" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK")
echo "token1 deployed at: $TOKEN_ADDR1"

echo "Initializing token1 with admin=$ADMIN..."
stellar contract invoke \
  --id "$TOKEN_ADDR1" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  initialize \
  --admin "$IDENTITY_STRING" \
  --decimal 7 \
  --name "TestToken1" \
  --symbol "TKN1"

echo "Deploying token contract 2 under admin..."
TOKEN_ADDR2=$(stellar contract deploy \
  --wasm-hash "$OLD_SOROBAN_TOKEN_WASM_HASH" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK")
echo "token2 deployed at: $TOKEN_ADDR2"

echo "Initializing token2 with admin=$ADMIN..."
stellar contract invoke \
  --id "$TOKEN_ADDR2" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  initialize \
  --admin "$IDENTITY_STRING" \
  --decimal 7 \
  --name "TestToken2" \
  --symbol "TKN2"

# Sort the token addresses alphabetically
if [[ "$TOKEN_ADDR1" < "$TOKEN_ADDR2" ]]; then
    TOKEN_ID1=$TOKEN_ADDR1
    TOKEN_ID2=$TOKEN_ADDR2
else
    TOKEN_ID1=$TOKEN_ADDR2
    TOKEN_ID2=$TOKEN_ADDR1
fi

echo "Sorted token1: $TOKEN_ID1 (admin: $ADMIN)"
echo "Sorted token2: $TOKEN_ID2 (admin: $ADMIN)"

echo "Deploying old liquidity pool contract..."
OLD_LP_ID=$(stellar contract deploy \
  --wasm-hash "$OLD_PHOENIX_POOL_WASM_HASH" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK")
echo "Old liquidity pool contract at: $OLD_LP_ID"


STAKE_WASM_HASH="$OLD_PHOENIX_STAKE_WASM_HASH"
TOKEN_WASM_HASH="$OLD_SOROBAN_TOKEN_WASM_HASH"

echo "Initializing old liquidity pool..."
stellar contract invoke \
  --id "$OLD_LP_ID" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  initialize \
  --stake_wasm_hash "$OLD_PHOENIX_STAKE_WASM_HASH" \
  --token_wasm_hash "$OLD_SOROBAN_TOKEN_WASM_HASH" \
  --lp_init_info "{ 
       \"admin\": \"${ADMIN_ADDRESS}\",
       \"swap_fee_bps\": 1000,
       \"fee_recipient\": \"${ADMIN_ADDRESS}\",
       \"max_allowed_slippage_bps\": 5000,
       \"default_slippage_bps\": 2500,
       \"max_allowed_spread_bps\": 10000,
       \"max_referral_bps\": 5000,
       \"token_init_info\": {
         \"token_a\": \"${TOKEN_ID1}\",
         \"token_b\": \"${TOKEN_ID2}\"
       },
       \"stake_init_info\": {
         \"min_bond\": \"100\",
         \"min_reward\": \"5\",
         \"manager\": \"${ADMIN_ADDRESS}\",
         \"max_complexity\": 7
       }
     }" \
  --factory_addr "$ADMIN_ADDRESS" \
  --share_token_decimals 7 \
  --share_token_name "Pool" \
  --share_token_symbol "PHOBTC" \
  --default_slippage_bps 100 \
  --max_allowed_fee_bps 1000

echo "Old LP initialized at $OLD_LP_ID"

echo "Querying old LP config for fee_recipient..."
FEE_RECIPIENT=$(stellar contract invoke \
  --id "$OLD_LP_ID" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  query_config | jq -r '.fee_recipient')

echo "Fee recipient in old LP: $FEE_RECIPIENT (expected ${ADMIN_ADDRESS})"


echo "Minting big amounts of token1 and token2 for user..."

stellar contract invoke \
  --id $TOKEN_ID1 \
  --source $IDENTITY_STRING \
  --network $NETWORK \
  -- \
  mint --to $USER --amount 10000000000 # 7 decimals, 1k tokens

stellar contract invoke \
  --id $TOKEN_ID2 \
  --source $IDENTITY_STRING \
  --network $NETWORK \
  -- \
  mint --to $USER --amount 10000000000 # 7 decimals, 1k tokens

echo "User1 now has 1000 of each token."

echo "User providing liquidity to old LP..."
echo "USER: " $USER
echo "OLD_LP_ID: " $OLD_LP_ID

stellar contract invoke \
  --id "$OLD_LP_ID" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  provide_liquidity \
  --sender $(stellar keys secret jarjar) \
  --min_a 5000000000 \
  --min_b 5000000000 \
  --desired_a 5000000000 \
  --desired_b 5000000000 \

echo "Liquidity provided in old LP."

stellar contract invoke \
  --id "$OLD_LP_ID" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  update \
  --new_wasm_hash "$LATEST_PHOENIX_POOL_WASM_HASH"

echo "Old LP updated to new code!"

NEW_FEE_RECIPIENT=$(stellar contract invoke \
  --id "$OLD_LP_ID" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  query_config | jq -r '.fee_recipient')

echo "Fee recipient in new LP: $NEW_FEE_RECIPIENT"


echo "User1 withdrawing half of liquidity from new LP..."

stellar contract invoke \
  --id "$OLD_LP_ID" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  withdraw_liquidity \
  --sender $(stellar keys secret jarjar) \
  --share_amount 2500000000 \
  --min_a 2500000000 \
  --min_b 2500000000 \

echo "Liquidity partially withdrawn."

POOL_INFO=$(stellar contract invoke \
  --id "$OLD_LP_ID" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  query_pool_info_for_factory)

echo "Pool info after upgrade: $POOL_INFO"
echo "Expected: asset_a.amount=2500000000, asset_b.amount=2500000000, etc."

ASSET_A_AMOUNT=$(echo "$POOL_INFO" | jq -r '.pool_response.asset_a.amount')
ASSET_B_AMOUNT=$(echo "$POOL_INFO" | jq -r '.pool_response.asset_b.amount')
ASSET_LP_SHARE_AMOUNT=$(echo "$POOL_INFO" | jq -r '.pool_response.asset_lp_share.amount')

echo "ASSET_A_AMOUNT: $ASSET_A_AMOUNT"
echo "ASSET_B_AMOUNT: $ASSET_B_AMOUNT"
echo "ASSET_LP_SHARE: $ASSET_LP_SHARE_AMOUNT"

echo "'update_liquidity_pool' test have been replicated via shell."
echo "Liquidity Pool contract address: $OLD_LP_ID"

stellar keys generate stake_admin --network "$NETWORK" --fund
stellar keys generate stake_user --network "$NETWORK" --fund

STAKE_ADMIN=$(stellar keys address stake_admin)
STAKE_USER=$(stellar keys address stake_user)

STAKE_ADMIN_SECRET=$(stellar keys secret stake_admin)
STAKE_USER_SECRET=$(stellar keys secret stake_user)

echo "Stake admin: $STAKE_ADMIN"
echo "Stake user:  $STAKE_USER"
echo "Stake admin: $STAKE_ADMIN_SECRET"
echo "Stake user:  $STAKE_USER_SECRET"

STAKE_TOKEN_ADDR=$(stellar contract deploy \
  --wasm-hash "$OLD_SOROBAN_TOKEN_WASM_HASH" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK")

echo "Token for stake deployed at: $STAKE_TOKEN_ADDR"

echo "Initializing stake token with admin=$STAKE_ADMIN..."
stellar contract invoke \
  --id "$STAKE_TOKEN_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  initialize \
  --admin "$STAKE_ADMIN" \
  --decimal 7 \
  --name "StakeToken" \
  --symbol "STK"

stellar contract invoke \
  --id "$STAKE_TOKEN_ADDR" \
  --source "$STAKE_ADMIN_SECRET" \
  --network "$NETWORK" \
  -- \
  mint \
  --to "$STAKE_USER" \
  --amount 1000

OLD_STAKE_ADDR=$(stellar contract deploy \
  --wasm-hash "$OLD_PHOENIX_STAKE_WASM_HASH" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK")
echo "Old stake contract deployed at: $OLD_STAKE_ADDR"

stellar keys generate stake_manager --network "$NETWORK" --fund
stellar keys generate stake_owner --network "$NETWORK" --fund

MANAGER_ADDR=$(stellar keys address stake_manager)
OWNER_ADDR=$(stellar keys address stake_owner)

MANAGER_SECRET=$(stellar keys secret stake_manager)
OWNER_SECRET=$(stellar keys secret stake_owner)

echo "Manager: $MANAGER_ADDR"
echo "Owner:   $OWNER_ADDR"

stellar contract invoke \
  --id "$OLD_STAKE_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  initialize \
  --admin "$STAKE_ADMIN" \
  --lp_token "$STAKE_TOKEN_ADDR" \
  --min_bond "10" \
  --min_reward "10" \
  --manager "$MANAGER_ADDR" \
  --owner "$OWNER_ADDR" \
  --max_complexity "10"

echo "Old stake initialized at $OLD_STAKE_ADDR"

echo "Bonding 1000 tokens from $STAKE_USER..."
stellar contract invoke \
  --id "$OLD_STAKE_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  bond \
  --sender "$STAKE_USER_SECRET" \
  --tokens 1000

echo "Bonded 1000 tokens."

echo "Checking staked info for user after bonding..."

STAKED_INFO=$(stellar contract invoke \
  --id "$OLD_STAKE_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  query_staked \
  --address "$STAKE_USER")

echo "Staked info: $STAKED_INFO"

STAKE_AMOUNT=$(echo "$STAKED_INFO" | jq -r '.stakes[0].stake')
STAKE_TIMESTAMP=$(echo "$STAKED_INFO" | jq -r '.stakes[0].stake_timestamp')
LAST_REWARD_TIME=$(echo "$STAKED_INFO" | jq -r '.last_reward_time')
TOTAL_STAKE=$(echo "$STAKED_INFO" | jq -r '.total_stake')

if [ "$STAKE_AMOUNT" -eq 1000 ] \
   && [ "$LAST_REWARD_TIME" -eq 0 ] \
   && [ "$TOTAL_STAKE" -eq 1000 ]; then
  echo "Staked info matches expected values!"
else
  echo "ERROR: Staked info mismatch."
  echo "  stake=$STAKE_AMOUNT (expected 1000)"
  echo "  last_reward_time=$LAST_REWARD_TIME (expected 0)"
  echo "  total_stake=$TOTAL_STAKE (expected 1000)"
  exit 1
fi

echo "Updating old stake contract to latest stake code..."
stellar contract invoke \
  --id "$OLD_STAKE_ADDR" \
  --source "$STAKE_ADMIN_SECRET" \
  --network "$NETWORK" \
  -- \
  update \
  --new_wasm_hash "$LATEST_PHOENIX_STAKE_WASM_HASH"

echo "Stake contract updated."

UPDATED_STAKE_ADMIN=$(stellar contract invoke \
  --id "$OLD_STAKE_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  query_admin)

echo "Updated stake admin is: $UPDATED_STAKE_ADMIN (expected \"$STAKE_ADMIN\")"

echo "Unbonding 1000 tokens from user..."
stellar contract invoke \
  --id "$OLD_STAKE_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  unbond \
  --sender "$STAKE_USER_SECRET" \
  --stake_amount 1000 \
  --stake_timestamp $STAKE_TIMESTAMP

echo "'upgrade_stake_contract' test replicated!"
echo "Stake contract address: $OLD_STAKE_ADDR"

echo "Running updapte_stake_rewards test"

stellar keys generate stake_rewards_admin --network "$NETWORK" --fund
stellar keys generate stake_rewards_staking --network "$NETWORK" --fund
stellar keys generate stake_rewards_token --network "$NETWORK" --fund

STAKE_REWARDS_ADMIN=$(stellar keys address stake_rewards_admin)
STAKE_REWARDS_STAKING=$(stellar keys address stake_rewards_staking)
STAKE_REWARDS_TOKEN=$(stellar keys address stake_rewards_token)

echo "Stake Rewards Admin:     $STAKE_REWARDS_ADMIN"
echo "Staking Contract (mock): $STAKE_REWARDS_STAKING"
echo "Reward Token (mock):     $STAKE_REWARDS_TOKEN"

OLD_STAKE_REWARDS_ADDR=$(stellar contract deploy \
  --wasm-hash "$OLD_PHOENIX_STAKE_REWARDS_WASM_HASH" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK")

echo "Old stake rewards contract deployed at: $OLD_STAKE_REWARDS_ADDR"

MAX_COMPLEXITY="10"
MIN_REWARD="5"
MIN_BOND="5"

echo "Initializing old stake rewards contract..."
stellar contract invoke \
  --id "$OLD_STAKE_REWARDS_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  initialize \
  --admin "$STAKE_REWARDS_ADMIN" \
  --staking_contract "$STAKE_REWARDS_STAKING" \
  --reward_token "$STAKE_REWARDS_TOKEN" \
  --max_complexity "$MAX_COMPLEXITY" \
  --min_reward "$MIN_REWARD" \
  --min_bond "$MIN_BOND"

echo "Old stake rewards contract initialized."

echo "Checking old stake rewards admin..."
OLD_STAKE_REWARDS_ADMIN=$(stellar contract invoke \
  --id "$OLD_STAKE_REWARDS_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  query_admin)
echo "Admin in old stake rewards: $OLD_STAKE_REWARDS_ADMIN (expected \"$STAKE_REWARDS_ADMIN\")"

echo "Querying old stake rewards config..."
OLD_SR_CONFIG=$(stellar contract invoke \
  --id "$OLD_STAKE_REWARDS_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  query_config)

echo "Old stake rewards config: $OLD_SR_CONFIG"
echo "Updating old stake rewards contract to latest code..."
stellar contract invoke \
  --id "$OLD_STAKE_REWARDS_ADDR" \
  --source "$(stellar keys secret stake_rewards_admin)" \
  --network "$NETWORK" \
  -- \
  update \
  --new_wasm_hash "$LATEST_PHOENIX_STAKE_REWARDS_WASM_HASH"

echo "Stake rewards contract updated."

echo "Checking updated stake rewards admin..."
UPDATED_STAKE_REWARDS_ADMIN=$(stellar contract invoke \
  --id "$OLD_STAKE_REWARDS_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  query_admin)
echo "Updated stake rewards admin is: $UPDATED_STAKE_REWARDS_ADMIN (expected \"$STAKE_REWARDS_ADMIN\")"

echo "Querying updated stake rewards config..."
UPDATED_SR_CONFIG=$(stellar contract invoke \
  --id "$OLD_STAKE_REWARDS_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  query_config)

echo "Updated stake rewards config: $UPDATED_SR_CONFIG"

echo "'updapte_stake_rewards' test replicated successfully!"
echo "Old -> Updated stake rewards contract address: $OLD_STAKE_REWARDS_ADDR"

# Check if the folder exists
if [ -d "$FOLDER" ]; then
    echo "Folder '$FOLDER' exists. Deleting it..."
    rm -rf "$FOLDER"
    echo "Folder '$FOLDER' has been deleted."
else
    echo "Folder '$FOLDER' does not exist."
fi

echo "Updates were successful"
