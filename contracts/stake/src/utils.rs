use soroban_sdk::contracttype;

// This type has been created because of lack of conversion of Option<T> into ScVal
#[contracttype]
pub enum OptionUint {
    Some(u128),
    None,
}
