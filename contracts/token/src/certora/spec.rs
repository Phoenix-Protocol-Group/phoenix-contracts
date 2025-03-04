use soroban_sdk::{Address, Env};

use crate::contract::Token;
use soroban_sdk::token::TokenInterface;

use cvlr::asserts::cvlr_satisfy;
use cvlr_soroban_derive::rule;

#[rule]
fn sanity(e: Env, addr: Address) {
    let _ = Token::balance(e, addr);
    cvlr_satisfy!(true);
}
