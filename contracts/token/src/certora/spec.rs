use soroban_sdk::{Address, Env};

use crate::contract::Token;
use soroban_sdk::token::TokenInterface;

use cvlr::asserts::{cvlr_assert, cvlr_assume, cvlr_satisfy};
use cvlr_soroban_derive::rule;

#[rule]
fn sanity(e: Env, addr: Address) {
    let _ = Token::balance(e, addr);
    cvlr_satisfy!(true);
}

#[rule]
fn transfer_is_correct(e: Env, to: Address, from: Address, amount: i128) {
    cvlr_assume!(
        e.storage().persistent().has(&from) && e.storage().persistent().has(&to) && to != from
    );
    let balance_from_before = Token::balance(e.clone(), from.clone());
    let balance_to_before = Token::balance(e.clone(), to.clone());
    Token::transfer(e.clone(), from.clone(), to.clone(), amount);
    let balance_from_after = Token::balance(e.clone(), from.clone());
    let balance_to_after = Token::balance(e.clone(), to.clone());
    cvlr_assert!(
        (balance_to_after == balance_to_before + amount)
            && (balance_from_after == balance_from_before - amount)
    );
}
