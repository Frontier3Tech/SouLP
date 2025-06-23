#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

  let state = State {
    pool: msg.pool,
    evacuate_address: info.sender.to_string(),
  };

  STATE.save(deps.storage, &state)?;

  Ok(Response::new()
    .add_attribute("method", "instantiate")
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
  use cosmwasm_std::coins;

  #[test]
  fn test_instantiate() {
    let mut deps = mock_dependencies();

    let pool = "pool_token_address".to_string();
    let creator = "creator_address".to_string();

    let msg = InstantiateMsg { pool: pool.clone() };
    let info = mock_info(&creator, &coins(1000, "earth"));

    // Call instantiate
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Check response
    assert_eq!(0, res.messages.len());
    assert_eq!(1, res.attributes.len());
    assert_eq!("method", res.attributes[0].key);
    assert_eq!("instantiate", res.attributes[0].value);

    // Check state was saved correctly
    let state = STATE.load(deps.as_ref().storage).unwrap();
    assert_eq!(state.pool, pool);
    assert_eq!(state.evacuate_address, creator);
  }
}
