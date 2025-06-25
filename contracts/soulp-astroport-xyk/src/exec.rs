#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use r#impl::execute::{ExecuteContext, Token};
use r#impl::msg::EvacuateAsset;
use r#impl::tokenfactory::{self, TFToken};

use crate::contract::SUBDENOM;
use crate::state::{State, STATE};
use crate::{ContractError, ContractResult};
use crate::msg::ExecuteMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> ContractResult<Response> {
  let mut ctx = ExecuteContext { deps, env, info };
  match msg {
    ExecuteMsg::Deposit {} =>
      deposit(&mut ctx)?,
    ExecuteMsg::Evacuate { asset } =>
      evacuate(&mut ctx, asset)?,
    ExecuteMsg::ChangeEvacuateAddress { new_address } =>
      change_evacuate_address(&mut ctx, new_address)?,
  };
  Ok(Response::new())
}

fn deposit(ctx: &mut ExecuteContext) -> ContractResult<Response> {
  if ctx.info.funds.len() != 1 {
    return Err(ContractError::InvalidFunds("Expected exactly one asset".to_string()));
  }

  let state = STATE.load(ctx.deps.storage)?;

  let fund = &ctx.info.funds[0];
  if fund.denom != state.pool {
    return Err(ContractError::InvalidFunds("Invalid asset".to_string()));
  }

  let mint_amount = fund.amount * state.mint_ratio;

  let token = tokenfactory::osmosis::TFToken::new(ctx.env.contract.address.clone(), SUBDENOM.to_string());

  // NOTE: if this is a non-standard TokenFactory we may need to adjust the messages here
  Ok(Response::new()
    .add_messages(token.mint(mint_amount, ctx.info.sender.to_string()))
  )
}

fn evacuate(ctx: &mut ExecuteContext, asset: EvacuateAsset) -> ContractResult<Response> {
  let state = STATE.load(ctx.deps.storage)?;
  let evacuate_address = ctx.deps.api.addr_validate(&state.evacuate_address)?;
  Ok(Response::new()
    .add_messages(r#impl::execute::evacuate(
      ctx,
      Token::Native(state.pool),
      asset,
      evacuate_address
    )?)
    .add_attribute("action", "evacuate")
  )
}

fn change_evacuate_address(ctx: &mut ExecuteContext, new_address: String) -> ContractResult<Response> {
  let state = STATE.load(ctx.deps.storage)?;
  if state.evacuate_address != ctx.info.sender {
    return Err(ContractError::Unauthorized {});
  }
  STATE.save(ctx.deps.storage, &State {
    evacuate_address: new_address,
    ..state
  })?;
  Ok(Response::new())
}

#[cfg(test)]
mod test {
  use std::marker::PhantomData;

  use super::*;
  use r#impl::tokenfactory::osmosis::MsgMint;

  use cosmwasm_std::{coin, coins, BankMsg, CosmosMsg, DepsMut, Empty, OwnedDeps, SubMsg, WasmMsg, Decimal};
  use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockStorage};
  use cw20::Cw20ExecuteMsg;
  use cw721::Cw721ExecuteMsg;
  use prost::Message;
  use test_utils::mock_querier::MockQuerier;

  fn setup_test_state(deps: &mut DepsMut) {
    let state = State {
      pool: "pool_token".to_string(),
      evacuate_address: "evacuate_addr".to_string(),
      mint_ratio: Decimal::percent(100),
    };
    STATE.save(deps.storage, &state).unwrap();
  }

  #[test]
  fn test_evacuate_native_assets() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);
    setup_test_state(&mut deps.as_mut());

    // Mock contract balance with multiple native tokens
    deps.querier.update_balance(
      env.contract.address.clone(),
      vec![
        coin(100, "uatom"),
        coin(50, "uosmo"),
        coin(100, "pool_token"),
      ]
    );

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let result = evacuate(&mut ctx, EvacuateAsset::Native {}).unwrap();

    // Should have 2 messages (uatom and uosmo, but not pool_token)
    assert_eq!(result.messages.len(), 2);

    // Check that the messages are BankMsg::Send to evacuate_address
    for msg in result.messages {
      match msg {
        SubMsg { msg: CosmosMsg::Bank(BankMsg::Send { to_address, amount }), .. } => {
          assert_eq!(to_address, "evacuate_addr");
          assert_eq!(amount.len(), 1);
          let coin = &amount[0];
          assert!(coin.denom != "pool_token");
          assert!(coin.denom == "uatom" || coin.denom == "uosmo");
        }
        _ => panic!("Expected BankMsg::Send"),
      }
    }

    // Check attributes
    assert_eq!(result.attributes.len(), 1);
    assert_eq!(result.attributes[0].key, "action");
    assert_eq!(result.attributes[0].value, "evacuate");
  }

  #[test]
  fn test_evacuate_cw20_asset() {
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let cw20_contract = "cw20_contract_addr";
    let expected_balance = 1000u128;

    let mut deps = OwnedDeps {
      custom_query_type: PhantomData::<Empty>,
      querier: MockQuerier::new().with_cw20_balance(env.contract.address.as_str(), expected_balance),
      storage: MockStorage::default(),
      api: MockApi::default(),
    };
    setup_test_state(&mut deps.as_mut());

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let result = evacuate(&mut ctx, EvacuateAsset::Cw20 {
      contract: cw20_contract.to_string(),
    }).unwrap();

    // Should have 1 message
    assert_eq!(result.messages.len(), 1);

    // Check that the message is WasmMsg::Execute for CW20 transfer
    match &result.messages[0] {
      SubMsg { msg: CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, funds }), .. } => {
        assert_eq!(contract_addr, cw20_contract);
        assert_eq!(funds.len(), 0);

        // Verify the message is a CW20 transfer
        let transfer_msg: Cw20ExecuteMsg = cosmwasm_std::from_json(msg).unwrap();
        match transfer_msg {
          Cw20ExecuteMsg::Transfer { recipient, amount } => {
            assert_eq!(recipient, "evacuate_addr");
            assert_eq!(amount.u128(), expected_balance);
          }
          _ => panic!("Expected Transfer message"),
        }
      }
      _ => panic!("Expected WasmMsg::Execute"),
    }
  }

  #[test]
  fn test_evacuate_cw721_assets() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);
    setup_test_state(&mut deps.as_mut());

    let cw721_contract = "cw721_contract_addr";
    let token_ids = vec!["token1".to_string(), "token2".to_string(), "token3".to_string()];

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let result = evacuate(&mut ctx, EvacuateAsset::Cw721 {
      contract: cw721_contract.to_string(),
      token_ids: token_ids.clone(),
    }).unwrap();

    // Should have 3 messages (one for each token)
    assert_eq!(result.messages.len(), 3);

    // Check that each message is WasmMsg::Execute for CW721 transfer
    for (i, msg) in result.messages.iter().enumerate() {
      match msg {
        SubMsg { msg: CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, funds }), .. } => {
          assert_eq!(contract_addr, cw721_contract);
          assert_eq!(funds.len(), 0);

          // Verify the message is a CW721 transfer
          let transfer_msg: Cw721ExecuteMsg = cosmwasm_std::from_json(msg).unwrap();
          match transfer_msg {
            Cw721ExecuteMsg::TransferNft { recipient, token_id } => {
              assert_eq!(recipient, "evacuate_addr");
              assert_eq!(token_id, token_ids[i]);
            }
            _ => panic!("Expected TransferNft message"),
          }
        }
        _ => panic!("Expected WasmMsg::Execute"),
      }
    }
  }

  #[test]
  fn test_evacuate_cw20_invalid_contract() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);
    setup_test_state(&mut deps.as_mut());

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let result = evacuate(&mut ctx, EvacuateAsset::Cw20 {
      contract: "invalid_address".to_string(),
    });

    // Should return an error for invalid address
    assert!(result.is_err());
  }

  #[test]
  fn test_change_evacuate_address_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("evacuate_addr", &[]);
    setup_test_state(&mut deps.as_mut());

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let new_address = "new_evacuate_addr".to_string();

    let result = change_evacuate_address(&mut ctx, new_address.clone()).unwrap();

    // Should succeed
    assert_eq!(result.messages.len(), 0);
    assert_eq!(result.attributes.len(), 0);

    // Verify state was updated
    let updated_state = STATE.load(ctx.deps.storage).unwrap();
    assert_eq!(updated_state.evacuate_address, new_address);
    assert_eq!(updated_state.pool, "pool_token"); // Pool should remain unchanged
  }

  #[test]
  fn test_change_evacuate_address_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("unauthorized_sender", &[]);
    setup_test_state(&mut deps.as_mut());

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let new_address = "new_evacuate_addr".to_string();

    let result = change_evacuate_address(&mut ctx, new_address);

    // Should return Unauthorized error
    assert!(result.is_err());
    match result.unwrap_err() {
      ContractError::Unauthorized {} => (), // Expected
      _ => panic!("Expected Unauthorized error"),
    }

    // Verify state was not changed
    let state = STATE.load(ctx.deps.storage).unwrap();
    assert_eq!(state.evacuate_address, "evacuate_addr"); // Should remain unchanged
  }

  #[test]
  fn test_evacuate_native_no_assets() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);
    setup_test_state(&mut deps.as_mut());

    // No balance set up, so contract has no assets
    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let result = evacuate(&mut ctx, EvacuateAsset::Native {}).unwrap();

    // Should succeed but with no messages
    assert_eq!(result.messages.len(), 0);
    assert_eq!(result.attributes.len(), 1);
    assert_eq!(result.attributes[0].key, "action");
    assert_eq!(result.attributes[0].value, "evacuate");
  }

  #[test]
  fn test_evacuate_native_only_pool_token() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);
    setup_test_state(&mut deps.as_mut());

    // Only pool token balance (should be filtered out)
    deps.querier.update_balance(
      env.contract.address.clone(),
      coins(100, "pool_token"),
    );

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let result = evacuate(&mut ctx, EvacuateAsset::Native {}).unwrap();

    // Should succeed but with no messages (pool token filtered out)
    assert_eq!(result.messages.len(), 0);
    assert_eq!(result.attributes.len(), 1);
    assert_eq!(result.attributes[0].key, "action");
    assert_eq!(result.attributes[0].value, "evacuate");
  }

  #[test]
  fn test_deposit_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &coins(100, "pool_token"));
    setup_test_state(&mut deps.as_mut());

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env: env.clone(), info };
    let result = deposit(&mut ctx).unwrap();

    // Should have 1 message (MsgMint)
    assert_eq!(result.messages.len(), 1);
    assert_eq!(result.attributes.len(), 0);

    // Check that the message is a Stargate message for MsgMint
    match &result.messages[0] {
      SubMsg { msg: CosmosMsg::Stargate { type_url, value }, .. } => {
        assert_eq!(type_url, "/osmosis.tokenfactory.v1beta1.MsgMint");

        // Decode the MsgMint to verify its contents
        let msg_mint = MsgMint::decode(value.as_slice()).unwrap();
        assert_eq!(msg_mint.sender, env.contract.address.to_string());
        assert_eq!(msg_mint.mint_to_address, "sender");

        // Check the minted amount (should be 100 * 1.0 = 100 since mint_ratio is 100%)
        let amount = msg_mint.amount.unwrap();
        assert_eq!(amount.denom, format!("factory/{}/SouLP", env.contract.address));
        assert_eq!(amount.amount, "100");
      }
      _ => panic!("Expected Stargate message"),
    }
  }

  #[test]
  fn test_deposit_with_mint_ratio() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    // Set up state with a different mint ratio (50%)
    let state = State {
      pool: "pool_token".to_string(),
      evacuate_address: "evacuate_addr".to_string(),
      mint_ratio: Decimal::percent(50),
    };
    STATE.save(deps.as_mut().storage, &state).unwrap();

    let info = mock_info("sender", &coins(100, "pool_token"));
    let mut ctx = ExecuteContext { deps: deps.as_mut(), env: env.clone(), info };
    let result = deposit(&mut ctx).unwrap();

    // Should have 1 message (MsgMint)
    assert_eq!(result.messages.len(), 1);

    // Check that the message is a Stargate message for MsgMint
    match &result.messages[0] {
      SubMsg { msg: CosmosMsg::Stargate { type_url, value }, .. } => {
        assert_eq!(type_url, "/osmosis.tokenfactory.v1beta1.MsgMint");

        // Decode the MsgMint to verify its contents
        let msg_mint = MsgMint::decode(value.as_slice()).unwrap();
        assert_eq!(msg_mint.sender, env.contract.address.to_string());
        assert_eq!(msg_mint.mint_to_address, "sender");

        // Check the minted amount (should be 100 * 0.5 = 50 since mint_ratio is 50%)
        let amount = msg_mint.amount.unwrap();
        assert_eq!(amount.denom, format!("factory/{}/SouLP", env.contract.address));
        assert_eq!(amount.amount, "50");
      }
      _ => panic!("Expected Stargate message"),
    }
  }

  #[test]
  fn test_deposit_no_funds() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);
    setup_test_state(&mut deps.as_mut());

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let result = deposit(&mut ctx);

    // Should return error for no funds
    assert!(result.is_err());
    match result.unwrap_err() {
      ContractError::InvalidFunds(msg) => {
        assert_eq!(msg, "Expected exactly one asset");
      }
      _ => panic!("Expected InvalidFunds error"),
    }
  }

  #[test]
  fn test_deposit_multiple_funds() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &coins(100, "pool_token"));
    let mut funds = info.funds.clone();
    funds.push(coin(50, "uatom"));
    let info = mock_info("sender", &funds);
    setup_test_state(&mut deps.as_mut());

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let result = deposit(&mut ctx);

    // Should return error for multiple funds
    assert!(result.is_err());
    match result.unwrap_err() {
      ContractError::InvalidFunds(msg) => {
        assert_eq!(msg, "Expected exactly one asset");
      }
      _ => panic!("Expected InvalidFunds error"),
    }
  }

  #[test]
  fn test_deposit_wrong_asset() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &coins(100, "uatom")); // Wrong asset
    setup_test_state(&mut deps.as_mut());

    let mut ctx = ExecuteContext { deps: deps.as_mut(), env, info };
    let result = deposit(&mut ctx);

    // Should return error for wrong asset
    assert!(result.is_err());
    match result.unwrap_err() {
      ContractError::InvalidFunds(msg) => {
        assert_eq!(msg, "Invalid asset");
      }
      _ => panic!("Expected InvalidFunds error"),
    }
  }
}
