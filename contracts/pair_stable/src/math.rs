use soroban_sdk::Env;

use crate::storage::AmplifierParameters;

pub const MAX_AMP: u64 = 1_000_000;
pub const MAX_AMP_CHANGE: u64 = 10;
pub const MIN_AMP_CHANGING_TIME: u64 = 86400;
pub const AMP_PRECISION: u64 = 100;

/// Compute the current pool amplification coefficient (AMP).
pub(crate) fn compute_current_amp(env: &Env, amp_params: &AmplifierParameters) -> u64 {
    let block_time = env.ledger().timestamp();
    if block_time < amp_params.next_amp_time {
        let elapsed_time: u128 = block_time.saturating_sub(amp_params.init_amp_time).into();
        let time_range = amp_params
            .next_amp_time
            .saturating_sub(amp_params.init_amp_time);
        let init_amp = amp_params.init_amp as u128;
        let next_amp = amp_params.next_amp as u128;

        if next_amp > init_amp {
            let amp_range = next_amp - init_amp;
            let res = init_amp + (amp_range * elapsed_time) / time_range as u128;
            res as u64
        } else {
            let amp_range = init_amp - next_amp;
            let res = init_amp - (amp_range * elapsed_time) / time_range as u128;
            res as u64
        }
    } else {
        amp_params.next_amp
    }
}
