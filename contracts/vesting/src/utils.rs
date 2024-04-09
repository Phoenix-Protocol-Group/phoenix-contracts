use soroban_sdk::testutils::arbitrary::std::dbg;
use soroban_sdk::{log, panic_with_error, Address, Env};

use crate::{
    error::ContractError,
    storage::{get_config, get_vesting, remove_vesting},
    token_contract,
};

pub fn verify_vesting_and_transfer(
    env: &Env,
    sender: &Address,
    to: &Address,
    amount: i128,
) -> Result<(), ContractError> {
    let vesting_amount = get_vesting(env, sender)
        .curve
        .value(env.ledger().timestamp()) as i128;

    if vesting_amount <= 0 {
        remove_vesting(env, sender);
    }

    let token_client = token_contract::Client::new(env, &get_config(env).token_info.address);
    let balance = token_client.balance(sender);
    let remainder = balance
        .checked_sub(amount)
        .ok_or(ContractError::NotEnoughBalance)?;

    if vesting_amount > remainder {
        log!(
            &env,
            "Vesting: Mixture: Remaining amount must be at least equal to vested amount"
        );
        panic_with_error!(env, ContractError::CantMoveVestingTokens);
    }

    token_client.transfer(sender, to, &amount);

    Ok(())
}
