use soroban_sdk::{log, panic_with_error, Address, Env};

use crate::{error::ContractError, storage::get_config, token_contract};

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
