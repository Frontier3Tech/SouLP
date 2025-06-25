#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

use crate::{msg::QueryMsg, state::{State, STATE}};

struct QueryCtx<'a> {
  deps: Deps<'a>,
  env: Env,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
  let ctx = QueryCtx { deps, env };
  match msg {
    QueryMsg::State {} => to_json_binary(&state(ctx)?),
    QueryMsg::TokenAddress {} => to_json_binary(&token_address(ctx)?),
  }
}

fn state(ctx: QueryCtx) -> StdResult<State> {
  let state = STATE.load(ctx.deps.storage)?;
  Ok(state)
}

fn token_address(ctx: QueryCtx) -> StdResult<String> {
  Ok(format!("factory/{}/SouLP", ctx.env.contract.address))
}