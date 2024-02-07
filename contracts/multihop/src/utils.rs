use soroban_sdk::Vec;

use crate::storage::Swap;

pub fn verify_swap(operations: &Vec<Swap>) {
    for (current, next) in operations.iter().zip(operations.iter().skip(1)) {
        if current.ask_asset != next.offer_asset {
            panic!("Multihop: Swap: Provided bad swap order");
        }
    }
}

pub fn verify_reverse_swap(operations: &Vec<Swap>) {
    for (current, next) in operations.iter().zip(operations.iter().skip(1)) {
        if current.offer_asset != next.ask_asset {
            panic!("Multihop: Reverse swap: Provided bad swap order");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage::Swap, utils::verify_swap};

    use soroban_sdk::{testutils::Address as _, vec, Address, Env};

    #[test]
    fn verify_operations_in_swap_should_work() {
        let env = Env::default();

        let token1 = Address::generate(&env);
        let token2 = Address::generate(&env);
        let token3 = Address::generate(&env);
        let token4 = Address::generate(&env);

        let swap1 = Swap {
            offer_asset: token1.clone(),
            ask_asset: token2.clone(),
            max_belief_price: None::<i64>,
        };
        let swap2 = Swap {
            offer_asset: token2.clone(),
            ask_asset: token3.clone(),
            max_belief_price: None::<i64>,
        };
        let swap3 = Swap {
            offer_asset: token3.clone(),
            ask_asset: token4.clone(),
            max_belief_price: None::<i64>,
        };

        let operations = vec![&env, swap1, swap2, swap3];

        verify_swap(&operations);
    }

    #[test]
    fn verify_operations_in_reverse_swap_should_work() {
        let env = Env::default();

        let token1 = Address::generate(&env);
        let token2 = Address::generate(&env);
        let token3 = Address::generate(&env);
        let token4 = Address::generate(&env);

        let swap1 = Swap {
            offer_asset: token3.clone(),
            ask_asset: token4.clone(),
            max_belief_price: None::<i64>,
        };
        let swap2 = Swap {
            offer_asset: token2.clone(),
            ask_asset: token3.clone(),
            max_belief_price: None::<i64>,
        };
        let swap3 = Swap {
            offer_asset: token1.clone(),
            ask_asset: token2.clone(),
            max_belief_price: None::<i64>,
        };

        let operations = vec![&env, swap1, swap2, swap3];

        verify_reverse_swap(&operations);
    }

    #[test]
    #[should_panic(expected = "Multihop: Swap: Provided bad swap order")]
    fn verify_operations_should_fail_when_bad_order_provided() {
        let env = Env::default();

        let token1 = Address::generate(&env);
        let token2 = Address::generate(&env);
        let token3 = Address::generate(&env);
        let token4 = Address::generate(&env);

        let swap1 = Swap {
            offer_asset: token1.clone(),
            ask_asset: token2.clone(),
            max_belief_price: None::<i64>,
        };
        let swap2 = Swap {
            offer_asset: token3.clone(),
            ask_asset: token4.clone(),
            max_belief_price: None::<i64>,
        };

        let operations = vec![&env, swap1, swap2];

        verify_swap(&operations);
    }

    #[test]
    #[should_panic(expected = "Multihop: Reverse swap: Provided bad swap order")]
    fn verify_operations_reverse_swap_should_fail_when_bad_order_provided() {
        let env = Env::default();

        let token1 = Address::generate(&env);
        let token2 = Address::generate(&env);
        let token3 = Address::generate(&env);
        let token4 = Address::generate(&env);

        let swap1 = Swap {
            offer_asset: token1.clone(),
            ask_asset: token2.clone(),
            max_belief_price: None::<i64>,
        };
        let swap2 = Swap {
            offer_asset: token3.clone(),
            ask_asset: token4.clone(),
            max_belief_price: None::<i64>,
        };

        let operations = vec![&env, swap1, swap2];

        verify_reverse_swap(&operations);
    }
}
