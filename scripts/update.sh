# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

IDENTITY_STRING=$1
ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)
NETWORK="testnet"

echo "Build and optimize the contracts...";

make build > /dev/null

echo "Contracts compiled."
echo "Optimize contracts..."

soroban contract optimize --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.wasm
soroban contract optimize --wasm .artifacts/old_soroban_token_contract.wasm

soroban contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_factory.wasm
soroban contract optimize --wasm .artifacts/old_phoenix_factory.wasm

soroban contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_pool.wasm
soroban contract optimize --wasm .artifacts/old_phoenix_pool.wasm

soroban contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_pool_stable.wasm
soroban contract optimize --wasm .artifacts/old_phoenix_pool_stable.wasm

soroban contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_stake.wasm
soroban contract optimize --wasm .artifacts/old_phoenix_stake.wasm

soroban contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_stake_rewards.wasm
soroban contract optimize --wasm .artifacts/old_phoenix_stake_rewards.wasm

soroban contract optimize --wasm target/wasm32-unknown-unknown/release/phoenix_multihop.wasm
soroban contract optimize --wasm .artifacts/old_phoenix_multihop.wasm

echo "Contracts optimized."

echo "installing old and latest wasm hashes"

OLD_SOROBAN_TOKEN_WASM_HASH=$(soroban contract install \
    --wasm .artifacts/old_soroban_token_contract.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

LATEST_SOROBAN_TOKEN_WASM_HASH=$(soroban contract install \
    --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

echo "Installed old token wasm: $OLD_SOROBAN_TOKEN_WASM_HASH"
echo "Installed latest token wasm: $LATEST_SOROBAN_TOKEN_WASM_HASH"

OLD_PHOENIX_FACTORY_WASM_HASH=$(soroban contract install \
    --wasm .artifacts/old_phoenix_factory.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_FACTORY_WASM_HASH=$(soroban contract install \
    --wasm target/wasm32-unknown-unknown/release/phoenix_factory.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old factory wasm: $OLD_PHOENIX_FACTORY_WASM_HASH"
echo "Installed latest factory wasm: $LATEST_PHOENIX_FACTORY_WASM_HASH"

OLD_PHOENIX_POOL_WASM_HASH=$(soroban contract install \
    --wasm .artifacts/old_phoenix_pool.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_POOL_WASM_HASH=$(soroban contract install \
    --wasm target/wasm32-unknown-unknown/release/phoenix_pool.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old pool wasm: $OLD_PHOENIX_POOL_WASM_HASH"
echo "Installed latest pool wasm: $LATEST_PHOENIX_POOL_WASM_HASH"

OLD_PHOENIX_POOL_STABLE_WASM_HASH=$(soroban contract install \
    --wasm .artifacts/old_phoenix_pool_stable.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_POOL_STABLE_WASM_HASH=$(soroban contract install \
    --wasm target/wasm32-unknown-unknown/release/phoenix_pool_stable.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old stable pool wasm: $OLD_PHOENIX_POOL_STABLE_WASM_HASH"
echo "Installed latest stable pool wasm: $LATEST_PHOENIX_POOL_STABLE_WASM_HASH"

OLD_PHOENIX_STAKE_WASM_HASH=$(soroban contract install \
    --wasm .artifacts/old_phoenix_stake.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_STAKE_WASM_HASH=$(soroban contract install \
    --wasm target/wasm32-unknown-unknown/release/phoenix_stake.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old stake wasm: $OLD_PHOENIX_STAKE_WASM_HASH"
echo "Installed latest stake wasm: $LATEST_PHOENIX_STAKE_WASM_HASH"

OLD_PHOENIX_STAKE_REWARDS_WASM_HASH=$(soroban contract install \
    --wasm .artifacts/old_phoenix_stake_rewards.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_STAKE_REWARDS_WASM_HASH=$(soroban contract install \
    --wasm target/wasm32-unknown-unknown/release/phoenix_stake_rewards.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old stake rewards wasm: $OLD_PHOENIX_STAKE_REWARDS_WASM_HASH"
echo "Installed latest stake rewards wasm: $LATEST_PHOENIX_STAKE_REWARDS_WASM_HASH"

OLD_PHOENIX_MULTIHOP_WASM_HASH=$(soroban contract install \
    --wasm .artifacts/old_phoenix_multihop.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
LATEST_PHOENIX_MULTIHOP_WASM_HASH=$(soroban contract install \
    --wasm target/wasm32-unknown-unknown/release/phoenix_multihop.wasm \
    --source "$IDENTITY_STRING" \
    --network "$NETWORK")
echo "Installed old multihop wasm: $OLD_PHOENIX_MULTIHOP_WASM_HASH"
echo "Installed latest multihop wasm: $LATEST_PHOENIX_MULTIHOP_WASM_HASH"

echo "All old and latest WASMs have been installed successfully."


echo "Deploying old factory contract..."
FACTORY_ADDR=$(soroban contract deploy \
  --wasm-hash "$OLD_PHOENIX_FACTORY_WASM_HASH" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK")
echo "Old factory deployed at: $FACTORY_ADDR"


echo "Initializing old factory..."
soroban contract invoke \
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

echo "Old factory initialized."


echo "Checking the admin of the old factory..."
FACTORY_ADMIN=$(soroban contract invoke \
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

soroban contract invoke \
  --id "$FACTORY_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  update \
  --new_wasm_hash "$LATEST_PHOENIX_FACTORY_WASM_HASH" \
  --new_stable_pool_hash "$LATEST_PHOENIX_POOL_STABLE_WASM_HASH"

echo "Factory updated to the latest code."


echo "Checking the admin of the updated factory..."
UPDATED_FACTORY_ADMIN=$(soroban contract invoke \
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

soroban contract invoke \
  --id "$FACTORY_ADDR" \
  --source "$IDENTITY_STRING" \
  --network "$NETWORK" \
  -- \
  update_wasm_hashes \
  --lp_wasm_hash "$LATEST_LP_WASM_HASH" \
  --stake_wasm_hash "$LATEST_STAKE_WASM_HASH" \
  --token_wasm_hash "$LATEST_TOKEN_WASM_HASH"

echo "WASM hashes updated on the factory."


echo "'update_factory' test have been replicated via shell."

