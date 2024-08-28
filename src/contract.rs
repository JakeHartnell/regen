use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Storage, Uint128,
};
use cw_storage_plus::Bound;
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{
    AllowedDenomsResponse, BuyOrderMsg, ExecuteMsg, InstantiateMsg, QueryMsg, SellOrderMsg,
    SellOrderResponse, SellOrdersResponse, UpdateSellOrderMsg,
};
use crate::state::{
    AllowedDenom, FeeParams, Market, SellOrder, ALLOWED_DENOMS, FEE_PARAMS, MARKETS, SELL_ORDERS,
    SELL_ORDER_SEQ,
};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

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

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Sell { orders } => execute_sell(deps, env, info, orders),
        ExecuteMsg::UpdateSellOrders { updates } => {
            execute_update_sell_orders(deps, env, info, updates)
        }
        ExecuteMsg::CancelSellOrder { sell_order_id } => {
            execute_cancel_sell_order(deps, env, info, sell_order_id)
        }
        ExecuteMsg::BuyDirect { orders } => execute_buy_direct(deps, env, info, orders),
        ExecuteMsg::AddAllowedDenom {
            bank_denom,
            display_denom,
            exponent,
        } => execute_add_allowed_denom(deps, env, info, bank_denom, display_denom, exponent),
        ExecuteMsg::RemoveAllowedDenom { denom } => {
            execute_remove_allowed_denom(deps, env, info, denom)
        }
        ExecuteMsg::GovSetFeeParams { fees } => execute_gov_set_fee_params(deps, env, info, fees),
        ExecuteMsg::GovSendFromFeePool { recipient, coins } => {
            execute_gov_send_from_fee_pool(deps, env, info, recipient, coins)
        }
    }
}

pub fn execute_sell(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    orders: Vec<SellOrderMsg>,
) -> Result<Response, ContractError> {
    let mut sell_order_ids = Vec::new();

    for order in orders {
        let id = SELL_ORDER_SEQ.update(deps.storage, |id| -> StdResult<_> { Ok(id + 1) })?;

        let sell_order = SellOrder {
            id,
            seller: info.sender.clone(),
            batch_key: 0, // This should be set properly based on the batch_denom
            quantity: order.quantity,
            market_id: 0, // This should be set properly based on the ask_price denom
            ask_amount: order.ask_price.amount.to_string(),
            disable_auto_retire: order.disable_auto_retire,
            expiration: order.expiration,
            maker: true,
        };

        SELL_ORDERS.save(deps.storage, id, &sell_order)?;
        sell_order_ids.push(id);
    }

    Ok(Response::new()
        .add_attribute("method", "sell")
        .add_attribute(
            "sell_order_ids",
            sell_order_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(","),
        ))
}

pub fn execute_update_sell_orders(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    updates: Vec<UpdateSellOrderMsg>,
) -> Result<Response, ContractError> {
    for update in updates {
        let mut sell_order = SELL_ORDERS.load(deps.storage, update.sell_order_id)?;

        if sell_order.seller != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(new_quantity) = update.new_quantity {
            sell_order.quantity = new_quantity;
        }

        if let Some(new_ask_price) = update.new_ask_price {
            sell_order.ask_amount = new_ask_price.amount.to_string();
            // Update market_id if necessary
        }

        if let Some(disable_auto_retire) = update.disable_auto_retire {
            sell_order.disable_auto_retire = disable_auto_retire;
        }

        if let Some(new_expiration) = update.new_expiration {
            sell_order.expiration = Some(new_expiration);
        }

        SELL_ORDERS.save(deps.storage, update.sell_order_id, &sell_order)?;
    }

    Ok(Response::new().add_attribute("method", "update_sell_orders"))
}

pub fn execute_cancel_sell_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sell_order_id: u64,
) -> Result<Response, ContractError> {
    let sell_order = SELL_ORDERS.load(deps.storage, sell_order_id)?;

    if sell_order.seller != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    SELL_ORDERS.remove(deps.storage, sell_order_id);

    Ok(Response::new()
        .add_attribute("method", "cancel_sell_order")
        .add_attribute("sell_order_id", sell_order_id.to_string()))
}

pub fn execute_buy_direct(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    orders: Vec<BuyOrderMsg>,
) -> Result<Response, ContractError> {
    let fee_params = FEE_PARAMS.load(deps.storage)?;

    for order in orders {
        let mut sell_order = SELL_ORDERS.load(deps.storage, order.sell_order_id)?;

        // Check if the bid price is sufficient
        if order.bid_price.amount < Uint128::from_str(&sell_order.ask_amount)? {
            return Err(ContractError::InsufficientBidPrice);
        }

        // Calculate fees
        let buyer_fee = calculate_fee(&order.bid_price, &fee_params.buyer_percentage_fee)?;
        let seller_fee = calculate_fee(&order.bid_price, &fee_params.seller_percentage_fee)?;

        // Check if the max fee amount is sufficient
        if buyer_fee.amount > order.max_fee_amount.amount {
            return Err(ContractError::MaxFeeExceeded);
        }

        // Process the trade
        // This is a simplified version. In a real implementation, you'd need to handle
        // partial fills, update balances, and possibly interact with other modules.

        // Update sell order quantity
        let order_quantity = Uint128::from_str(&order.quantity)?;
        let sell_order_quantity = Uint128::from_str(&sell_order.quantity)?;

        if order_quantity > sell_order_quantity {
            return Err(ContractError::InsufficientSellOrderQuantity);
        }

        sell_order.quantity = (sell_order_quantity - order_quantity).to_string();

        if sell_order.quantity == "0" {
            SELL_ORDERS.remove(deps.storage, order.sell_order_id);
        } else {
            SELL_ORDERS.save(deps.storage, order.sell_order_id, &sell_order)?;
        }

        // In a real implementation, you'd transfer tokens here
    }

    Ok(Response::new().add_attribute("method", "buy_direct"))
}

pub fn execute_add_allowed_denom(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    bank_denom: String,
    display_denom: String,
    exponent: u32,
) -> Result<Response, ContractError> {
    // In a real implementation, you'd check if the sender has the authority to add denoms

    let allowed_denom = AllowedDenom {
        bank_denom: bank_denom.clone(),
        display_denom,
        exponent,
    };

    ALLOWED_DENOMS.save(deps.storage, bank_denom.clone(), &allowed_denom)?;

    Ok(Response::new()
        .add_attribute("method", "add_allowed_denom")
        .add_attribute("bank_denom", bank_denom))
}

pub fn execute_remove_allowed_denom(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: String,
) -> Result<Response, ContractError> {
    // In a real implementation, you'd check if the sender has the authority to remove denoms

    ALLOWED_DENOMS.remove(deps.storage, denom.clone());

    Ok(Response::new()
        .add_attribute("method", "remove_allowed_denom")
        .add_attribute("denom", denom))
}

pub fn execute_gov_set_fee_params(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fees: FeeParams,
) -> Result<Response, ContractError> {
    // In a real implementation, you'd check if the sender has the authority to set fees

    FEE_PARAMS.save(deps.storage, &fees)?;

    Ok(Response::new().add_attribute("method", "gov_set_fee_params"))
}

pub fn execute_gov_send_from_fee_pool(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    coins: Vec<Coin>,
) -> Result<Response, ContractError> {
    // In a real implementation, you'd check if the sender has the authority to send from the fee pool
    // You'd also need to implement the actual transfer of coins from the fee pool

    Ok(Response::new()
        .add_attribute("method", "gov_send_from_fee_pool")
        .add_attribute("recipient", recipient))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SellOrder { sell_order_id } => to_binary(&query_sell_order(deps, sell_order_id)?),
        QueryMsg::SellOrders { start_after, limit } => {
            to_binary(&query_sell_orders(deps, start_after, limit)?)
        }
        QueryMsg::SellOrdersByBatch {
            batch_denom,
            start_after,
            limit,
        } => to_binary(&query_sell_orders_by_batch(
            deps,
            batch_denom,
            start_after,
            limit,
        )?),
        QueryMsg::SellOrdersBySeller {
            seller,
            start_after,
            limit,
        } => to_binary(&query_sell_orders_by_seller(
            deps,
            seller,
            start_after,
            limit,
        )?),
        QueryMsg::AllowedDenoms { start_after, limit } => {
            to_binary(&query_allowed_denoms(deps, start_after, limit)?)
        }
    }
}

fn query_sell_order(deps: Deps, sell_order_id: u64) -> StdResult<SellOrderResponse> {
    let sell_order = SELL_ORDERS.load(deps.storage, sell_order_id)?;
    Ok(SellOrderResponse { sell_order })
}

fn query_sell_orders(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<SellOrdersResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let sell_orders: StdResult<Vec<SellOrder>> = SELL_ORDERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, sell_order)| sell_order))
        .collect();

    Ok(SellOrdersResponse {
        sell_orders: sell_orders?,
    })
}

fn query_sell_orders_by_batch(
    deps: Deps,
    batch_denom: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<SellOrdersResponse> {
    // In a real implementation, you'd filter sell orders by batch_denom
    // For simplicity, we're just calling the general query_sell_orders function
    query_sell_orders(deps, start_after, limit)
}

fn query_sell_orders_by_seller(
    deps: Deps,
    seller: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<SellOrdersResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let seller_addr = deps.api.addr_validate(&seller)?;

    let sell_orders: StdResult<Vec<SellOrder>> = SELL_ORDERS
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|r| match r {
            Ok((_, sell_order)) => sell_order.seller == seller_addr,
            Err(_) => true,
        })
        .take(limit)
        .map(|item| item.map(|(_, sell_order)| sell_order))
        .collect();

    Ok(SellOrdersResponse {
        sell_orders: sell_orders?,
    })
}

fn query_allowed_denoms(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllowedDenomsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let allowed_denoms: StdResult<Vec<AllowedDenom>> = ALLOWED_DENOMS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, allowed_denom)| allowed_denom))
        .collect();

    Ok(AllowedDenomsResponse {
        allowed_denoms: allowed_denoms?,
    })
}

fn calculate_fee(price: &Coin, fee_percentage: &str) -> StdResult<Coin> {
    let fee_percentage = Uint128::from_str(fee_percentage)?;
    let fee_amount = price
        .amount
        .multiply_ratio(fee_percentage, Uint128::new(100));
    Ok(Coin {
        denom: price.denom.clone(),
        amount: fee_amount,
    })
}
