use soroban_sdk::{log, panic_with_error, Address, ConversionError, Env, TryFromVal, Val};

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Admin,
}

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

pub fn save_admin(e: &Env, address: &Address) {
    e.storage().persistent().set(&DataKey::Admin, address)
}

pub fn get_admin(e: &Env) -> Address {
    e.storage().persistent().get(&DataKey::Admin).unwrap()
}
