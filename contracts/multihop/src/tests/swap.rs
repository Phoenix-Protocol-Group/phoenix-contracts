use crate::error::ContractError;
use crate::tests::setup::{deploy_multihop_contract, deploy_factory_contract, factory};
use soroban_sdk::arbitrary::std::dbg;
use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, BytesN, Env};

#[test]
fn test_swap() {
    let env = Env::default();
    let admin = Address::random(&env);

    // 1. deploy factory
    // this fails with all the below given client initializations of factory
    // either HostError: Error(Value, InvalidInput) or HostError: Error(Context, MissingValue)

    let factory_addr = deploy_factory_contract(&env, admin.clone());
    let factory_client = factory::Client::new(&env, &factory_addr);
    factory_client.initialize(&admin.clone());

    let admin = factory_client.get_admin();

    //
    // dbg!(admin.clone());
    // 2. create liquidity pool from factory
    // 3. use the swap method of multihop
    // 4. check if it goes according to plan

    // let multihop = deploy_multihop_contract(&env, admin, factory_client.address);
    //
    // let recipient = Address::random(&env);
    // let swap1 = Swap {
    //     ask_asset: Address::random(&env),
    //     offer_asset: Address::random(&env),
    // };
    //
    // let swap2 = Swap {
    //     ask_asset: Address::random(&env),
    //     offer_asset: Address::random(&env),
    // };
    // let swap3 = Swap {
    //     ask_asset: Address::random(&env),
    //     offer_asset: Address::random(&env),
    // };
    //
    // let swap_vec = vec![&env, swap1, swap2, swap3];

    // WHY WOULD &swap_vec BE MARKED BY THE COMPILER LIKE THAT...
    // multihop.swap(&recipient, &swap_vec, &5i128);
}

#[test]
fn test_swap_should_return_err() {
    let env = Env::default();
    let admin = Address::random(&env);
    let factory = Address::random(&env);

    let multihop = deploy_multihop_contract(&env, admin, factory);

    let recipient = Address::random(&env);

    let swap_vec = vec![&env];

    assert_eq!(
        multihop.try_swap(&recipient, &swap_vec, &5i128),
        Err(Ok(ContractError::OperationsEmpty))
    );
}
