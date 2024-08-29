use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp};

use crate::state::{AllowedDenom, FeeParams, SellOrder};

#[cw_serde]
pub struct InstantiateMsg {
    pub fee_params: FeeParams,
}

#[cw_serde]
pub enum ExecuteMsg {
    Sell {
        orders: Vec<SellOrderMsg>,
    },
    UpdateSellOrders {
        updates: Vec<UpdateSellOrderMsg>,
    },
    CancelSellOrder {
        sell_order_id: u64,
    },
    BuyDirect {
        orders: Vec<BuyOrderMsg>,
    },
    AddAllowedDenom {
        bank_denom: String,
        display_denom: String,
        exponent: u32,
    },
    RemoveAllowedDenom {
        denom: String,
    },
    GovSetFeeParams {
        fees: FeeParams,
    },
    GovSendFromFeePool {
        recipient: String,
        coins: Vec<Coin>,
    },
}

#[cw_serde]
pub struct SellOrderMsg {
    pub batch_denom: String,
    pub quantity: String,
    pub ask_price: Coin,
    pub disable_auto_retire: bool,
    pub expiration: Option<Timestamp>,
}

#[cw_serde]
pub struct UpdateSellOrderMsg {
    pub sell_order_id: u64,
    pub new_quantity: Option<String>,
    pub new_ask_price: Option<Coin>,
    pub disable_auto_retire: Option<bool>,
    pub new_expiration: Option<Timestamp>,
}

#[cw_serde]
pub struct BuyOrderMsg {
    pub sell_order_id: u64,
    pub quantity: String,
    pub bid_price: Coin,
    pub disable_auto_retire: bool,
    pub retirement_jurisdiction: Option<String>,
    pub retirement_reason: Option<String>,
    pub max_fee_amount: Coin,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SellOrderResponse)]
    SellOrder { sell_order_id: u64 },
    #[returns(SellOrdersResponse)]
    SellOrders {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(SellOrdersResponse)]
    SellOrdersByBatch {
        batch_denom: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(SellOrdersResponse)]
    SellOrdersBySeller {
        seller: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(AllowedDenomsResponse)]
    AllowedDenoms {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct SellOrderResponse {
    pub sell_order: SellOrder,
}

#[cw_serde]
pub struct SellOrdersResponse {
    pub sell_orders: Vec<SellOrder>,
}

#[cw_serde]
pub struct AllowedDenomsResponse {
    pub allowed_denoms: Vec<AllowedDenom>,
}
