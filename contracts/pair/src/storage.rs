use soroban_sdk::{contracttype, Address, ConversionError, Env, RawVal, Symbol, TryFromVal};

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    TotalShares = 0,
    ReserveA = 1,
    ReserveB = 2,
}

impl TryFromVal<Env, DataKey> for RawVal {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Config {
    pub token_a: Address,
    pub token_b: Address,
    pub share_token: Address,
}
const CONFIG: Symbol = Symbol::short("CONFIG");

pub fn get_config(env: &Env) -> Config {
    env.storage().get(&CONFIG).unwrap().unwrap()
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().set(&CONFIG, &config);
}
