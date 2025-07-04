use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("Unauthorized")]
  Unauthorized {},

  #[error("Invalid funds: {0}")]
  InvalidFunds(String),

  #[error("{0}")]
  Generic(String),
}
