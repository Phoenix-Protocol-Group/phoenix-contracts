# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK="mainnet"

# Fetch the admin's address
ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)


FACTORY="CB4SVAWJA6TSRNOJZ7W2AWFW46D5VR4ZMFZKDIKXEINZCZEGZCJZCKMI"

# https://stellar.expert/explorer/public/asset/XLM
XLM="CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA"

# https://stellar.expert/explorer/public/asset/EURC-GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2
EURC="CDTKPWPLOURQA2SGTKTUQOWRCBZEORB4BWBOMJ3D3ZTQQSGE5F6JBQLV"

# https://stellar.expert/explorer/public/asset/USDC-GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN
USDC="CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75"

# https://stellar.expert/explorer/public/asset/USDx-GAVH5ZWACAY2PHPUG4FL3LHHJIYIHOFPSIUGM2KHK25CJWXHAV6QKDMN-1
USDX="CDIKURWHYS4FFTR5KOQK6MBFZA2K3E26WGBQI6PXBYWZ4XIOPJHDFJKP"

# https://stellar.expert/explorer/public/asset/EURx-GAVH5ZWACAY2PHPUG4FL3LHHJIYIHOFPSIUGM2KHK25CJWXHAV6QKDMN
EURX="CBN3NCJSMOQTC6SPEYK3A44NU4VS3IPKTARJLI3Y77OH27EWBY36TP7U"

# https://stellar.expert/explorer/public/asset/GBPx-GAVH5ZWACAY2PHPUG4FL3LHHJIYIHOFPSIUGM2KHK25CJWXHAV6QKDMN-1
GBPX="CBCO65UOWXY2GR66GOCMCN6IU3Y45TXCPBY3FLUNL4AOUMOCKVIVV6JC"

# USDC<>EURx
# EURx<>XLM
# GBPx<>XLM
# GBPx<>USDC

# ====================

echo "EURX<>USDC"
echo
soroban contract invoke \
    --id CC6MJZN3HFOJKXN42ANTSCLRFOMHLFXHWPNAX64DQNUEBDMUYMPHASAV \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 10000000 --desired_b 10400000

echo "XLM<>EURX"
echo
soroban contract invoke \
    --id CB5QUVK5GS3IU23TMFZQ3P5J24YBBZP5PHUQAEJ2SP5K55PFTJRUQG2L \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 28900000 --desired_b 10000000

echo "XLM<>GBPX"
echo
soroban contract invoke \
    --id CCKOC2LJTPDBKDHTL3M5UO7HFZ2WFIHSOKCELMKQP3TLCIVUBKOQL4HB \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 34800000 --desired_b 10000000

echo "GBPX<>USDC"
echo
soroban contract invoke \
    --id CCUCE5H5CKW3S7JBESGCES6ZGDMWLNRY3HOFET3OH33MXZWKXNJTKSM3 \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    provide_liquidity --sender $ADMIN_ADDRESS --desired_a 10000000 --desired_b 12500000

# soroban contract invoke \
#     --id $FACTORY \
#     --source $IDENTITY_STRING \
#     --network $NETWORK \
#     -- \
#     create_liquidity_pool \
#     --sender $ADMIN_ADDRESS \
#     --lp_init_info "{ \"admin\": \"${ADMIN_ADDRESS}\", \"swap_fee_bps\": 100, \"fee_recipient\": \"${ADMIN_ADDRESS}\", \"max_allowed_slippage_bps\": 10000, \"default_slippage_bps\": 3000, \"max_allowed_spread_bps\": 10000, \"max_referral_bps\": 5000, \"token_init_info\": { \"token_a\": \"${GBPX}\", \"token_b\": \"${USDC}\" }, \"stake_init_info\": { \"min_bond\": \"100\", \"min_reward\": \"100\", \"max_distributions\": 3, \"manager\": \"${ADMIN_ADDRESS}\", \"max_complexity\": 7 } }" \
#     --default_slippage_bps 3000 \
#     --max_allowed_fee_bps 10000 \
#     --share_token_name "GBPXUSDCST" \
#     --share_token_symbol "GXUT" \
#     --pool_type 0
