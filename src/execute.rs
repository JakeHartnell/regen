use cosmwasm_std::{Coin, DepsMut, Env, MessageInfo, Response, Uint128};
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{BuyOrderMsg, SellOrderMsg, UpdateSellOrderMsg};
use crate::state::{
    AllowedDenom, FeeParams, SellOrder, ALLOWED_DENOMS, FEE_PARAMS, SELL_ORDERS, SELL_ORDER_SEQ,
};

pub fn sell(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    orders: Vec<SellOrderMsg>,
) -> Result<Response, ContractError> {
    let mut sell_order_ids = Vec::new();

    for order in orders {
        let id = SELL_ORDER_SEQ.update(deps.storage, |id| -> Result<_, ContractError> {
            Ok(id + 1)
        })?;

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

pub fn update_sell_orders(
    deps: DepsMut,
    _env: Env,
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

pub fn cancel_sell_order(
    deps: DepsMut,
    _env: Env,
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

pub fn buy_direct(
    deps: DepsMut,
    _env: Env,
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
    }

    Ok(Response::new().add_attribute("method", "buy_direct"))
}

pub fn add_allowed_denom(
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

pub fn remove_allowed_denom(
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

pub fn gov_set_fee_params(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fees: FeeParams,
) -> Result<Response, ContractError> {
    // In a real implementation, you'd check if the sender has the authority to set fees

    FEE_PARAMS.save(deps.storage, &fees)?;

    Ok(Response::new().add_attribute("method", "gov_set_fee_params"))
}

pub fn gov_send_from_fee_pool(
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

fn calculate_fee(price: &Coin, fee_percentage: &str) -> Result<Coin, ContractError> {
    let fee_percentage =
        Uint128::from_str(fee_percentage).map_err(|_| ContractError::InvalidInput {})?;
    let fee_amount = price
        .amount
        .multiply_ratio(fee_percentage, Uint128::new(10000)); // Assuming fee is in basis points
    Ok(Coin {
        denom: price.denom.clone(),
        amount: fee_amount,
    })
}
