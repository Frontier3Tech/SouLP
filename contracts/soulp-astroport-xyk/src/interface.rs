use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, CustomQuery, Querier, QuerierWrapper, StdResult, WasmMsg, WasmQuery};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::ExecuteMsg;

/// CwTemplateContract is a wrapper around Addr that provides various helper functions to interface
/// with live instances of this contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
  pub fn addr(&self) -> Addr {
    self.0.clone()
  }

  pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
    let msg = to_json_binary(&msg.into())?;
    Ok(
      WasmMsg::Execute {
        contract_addr: self.addr().into(),
        msg,
        funds: vec![],
      }.into()
    )
  }
}