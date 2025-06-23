use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
  pub pool: String,
}

#[cw_serde]
pub enum ExecuteMsg {
  /// Permanently lock the provided liquidity & mint a SouLP token.
  Deposit {},
  /// Evacuate assets sent on accident (including LP rewards) to this contract to the configured evacuation address.
  Evacuate {
    asset: EvacuateAsset,
  },
  /// Change the address to evacuate assets to. Can only be called by the current evacuation address.
  ChangeEvacuateAddress {
    new_address: String,
  },
}

#[cw_serde]
pub enum EvacuateAsset {
  /// Evacuate all native assets (except the pool token)
  Native {},
  /// Evacuate given cw20 asset
  Cw20 {
    contract: String,
  },
  /// Evacuate given cw721 NFTs
  Cw721 {
    contract: String,
    token_ids: Vec<String>,
  },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
