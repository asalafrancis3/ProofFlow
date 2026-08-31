use soroban_sdk::{Env, String};

use crate::errors::Error;

pub fn validate_non_empty_string(value: &String) -> Result<(), Error> {
    if value.is_empty() {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

pub fn validate_amount(amount: u128) -> Result<(), Error> {
    if amount == 0 {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

pub fn validate_title(title: &String) -> Result<(), Error> {
    if title.is_empty() || title.len() > 256 {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

pub fn validate_description(desc: &String) -> Result<(), Error> {
    if desc.len() > 4096 {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

pub fn validate_milestone_index(index: u32, total: u32) -> Result<(), Error> {
    if index >= total {
        return Err(Error::InvalidMilestoneIndex);
    }
    Ok(())
}

pub fn current_timestamp(env: &Env) -> u64 {
    env.ledger().timestamp()
}
