// Constants for storage bump amounts
pub const DAY_IN_LEDGERS: u32 = 17280;

// target TTL for the contract instance and its code.
// When a TTL extension is triggered the instance's TTL is reset to this value (7 days of ledger units).
pub const INSTANCE_TARGET_TTL: u32 = 7 * DAY_IN_LEDGERS;
// if the current instance TTL falls below this threshold (i.e., less than 6 days of ledger units), the TTL extension mechanism will refresh it to INSTANCE_TARGET_TTL.
pub const INSTANCE_RENEWAL_THRESHOLD: u32 = INSTANCE_TARGET_TTL - DAY_IN_LEDGERS;

// when TTL extension (if the current TTL is below its renewal threshold), the persistent TTL is set to this value (30 days of ledger units).
pub const PERSISTENT_TARGET_TTL: u32 = 30 * DAY_IN_LEDGERS;
// if the current persistent TTL drops below this threshold (i.e., less than 29 days of ledger units), the TTL extension will bump it back to PERSISTENT_TARGET_TTL.
pub const PERSISTENT_RENEWAL_THRESHOLD: u32 = PERSISTENT_TARGET_TTL - DAY_IN_LEDGERS;
