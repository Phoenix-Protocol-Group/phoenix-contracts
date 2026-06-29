#!/bin/bash

NETWORK="mainnet"
SOURCE="futurenetacc"
NEW_STAKE_WASM_HASH="649715de2b14df3c34a560d9ffb01deeefe029404cd3e9ab184ba22702cbed79"

FACTORY_ADDRESS="CB4SVAWJA6TSRNOJZ7W2AWFW46D5VR4ZMFZKDIKXEINZCZEGZCJZCKMI"

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

STAKE_CONTRACTS=(
    # "CDOXQONPND365K6MHR3QBSVVTC3MKR44ORK6TI2GQXUXGGAS5SNDAYRI"  # PHO_USDC_STAKE_ADDRESS
    # "CBRGNWGAC25CPLMOAMR7WBPOF5QTFA5RYXQH4DEJ4K65G2QFLTLMW7RO"  # XLM_PHO_STAKE_ADDRESS
    # "CAF3UJ45ZQJP6USFUIMVMGOUETUTXEC35R2247VJYIVQBGKTKBZKNBJ3"  # XLM_USDC_STAKE_ADDRESS
    "CDEQYRWFU3IHPRR6H6VOQRUU3JFS6DTUYUL4YAQSD3ALB5IPBTEOZUFM"  # XLM_EURC_STAKE_ADDRESS
    # "CCP653KENMYCAYQ3PHJDT6PITMG4XYKVWV3OEDDCOAOS6Z4GOMXGYH3Z"  # USDC_VEUR_STAKE_ADDRESS
    # "CCIWIW6ESCCCFMEI5QOSUHDKTMBEMRJ22F7GPYNRKM2UI2FH6WYUKOUU"  # USDC_VCHF_STAKE_ADDRESS
    "CBULEXIMZ5C4CSUPZ4E5LXATWDZNS6MDM2A57DAUD5GXSUG4IWKLOSOC"  # XLM_USDX_STAKE_ADDRESS
    "CD2YKNPX3JPTGDANJRPEJS42MPQLEVUVVRZKJYLLUSPJKQJA7LUANBO4"  # EURX_USDC_STAKE_ADDRESS
    "CDBMVFP7KJXW3YEFSLOU5GYUQHHJJI7QPZJPCSPDK6HHBCBZAMCHS2QY"  # XLM_EURX_STAKE_ADDRESS
    "CDH6JILIADIC5SKE6OZJAYV3GM62RTR4O54OMVNP4ZOK4HH4J2JWJPVW"  # XLM_GBPX_STAKE_ADDRESS
    "CBDCTYZSZIOWCK5IGCQZNFUOJ53KMPYG2MG7GMVGE3A2LEYCFTDYYZ3S"  # GBPX_USDC_STAKE_ADDRESS
)

echo "Upgrading ${#STAKE_CONTRACTS[@]} stake contracts..."
echo "=================================================="

for i in "${!STAKE_CONTRACTS[@]}"; do
    CONTRACT_ID="${STAKE_CONTRACTS[$i]}"
    echo ""
    echo "[$((i+1))/${#STAKE_CONTRACTS[@]}] Processing stake contract: $CONTRACT_ID"
    echo ""

    # Call restore.sh with hardcoded pho-test-distributor2 source
    echo "Running restore.sh with pho-test-distributor2..."
    ./restore.sh "$CONTRACT_ID" pho-test-distributor2

    if [ $? -ne 0 ]; then
        echo "Error: Failed to run restore.sh for $CONTRACT_ID"
        break
    fi

    echo "Restore completed successfully"
    echo ""

    # Step 1: Build the transaction
    echo "Step 1: Building transaction..."
    BUILD_OUTPUT=$(soroban contract invoke \
        --id "$CONTRACT_ID" \
        --source "$SOURCE" \
        --network "$NETWORK" \
        --build-only \
        -- \
        update --new_wasm_hash "$NEW_STAKE_WASM_HASH")

    if [ $? -ne 0 ]; then
        echo "Error: Failed to build transaction for $CONTRACT_ID"
        break
    fi

    echo "Build output: $BUILD_OUTPUT"
    echo ""

    # Step 2: Simulate the transaction
    echo "Step 2: Simulating transaction..."
    SIMULATE_OUTPUT=$(soroban tx simulate \
        --source "$SOURCE" \
        --network "$NETWORK" \
        -- \
        "$BUILD_OUTPUT")

    if [ $? -ne 0 ]; then
        echo "Error: Failed to simulate transaction for $CONTRACT_ID"
        break
    fi

    echo "Simulate output: $SIMULATE_OUTPUT"
    echo ""

    # Step 3: Sign the transaction
    echo "Step 3: Signing transaction..."
    SIGN_OUTPUT=$(soroban tx sign \
        --sign-with-key "$SOURCE" \
        --network "$NETWORK" \
        -- \
        "$SIMULATE_OUTPUT")

    if [ $? -ne 0 ]; then
        echo "Error: Failed to sign transaction for $CONTRACT_ID"
        break
    fi

    echo "Sign output: $SIGN_OUTPUT"
    echo ""

    echo "Transaction successfully built, simulated, and signed for $CONTRACT_ID!"
    echo "Final signed transaction:"
    echo "$SIGN_OUTPUT"
    echo ""
    echo "=================================================="

    # Wait for two Enter presses before continuing to update_config
    echo "Press Enter twice to continue to update_config for this contract or Ctrl+C to exit..."
    read
    read
    
    # Now call update_config on the same contract
    echo ""
    echo "Running update_config for contract: $CONTRACT_ID"
    echo ""
    
    # Step 1: Build the update_config transaction
    echo "Step 1: Building update_config transaction..."
    BUILD_CONFIG_OUTPUT=$(soroban contract invoke \
        --id "$CONTRACT_ID" \
        --source "$SOURCE" \
        --network "$NETWORK" \
        --build-only \
        -- \
        update_config --manager GAPRPZYCIV3QPMCTWSRDNY64EJMZNCJFUCTJHQDQNW6RJ66TEVEH5UDU)
    
    if [ $? -ne 0 ]; then
        echo "Error: Failed to build update_config transaction for $CONTRACT_ID"
        break
    fi
    
    echo "Build output: $BUILD_CONFIG_OUTPUT"
    echo ""
    
    # Step 2: Simulate the update_config transaction
    echo "Step 2: Simulating update_config transaction..."
    SIMULATE_CONFIG_OUTPUT=$(soroban tx simulate \
        --source "$SOURCE" \
        --network "$NETWORK" \
        -- \
        "$BUILD_CONFIG_OUTPUT")
    
    if [ $? -ne 0 ]; then
        echo "Error: Failed to simulate update_config transaction for $CONTRACT_ID"
        break
    fi
    
    echo "Simulate output: $SIMULATE_CONFIG_OUTPUT"
    echo ""
    
    # Step 3: Sign the update_config transaction
    echo "Step 3: Signing update_config transaction..."
    SIGN_CONFIG_OUTPUT=$(soroban tx sign \
        --sign-with-key "$SOURCE" \
        --network "$NETWORK" \
        -- \
        "$SIMULATE_CONFIG_OUTPUT")
    
    if [ $? -ne 0 ]; then
        echo "Error: Failed to sign update_config transaction for $CONTRACT_ID"
        break
    fi
    
    echo "Sign output: $SIGN_CONFIG_OUTPUT"
    echo ""
    
    echo "Update_config transaction successfully built, simulated, and signed for $CONTRACT_ID!"
    echo "Final signed transaction:"
    echo "$SIGN_CONFIG_OUTPUT"
    echo ""
    echo "=================================================="

    # Wait for two Enter presses before continuing to next contract
    if [ $((i+1)) -lt ${#STAKE_CONTRACTS[@]} ]; then
        echo "Press Enter twice to continue to next contract or Ctrl+C to exit..."
        read
        read
    fi
done


echo ""
echo "All stake contracts processed!"
echo "Each signed transaction above needs to be executed by your partner."
