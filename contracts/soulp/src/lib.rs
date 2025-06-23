pub mod contract;
pub mod error;
pub mod exec;
pub mod msg;
pub mod query;
pub mod state;

pub use crate::error::ContractError;
pub type ContractResult<T> = std::result::Result<T, ContractError>;
