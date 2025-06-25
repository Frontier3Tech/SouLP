use cosmwasm_schema::cw_serde;

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
