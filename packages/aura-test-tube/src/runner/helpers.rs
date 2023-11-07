use test_tube::account::{FeeSetting, SigningAccount};
use cosmwasm_std::{Coin, Uint128};
use test_tube::runner::error::DecodeError;
use test_tube::runner::result::RunnerResult;
use cosmrs::crypto::secp256k1::SigningKey;
use crate::runner::app::{FEE_DENOM, DEFAULT_GAS_LIMIT};

pub fn init_local_smart_account(address: String, private_key: Vec<u8>) -> RunnerResult<SigningAccount> {
    let signging_key = SigningKey::from_bytes(&private_key).map_err(|e| {
        let msg = e.to_string();
        DecodeError::SigningKeyDecodeError { msg }
    })?;
    Ok(SigningAccount::new(
        address,
        signging_key,
        private_key,
        FeeSetting::Custom { 
            // gas price is 0.025
            amount: Coin { 
                denom: String::from(FEE_DENOM), 
                amount: Uint128::from(50000u128)
            }, 
            gas_limit: DEFAULT_GAS_LIMIT
        },
    ))
}   