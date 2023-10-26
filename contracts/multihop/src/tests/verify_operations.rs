extern crate std;
use crate::{storage::Swap, utils::verify_swap};

use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn verify_operations_should_work() {
    let env = Env::default();

    let token1 = Address::random(&env);
    let token2 = Address::random(&env);
    let token3 = Address::random(&env);
    let token4 = Address::random(&env);

    let swap1 = Swap {
        offer_asset: token1.clone(),
        ask_asset: token2.clone(),
    };
    let swap2 = Swap {
        offer_asset: token2.clone(),
        ask_asset: token3.clone(),
    };
    let swap3 = Swap {
        offer_asset: token3.clone(),
        ask_asset: token4.clone(),
    };

    let operations = vec![&env, swap1, swap2, swap3];

    verify_swap(&operations);
}

#[test]
#[should_panic(expected = "Multihop: Swap: Provided bad swap order")]
fn verify_operations_should_fail_when_bad_order_provided() {
    let env = Env::default();

    let token1 = Address::random(&env);
    let token2 = Address::random(&env);
    let token3 = Address::random(&env);
    let token4 = Address::random(&env);
    let token5 = Address::random(&env);
    let token6 = Address::random(&env);

    let swap1 = Swap {
        offer_asset: token1.clone(),
        ask_asset: token2.clone(),
    };
    let swap2 = Swap {
        offer_asset: token3.clone(),
        ask_asset: token4.clone(),
    };
    let swap3 = Swap {
        offer_asset: token5.clone(),
        ask_asset: token6.clone(),
    };

    let operations = vec![&env, swap1, swap2, swap3];

    verify_swap(&operations);
}
