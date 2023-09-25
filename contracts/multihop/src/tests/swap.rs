use crate::error::ContractError;
use crate::storage::Swap;
use crate::tests::setup::{deploy_factory, deploy_multihop_contract};
use soroban_sdk::{
    testutils::Address as _, vec, Address, ConversionError, Env, IntoVal, TryFromVal, Val, Vec,
};
use soroban_sdk::arbitrary::std::dbg;

#[test]
fn test_swap() {
    let env = Env::default();
    let admin = Address::random(&env);
    let factory = deploy_factory(&env, &admin);


    // factory.initialize(&admin);
    // dbg!(factory.get_admin());

    let multihop = deploy_multihop_contract(&env, admin, factory.address);

    let recipient = Address::random(&env);
    let swap1 = Swap {
        ask_asset: Address::random(&env),
        offer_asset: Address::random(&env),
    };

    let swap2 = Swap {
        ask_asset: Address::random(&env),
        offer_asset: Address::random(&env),
    };
    let swap3 = Swap {
        ask_asset: Address::random(&env),
        offer_asset: Address::random(&env),
    };

    let swap_vec = vec![&env, swap1, swap2, swap3];

    // WHY WOULD &swap_vec BE MARKED BY THE COMPILER LIKE THAT...
    multihop.swap(&recipient, &swap_vec, &5i128);
}

#[test]
fn test_swap_should_return_err() {
    let env = Env::default();
    let admin = Address::random(&env);
    let factory = Address::random(&env);

    let multihop = deploy_multihop_contract(&env, admin, factory);

    let recipient = Address::random(&env);

    let swap_vec = vec![&env];

    // WHY WOULD &swap_vec BE MARKED BY THE COMPILER LIKE THAT...
    assert_eq!(
        multihop.try_swap(&recipient, &swap_vec, &5i128),
        Err(Ok(ContractError::OperationsEmpty))
    );
}
