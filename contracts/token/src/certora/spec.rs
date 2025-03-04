#![no_std]
use soroban_sdk::{Address, Env};

use crate::contract::Token;
use soroban_sdk::token::TokenInterface;

use cvlr::asserts::{cvlr_assert, cvlr_assume, cvlr_satisfy};
use cvlr_soroban_derive::rule;

use certora_soroban::{certora_print_i64, is_auth, CERTORA_calltrace_print_c_i64};

// Sunbeam specs

// Exercise 0
#[rule]
fn sanity(e: Env, addr: Address) {
    let balance = Token::balance(e, addr);
    cvlr_satisfy!(true);
}

// // Exercise 1
// #[rule]
// fn init_balance(e: Env, addr: Address) {
//     // Your property here
// }

#[rule]
fn init_balance(e: Env, addr: Address) {
    // precondition macro
    cvlr_assume!(!e.storage().persistent().has(&addr));
    let balance = Token::balance(e, addr);
    // use this macro to see additional information in the calltrace
    certora_print_i64!("value of balance is:", balance as i64);
    // postcondition macro
    cvlr_assert!(balance == 1);
}

// Exercise 2
#[rule]
fn transfer_is_correct(e: Env, to: Address, from: Address, amount: i64) {
    // Your property here
}

// Exercise 2
#[rule]
fn transfer_no_effect_on_other(e: Env, amount: i64, from: Address, to: Address, other: Address) {
    // Your property here
}

// Exercise 3
#[rule]
fn transfer_fails_if_low_balance(e: Env, to: Address, from: Address, amount: i64) {
    // Your property here
}

// Exercise 4
// Add your rules for `mint` and `burn` here!
