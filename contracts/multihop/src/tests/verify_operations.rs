extern crate std;
use crate::{storage::Swap, utils::verify_operations};

use soroban_sdk::{testutils::Address as _, vec, Address, Env, Vec};

#[test]
#[should_panic(expected = "Multihop: Operations empty")]
fn verify_operations_should_fail_when_empty_operations() {
    let env = Env::default();
    let empty_vec = Vec::<Swap>::new(&env);

    verify_operations(&empty_vec)
}

#[test]
fn verify_operations_should_work() {
    let env = Env::default();

    let mut token1 = Address::random(&env);
    let mut token2 = Address::random(&env);
    let mut token3 = Address::random(&env);
    let mut token4 = Address::random(&env);

    if token2 < token1 {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token3 < token2 {
        std::mem::swap(&mut token2, &mut token3);
    }

    if token4 < token3 {
        std::mem::swap(&mut token4, &mut token3);
    }

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

    verify_operations(&operations);
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

    verify_operations(&operations);
}
