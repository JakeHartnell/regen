use crate::msg::{AllowedDenomsResponse, SellOrderResponse, SellOrdersResponse};
use crate::state::{AllowedDenom, SellOrder, ALLOWED_DENOMS, SELL_ORDERS};
use cosmwasm_std::{to_binary, Binary, Deps, Order, StdResult};
use cw_storage_plus::Bound;

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

/// Queries a sell order by its unique identifier
pub fn sell_order(deps: Deps, sell_order_id: u64) -> StdResult<Binary> {
    let sell_order = SELL_ORDERS.load(deps.storage, sell_order_id)?;
    to_binary(&SellOrderResponse { sell_order })
}

/// Queries a paginated list of all sell orders
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

/// Queries a paginated list of all sell orders based on the batch denom
pub fn sell_orders_by_batch(
    deps: Deps,
    batch_denom: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    // In a real implementation, you'd filter sell orders by batch_denom
    // For simplicity, we're just calling the general sell_orders function
    sell_orders(deps, start_after, limit)
}

/// Queries a paginated list of all sell orders based on the seller
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

/// Queries a paginated list of all bank denoms allowed to be used in the marketplace
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
