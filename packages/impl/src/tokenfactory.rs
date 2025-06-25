use prost::Message;
use cosmwasm_std::{Addr, CosmosMsg, Env, Uint128};

pub trait TFToken {
  /// Address of the owner of the token, usually the contract address
  fn owner(&self) -> Addr;

  /// Subdenom of the token, e.g. "SouLP"
  fn subdenom(&self) -> String;

  /// Full denom of the token, based on `owner` and `subdenom`
  fn denom(&self) -> String;

  /// Create the token, e.g. register the denom on the chain
  fn create(&self) -> Vec<CosmosMsg>;

  /// Mint tokens to a recipient
  fn mint(&self, amount: Uint128, recipient: String) -> Vec<CosmosMsg>;
}

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

pub mod osmosis {
  use super::*;

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

  pub struct TFToken {
    pub owner: Addr,
    pub subdenom: String,
  }

  impl TFToken {
    pub fn new(owner: Addr, subdenom: String) -> Self {
      Self { owner, subdenom }
    }
  }

  impl super::TFToken for TFToken {
    fn owner(&self) -> Addr {
      self.owner.clone()
    }

    fn subdenom(&self) -> String {
      self.subdenom.clone()
    }

    fn denom(&self) -> String {
      format!("factory/{}/{}", self.owner, self.subdenom)
    }

    fn create(&self) -> Vec<CosmosMsg> {
      vec![MsgCreateDenom { sender: self.owner.to_string(), subdenom: self.subdenom.clone() }.into()]
    }

    fn mint(&self, amount: Uint128, recipient: String) -> Vec<CosmosMsg> {
      vec![MsgMint {
        sender: self.owner.to_string(),
        amount: Some(Coin {
          denom: self.denom(),
          amount: amount.to_string(),
        }),
        mint_to_address: recipient,
      }.into()]
    }
  }
}
