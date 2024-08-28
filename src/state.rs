use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct SellOrder {
    pub id: u64,
    pub seller: Addr,
    pub batch_key: u64,
    pub quantity: String,
    pub market_id: u64,
    pub ask_amount: String,
    pub disable_auto_retire: bool,
    pub expiration: Option<Timestamp>,
    pub maker: bool,
}

#[cw_serde]
pub struct AllowedDenom {
    pub bank_denom: String,
    pub display_denom: String,
    pub exponent: u32,
}

#[cw_serde]
pub struct Market {
    pub id: u64,
    pub credit_type_abbrev: String,
    pub bank_denom: String,
    pub precision_modifier: u32,
}

#[cw_serde]
pub struct FeeParams {
    pub buyer_percentage_fee: String,
    pub seller_percentage_fee: String,
}

pub const SELL_ORDER_SEQ: Item<u64> = Item::new("sell_order_seq");
pub const SELL_ORDERS: Map<u64, SellOrder> = Map::new("sell_orders");
pub const ALLOWED_DENOMS: Map<String, AllowedDenom> = Map::new("allowed_denoms");
pub const MARKETS: Map<u64, Market> = Map::new("markets");
pub const FEE_PARAMS: Item<FeeParams> = Item::new("fee_params");
