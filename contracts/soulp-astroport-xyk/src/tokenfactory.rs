//! NOTE: This assumes the Osmosis TokenFactory. Some networks like Injective and Sei have deviating
//! implementations with differing type URLs and require specialized handling (e.g. cfg directives).
use prost::Message;

use cosmwasm_std::{Addr, CosmosMsg, Env, Uint128};
#[derive(Clone, PartialEq, Message)]
pub struct Coin {
  #[prost(string, tag = "1")]
  pub denom: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub amount: ::prost::alloc::string::String,
}

impl Coin {
  pub const TYPE_URL: &'static str = "/cosmos.bank.v1beta1.Coin";
}

#[derive(Clone, PartialEq, Message)]
pub struct MsgCreateDenom {
  #[prost(string, tag = "1")]
  pub sender: String,
  #[prost(string, tag = "2")]
  pub subdenom: String,
}

impl MsgCreateDenom {
  pub const TYPE_URL: &'static str = "/osmosis.tokenfactory.v1beta1.MsgCreateDenom";
}

impl Into<CosmosMsg> for MsgCreateDenom {
  fn into(self) -> CosmosMsg {
    CosmosMsg::Stargate {
      type_url: Self::TYPE_URL.to_string(),
      value: self.encode_to_vec().into(),
    }
  }
}

#[derive(Clone, PartialEq, Message)]
pub struct MsgMint {
  #[prost(string, tag = "1")]
  pub sender: ::prost::alloc::string::String,
  #[prost(message, optional, tag = "2")]
  pub amount: Option<Coin>,
  #[prost(string, tag = "3")]
  pub mint_to_address: String,
}

impl MsgMint {
  pub const TYPE_URL: &'static str = "/osmosis.tokenfactory.v1beta1.MsgMint";

  pub fn subdenom(env: &Env, subdenom: &dyn AsRef<str>, amount: Uint128, mint_to_address: Addr) -> MsgMint {
    let subdenom = subdenom.as_ref();
    MsgMint {
      sender: env.contract.address.to_string(),
      amount: Some(Coin {
        denom: format!("factory/{}/{}", env.contract.address, subdenom),
        amount: amount.to_string(),
      }),
      mint_to_address: mint_to_address.to_string(),
    }
  }
}

impl Into<CosmosMsg> for MsgMint {
  fn into(self) -> CosmosMsg {
    CosmosMsg::Stargate {
      type_url: Self::TYPE_URL.to_string(),
      value: self.encode_to_vec().into(),
    }
  }
}