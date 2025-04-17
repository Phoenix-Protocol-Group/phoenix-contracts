# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

IDENTITY_STRING=$1
ADMIN_ADDRESS=$(stellar keys address $IDENTITY_STRING)
NETWORK="mainnet"

FACTORY_ADDRESS="CB4SVAWJA6TSRNOJZ7W2AWFW46D5VR4ZMFZKDIKXEINZCZEGZCJZCKMI"
MULTIHOP_ADDRESS="CCLZRD4E72T7JCZCN3P7KNPYNXFYKQCL64ECLX7WP5GNVYPYJGU2IO2G"
VESTING_ADDRESS="CDEGWCGEMNFZT3UUQD7B4TTPDHXZLGEDB6WIP4PWNTXOR5EZD34HJ64O"

PHO_USDC_POOL_ADDRESS="CD5XNKK3B6BEF2N7ULNHHGAMOKZ7P6456BFNIHRF4WNTEDKBRWAE7IAA"
XLM_PHO_POOL_ADDRESS="CBCZGGNOEUZG4CAAE7TGTQQHETZMKUT4OIPFHHPKEUX46U4KXBBZ3GLH"
XLM_USDC_POOL_ADDRESS="CBHCRSVX3ZZ7EGTSYMKPEFGZNWRVCSESQR3UABET4MIW52N4EVU6BIZX"
XLM_EURC_POOL_ADDRESS="CBISULYO5ZGS32WTNCBMEFCNKNSLFXCQ4Z3XHVDP4X4FLPSEALGSY3PS"
USDC_VEUR_POOL_ADDRESS="CDQLKNH3725BUP4HPKQKMM7OO62FDVXVTO7RCYPID527MZHJG2F3QBJW"
USDC_VCHF_POOL_ADDRESS="CBW5G5SO5SDYUGQVU7RMZ2KJ34POM3AMODOBIV2RQYG4KJDUUBVC3P2T"
XLM_USDX_POOL_ADDRESS="CDMXKSLG5GITGFYERUW2MRYOBUQCMRT2QE5Y4PU3QZ53EBFWUXAXUTBC"
EURX_USDC_POOL_ADDRESS="CC6MJZN3HFOJKXN42ANTSCLRFOMHLFXHWPNAX64DQNUEBDMUYMPHASAV"
XLM_EURX_POOL_ADDRESS="CB5QUVK5GS3IU23TMFZQ3P5J24YBBZP5PHUQAEJ2SP5K55PFTJRUQG2L"
XLM_GBPX_POOL_ADDRESS="CCKOC2LJTPDBKDHTL3M5UO7HFZ2WFIHSOKCELMKQP3TLCIVUBKOQL4HB"
GBPX_USDC_POOL_ADDRESS="CCUCE5H5CKW3S7JBESGCES6ZGDMWLNRY3HOFET3OH33MXZWKXNJTKSM3"


pools=(
  PHO_USDC_POOL_ADDRESS
  XLM_PHO_POOL_ADDRESS
  XLM_USDC_POOL_ADDRESS
  XLM_EURC_POOL_ADDRESS
  USDC_VEUR_POOL_ADDRESS
  USDC_VCHF_POOL_ADDRESS
  XLM_USDX_POOL_ADDRESS
  EURX_USDC_POOL_ADDRESS
  XLM_EURX_POOL_ADDRESS
  XLM_GBPX_POOL_ADDRESS
  GBPX_USDC_POOL_ADDRESS
)


PHO_USDC_STAKE_ADDRESS="CDOXQONPND365K6MHR3QBSVVTC3MKR44ORK6TI2GQXUXGGAS5SNDAYRI"
XLM_PHO_STAKE_ADDRESS="CBRGNWGAC25CPLMOAMR7WBPOF5QTFA5RYXQH4DEJ4K65G2QFLTLMW7RO"
XLM_USDC_STAKE_ADDRESS="CAF3UJ45ZQJP6USFUIMVMGOUETUTXEC35R2247VJYIVQBGKTKBZKNBJ3"
XLM_EURC_STAKE_ADDRESS="CDEQYRWFU3IHPRR6H6VOQRUU3JFS6DTUYUL4YAQSD3ALB5IPBTEOZUFM"
USDC_VEUR_STAKE_ADDRESS="CCP653KENMYCAYQ3PHJDT6PITMG4XYKVWV3OEDDCOAOS6Z4GOMXGYH3Z"
USDC_VCHF_STAKE_ADDRESS="CCIWIW6ESCCCFMEI5QOSUHDKTMBEMRJ22F7GPYNRKM2UI2FH6WYUKOUU"
XLM_USDX_STAKE_ADDRESS="CBULEXIMZ5C4CSUPZ4E5LXATWDZNS6MDM2A57DAUD5GXSUG4IWKLOSOC"
EURX_USDC_STAKE_ADDRESS="CD2YKNPX3JPTGDANJRPEJS42MPQLEVUVVRZKJYLLUSPJKQJA7LUANBO4"
XLM_EURX_STAKE_ADDRESS="CDBMVFP7KJXW3YEFSLOU5GYUQHHJJI7QPZJPCSPDK6HHBCBZAMCHS2QY"
XLM_GBPX_STAKE_ADDRESS="CDH6JILIADIC5SKE6OZJAYV3GM62RTR4O54OMVNP4ZOK4HH4J2JWJPVW"
GBPX_USDC_STAKE_ADDRESS="CBDCTYZSZIOWCK5IGCQZNFUOJ53KMPYG2MG7GMVGE3A2LEYCFTDYYZ3S"

stakes=(
    PHO_USDC_STAKE_ADDRESS
    XLM_PHO_STAKE_ADDRESS
    XLM_USDC_STAKE_ADDRESS
    XLM_EURC_STAKE_ADDRESS
    USDC_VEUR_STAKE_ADDRESS
    USDC_VCHF_STAKE_ADDRESS
    XLM_USDX_STAKE_ADDRESS
    EURX_USDC_STAKE_ADDRESS
    XLM_EURX_STAKE_ADDRESS
    XLM_GBPX_STAKE_ADDRES
    GBPX_USDC_STAKE_ADDRESS
)

upgrade_stellar_contract() {
  local contract_id="$1"
  local account="$2"
  local new_wasm_hash="$3"

  if [[ -z "$pool_id" || -z "$account" || -z "$new_wasm_hash" ]]; then
    echo "Error: Missing required parameters (pool_id, account, new_wasm_hash)."
    return 1
  fi

  echo "Processing contract id: $contract_id"

  echo "Building..."
  built=$(stellar contract invoke \
    --id "$contract_id" \
    --source-account "$account" \
    --rpc-url https://mainnet.sorobanrpc.com \
    --network-passphrase "Public Global Stellar Network ; September 2015" \
    -- \
    upgrade \
    --new_wasm_hash "$new_wasm_hash" \
    --build-only) || { echo "Error: Build failed."; return 1; }

  echo "Simulate..."
  simulated=$(stellar tx simulate \
    --source-account "$account" \
    --rpc-url https://mainnet.sorobanrpc.com \
    --network-passphrase "Public Global Stellar Network ; September 2015" \
    "$built") || { echo "Error: Simulation failed."; return 1; }

  echo "Sign..."
  signed=$(stellar tx sign \
    --rpc-url https://mainnet.sorobanrpc.com \
    --network-passphrase "Public Global Stellar Network ; September 2015" \
    --sign-with-key "$account" \
    "$simulated") || { echo "Error: Signing failed."; return 1; }

  echo "Send!"
  stellar tx send --quiet \
    --rpc-url https://mainnet.sorobanrpc.com \
    --network-passphrase "Public Global Stellar Network ; September 2015" \
    "$signed" || { echo "Error: Sending failed."; return 1; }

  echo "Transaction sent successfully for contract: $contract_id"
}


echo "Build and optimize the contracts...";

make build > /dev/null
cd target/wasm32-unknown-unknown/release

echo "Contracts compiled."
echo "Optimize contracts..."

soroban contract optimize --wasm phoenix_factory.wasm
soroban contract optimize --wasm phoenix_pool.wasm
soroban contract optimize --wasm phoenix_stake.wasm
soroban contract optimize --wasm phoenix_multihop.wasm
soroban contract optimize --wasm phoenix_vesting.wasm

echo "Contracts optimized."

echo "Uploading latest factory wasm..."
NEW_FACTORY_WASM_HASH = $(stellar contract upload \
    --wasm ../target/wasm32-unknown-unknown/release/phoenix_factory.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

echo "Uploading latest multihop wasm..."
NEW_MULTIHOP_WASM_HASH = $(stellar contract upload \
    --wasm ../target/wasm32-unknown-unknown/release/phoenix_multihop.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

echo "Uploading latest pool wasm..."
NEW_POOL_WASM_HASH = $(stellar contract upload \
    --wasm ../target/wasm32-unknown-unknown/release/phoenix_pool.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

echo "Uploading latest stake wasm..."
NEW_STAKE_WASM_HASH = $(stellar contract upload \
    --wasm ../target/wasm32-unknown-unknown/release/phoenix_stake.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

echo "Uploading latest vesting wasm..."
NEW_VESTING_WASM_HASH = $(stellar contract upload \
    --wasm ../target/wasm32-unknown-unknown/release/phoenix_vesting.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

echo "Updating factory contract..."
upgrade_stellar_contract $FACTORY_ADDRESS $ACCOUNT $NEW_FACTORY_WASM_HASH
echo "Updated factory contract..."

echo "Updating multihop contract..."
upgrade_stellar_contract $MULTIHOP_ADDRESS $ACCOUNT $NEW_MULTIHOP_WASM_HASH
echo "Updated multihop contract..."

echo "Updating pools..."
for pool in "${pools[@]}"; do
  echo "Will update $pool"
  pool_id="${!pool}"
  upgrade_stellar_contract $pool_id $ACCOUNT $NEW_POOL_WASM_HASH
  echo "Done updating $pool"
done
echo "Updated all pools"


echo "Updating stake contracts..."
for stake in "${stakes[@]}"; do
  echo "Will update $stake"
  stake_id="${!stake}"
  upgrade_stellar_contract $stake_id $ACCOUNT $NEW_STAKE_WASM_HASH
  echo "Done updating $stake"
done
echo "Updated all staking contracts"

echo "Updating vesting contract..."
upgrade_stellar_contract $VESTING_ADDRESS $ACCOUNT $NEW_VESTING_WASM_HASH
echo "Updated vesting contract..."
