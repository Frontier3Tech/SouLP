use cosmwasm_std::{to_json_binary, Querier, QuerierResult, QueryRequest, WasmQuery};
use cw20::{BalanceResponse, Cw20QueryMsg};

/// Custom querier to handle CW20 queries
pub struct MockQuerier {
  cw20_balances: std::collections::HashMap<String, u128>,
}

impl MockQuerier {
  pub fn new() -> Self {
    Self {
      cw20_balances: std::collections::HashMap::new(),
    }
  }

  pub fn with_cw20_balance(mut self, contract: &str, balance: u128) -> Self {
    self.cw20_balances.insert(contract.to_string(), balance);
    self
  }
}

impl Querier for MockQuerier {
  fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
    let request: QueryRequest<cosmwasm_std::Empty> = match cosmwasm_std::from_json(bin_request) {
      Ok(v) => v,
      Err(e) => {
        return Err(cosmwasm_std::SystemError::InvalidRequest {
          error: format!("Parsing query request: {}", e),
          request: bin_request.into(),
        }).into();
      }
    };

    match request {
      QueryRequest::Wasm(WasmQuery::Smart { msg, .. }) => {
        // Handle CW20 balance queries
        if let Ok(balance_query) = cosmwasm_std::from_json::<Cw20QueryMsg>(&msg) {
          match balance_query {
            Cw20QueryMsg::Balance { address } => {
              println!("balances: {:?}", self.cw20_balances);
              if let Some(balance) = self.cw20_balances.get(&address) {
                let response = BalanceResponse {
                  balance: cosmwasm_std::Uint128::from(*balance),
                };
                return cosmwasm_std::SystemResult::Ok(
                  cosmwasm_std::ContractResult::Ok(to_json_binary(&response).unwrap())
                );
              }
            }
            _ => {
              println!("Unknown Cw20QueryMsg: {:?}", balance_query);
            }
          }
        }
      }
      _ => {
        println!("Unknown Request: {:?}", request);
      }
    }

    Err(cosmwasm_std::SystemError::InvalidRequest {
      error: "MockQuerier: Unknown query".to_string(),
      request: bin_request.into(),
    }).into()
  }
}
