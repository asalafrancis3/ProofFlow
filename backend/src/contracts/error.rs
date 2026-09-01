use std::fmt;

use super::types::ContractErrorCode;

#[derive(Debug)]
pub enum ContractError {
    Contract(ContractErrorCode),
    Rpc(String),
    Serialization(String),
    Network(String),
    NotInitialized,
    Encoding(String),
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::Contract(code) => write!(f, "contract error: {:?}", code),
            ContractError::Rpc(msg) => write!(f, "rpc error: {msg}"),
            ContractError::Serialization(msg) => write!(f, "serialization error: {msg}"),
            ContractError::Network(msg) => write!(f, "network error: {msg}"),
            ContractError::NotInitialized => write!(f, "contract adapter not initialized"),
            ContractError::Encoding(msg) => write!(f, "encoding error: {msg}"),
        }
    }
}

impl std::error::Error for ContractError {}

impl From<ContractErrorCode> for ContractError {
    fn from(code: ContractErrorCode) -> Self {
        ContractError::Contract(code)
    }
}

impl From<serde_json::Error> for ContractError {
    fn from(e: serde_json::Error) -> Self {
        ContractError::Serialization(e.to_string())
    }
}
