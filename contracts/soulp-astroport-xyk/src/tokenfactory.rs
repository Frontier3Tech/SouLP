#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
  #[prost(string, tag = "1")]
  pub denom: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub amount: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateDenom {
  #[prost(string, tag = "1")]
  pub sender: String,
  #[prost(string, tag = "2")]
  pub subdenom: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMint {
  #[prost(string, tag = "1")]
  pub sender: ::prost::alloc::string::String,
  #[prost(message, optional, tag = "2")]
  pub amount: Option<Coin>,
  #[prost(string, tag = "3")]
  pub denom: ::prost::alloc::string::String,
}
