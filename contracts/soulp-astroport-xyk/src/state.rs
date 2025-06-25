use cosmwasm_std::Decimal;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
  pub pool: String,
  pub evacuate_address: String,
  pub mint_ratio: Decimal,
}

pub const STATE: Item<State> = Item::new("state");
