use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
  pub pool: String,
  pub evacuate_address: String,
}

pub const STATE: Item<State> = Item::new("state");
