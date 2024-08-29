use cosmwasm_std::{to_binary, Binary, Deps, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

use crate::msg::{AllowedDenomsResponse, SellOrderResponse, SellOrdersResponse};
use crate::state::{AllowedDenom, SellOrder, ALLOWED_DENOMS, SELL_ORDERS};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub fn sell_order(deps: Deps, sell_order_id: u64) -> StdResult<Binary> {
    let sell_order = SELL_ORDERS.load(deps.storage, sell_order_id)?;
    to_binary(&SellOrderResponse { sell_order })
}

pub fn sell_orders(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let sell_orders: StdResult<Vec<SellOrder>> = SELL_ORDERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, sell_order)| sell_order))
        .collect();

    to_binary(&SellOrdersResponse {
        sell_orders: sell_orders?,
    })
}

pub fn sell_orders_by_batch(
    deps: Deps,
    batch_denom: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let sell_orders: StdResult<Vec<SellOrder>> = SELL_ORDERS
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|r| match r {
            Ok((_, sell_order)) => sell_order.batch_key.to_string() == batch_denom,
            Err(_) => true,
        })
        .take(limit)
        .map(|item| item.map(|(_, sell_order)| sell_order))
        .collect();

    to_binary(&SellOrdersResponse {
        sell_orders: sell_orders?,
    })
}

pub fn sell_orders_by_seller(
    deps: Deps,
    seller: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Binary> {
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

    to_binary(&SellOrdersResponse {
        sell_orders: sell_orders?,
    })
}

pub fn allowed_denoms(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let allowed_denoms: StdResult<Vec<AllowedDenom>> = ALLOWED_DENOMS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, allowed_denom)| allowed_denom))
        .collect();

    to_binary(&AllowedDenomsResponse {
        allowed_denoms: allowed_denoms?,
    })
}
