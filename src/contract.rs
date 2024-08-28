use crate::error::ContractError;
use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query;
use crate::state::{FEE_PARAMS, SELL_ORDER_SEQ};
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

/// Initializes the contract with the given fee parameters
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    FEE_PARAMS.save(deps.storage, &msg.fee_params)?;
    SELL_ORDER_SEQ.save(deps.storage, &0u64)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

/// Handles all execute messages
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Sell { orders } => execute::sell(deps, env, info, orders),
        ExecuteMsg::UpdateSellOrders { updates } => {
            execute::update_sell_orders(deps, env, info, updates)
        }
        ExecuteMsg::CancelSellOrder { sell_order_id } => {
            execute::cancel_sell_order(deps, env, info, sell_order_id)
        }
        ExecuteMsg::BuyDirect { orders } => execute::buy_direct(deps, env, info, orders),
        ExecuteMsg::AddAllowedDenom {
            bank_denom,
            display_denom,
            exponent,
        } => execute::add_allowed_denom(deps, env, info, bank_denom, display_denom, exponent),
        ExecuteMsg::RemoveAllowedDenom { denom } => {
            execute::remove_allowed_denom(deps, env, info, denom)
        }
        ExecuteMsg::GovSetFeeParams { fees } => execute::gov_set_fee_params(deps, env, info, fees),
        ExecuteMsg::GovSendFromFeePool { recipient, coins } => {
            execute::gov_send_from_fee_pool(deps, env, info, recipient, coins)
        }
    }
}

/// Handles all query messages
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SellOrder { sell_order_id } => query::sell_order(deps, sell_order_id),
        QueryMsg::SellOrders { start_after, limit } => query::sell_orders(deps, start_after, limit),
        QueryMsg::SellOrdersByBatch {
            batch_denom,
            start_after,
            limit,
        } => query::sell_orders_by_batch(deps, batch_denom, start_after, limit),
        QueryMsg::SellOrdersBySeller {
            seller,
            start_after,
            limit,
        } => query::sell_orders_by_seller(deps, seller, start_after, limit),
        QueryMsg::AllowedDenoms { start_after, limit } => {
            query::allowed_denoms(deps, start_after, limit)
        }
    }
}
