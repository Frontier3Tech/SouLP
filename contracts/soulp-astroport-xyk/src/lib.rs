pub mod contract;
pub mod exec;
pub mod msg;
pub mod query;
pub mod state;

pub use r#impl::ContractError;
pub type ContractResult<T> = std::result::Result<T, ContractError>;
