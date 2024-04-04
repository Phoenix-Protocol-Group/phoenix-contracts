use soroban_sdk::{log, panic_with_error, Address, Env};

use crate::{
    error::ContractError,
    storage::{get_config, get_delegated, get_vesting, remove_vesting},
    token_contract,
};

pub fn transfer(
    env: &Env,
    from: &Address,
    to: &Address,
    amount: i128,
    vesting_amount: i128,
) -> Result<(), ContractError> {
    let vesting_token_address = get_config(&env).token_info.address;

    let vestint_token_client = token_contract::Client::new(&env, &vesting_token_address);

    let sender_balance = vestint_token_client.balance(&from);

    if let Some(remainder) = sender_balance.checked_sub(amount) {
        if vesting_amount > remainder {
            log!(
                &env,
                "Vesting: Transfer Token: Remaining amount must be at least equal to vested amount"
            );
            panic_with_error!(env, ContractError::CantMoveVestingTokens);
        }
        vestint_token_client.transfer(&from, &to, &amount);
    } else {
        log!(
            &env,
            "Vesting: Transfer Token: Not enough balance to transfer"
        );
        panic_with_error!(env, ContractError::NotEnoughBalance);
    }

    Ok(())
}

/// This reduces the account by the given amount, but it also checks the vesting schedule to
/// ensure there is enough liquidity to do the transfer.
/// (Always use this to enforce the vesting schedule)
pub fn deduct_coints(env: &Env, sender: &Address, amount: i128) -> Result<i128, ContractError> {
    let vesting_amount = get_vesting(env, sender)
        .curve
        .value(env.ledger().timestamp()) as i128;

    if vesting_amount <= 0 {
        remove_vesting(env, sender);
    }

    let delegated = get_delegated(env, sender);
    let token_client = token_contract::Client::new(&env, &get_config(env).token_info.address);
    let balance = token_client.balance(sender);

    let remainder = (balance + delegated)
        .checked_sub(amount)
        .ok_or(ContractError::NotEnoughBalance)?;

    if vesting_amount > remainder {
        log!(
            &env,
            "Vesting: Deduct Coins: Remaining amount must be at least equal to vested amount"
        );
        panic_with_error!(env, ContractError::CantMoveVestingTokens);
    }

    token_client.transfer(sender, &env.current_contract_address(), &amount);

    Ok(vesting_amount)
}
