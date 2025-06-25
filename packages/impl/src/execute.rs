use cosmwasm_std::{to_json_binary, Addr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, WasmMsg};
use cw20::{Cw20Contract, Cw20ExecuteMsg};
use cw721::Cw721ExecuteMsg;

use crate::error::ContractError;
use crate::msg::EvacuateAsset;

pub struct ExecuteContext<'a> {
  pub deps: DepsMut<'a>,
  pub env: Env,
  pub info: MessageInfo,
}

pub enum Token {
  /// Native token address
  Native(String),
  /// Cw20 token address
  Cw20(String),
}

pub fn evacuate(
  ctx: &ExecuteContext,
  lp_token: Token,
  request: EvacuateAsset,
  recipient: Addr,
) -> Result<Vec<CosmosMsg>, ContractError> {
  match request {
    EvacuateAsset::Native {} => {
      let mut messages: Vec<CosmosMsg> = vec![];
      let balances = ctx.deps.querier
        .query_all_balances(ctx.env.contract.address.clone())?;

      let lp_token = if let Token::Native(lp_token) = lp_token {
        Some(lp_token)
      } else {
        None
      };

      let balances = balances
        .iter()
        .filter(|balance| {
          if let Some(lp_token) = &lp_token {
            &balance.denom != lp_token
          } else {
            true
          }
        });
      for balance in balances {
        messages.push(BankMsg::Send {
          to_address: recipient.to_string(),
          amount: vec![balance.clone()],
        }.into());
      }
      return Ok(messages)
    }
    EvacuateAsset::Cw20 { contract } => {
      let mut messages: Vec<CosmosMsg> = vec![];
      let contract = ctx.deps.api.addr_validate(&contract)?;
      let contract = Cw20Contract(contract);
      let balance = Cw20Contract::balance(&contract, &ctx.deps.querier, ctx.env.contract.address.clone())?;

      if let Token::Cw20(lp_token) = lp_token {
        if lp_token == contract.addr() {
          return Err(ContractError::InvalidFunds("Cannot evacuate the pool token".to_string()))
        }
      }

      messages.push(WasmMsg::Execute {
        contract_addr: contract.addr().to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
          recipient: recipient.to_string(),
          amount: balance,
        })?,
        funds: vec![],
      }.into());
      return Ok(messages)
    }
    EvacuateAsset::Cw721 { contract, token_ids } => {
      let mut messages: Vec<CosmosMsg> = vec![];
      for token_id in token_ids {
        messages.push(WasmMsg::Execute {
          contract_addr: contract.clone(),
          msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: recipient.to_string(),
            token_id: token_id.clone(),
          })?,
          funds: vec![],
        }.into());
      }
      return Ok(messages)
    }
  }
}
