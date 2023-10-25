extern crate std;
use crate::{storage::Swap, utils::verify_operations};

use soroban_sdk::{testutils::Address as _, vec, Address, Env, Symbol, Vec};

#[test]
#[should_panic(expected = "Multihop: Swap: Operations empty")]
fn verify_operations_should_fail_when_empty_operations() {
    let env = Env::default();
    let empty_vec = Vec::<Swap>::new(&env);

    if let Some(err) = verify_operations(&env, &empty_vec) {
        if err.eq(&Symbol::new(&env, "operations_empty")) {
            panic!("Multihop: Swap: Operations empty")
        } else {
            panic!("Multihop: Swap: Provided bad swap order")
        }
    };
}

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

    verify_operations(&env, &operations);
}

#[test]
#[should_panic(expected = "Multihop: Provided bad swap order")]
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

    if let Some(err) = verify_operations(&env, &operations) {
        if err.eq(&Symbol::new(&env, "operations_empty")) {
            panic!("Multihop: Swap: Operations empty")
        } else {
            panic!("Multihop: Provided bad swap order")
        }
    };
}
