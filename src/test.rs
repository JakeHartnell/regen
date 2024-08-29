use cosmwasm_std::{Addr, Coin, Uint128};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::contract::{execute, instantiate, query};
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, SellOrderMsg, SellOrderResponse, SellOrdersResponse,
};
use crate::state::FeeParams;

fn mock_app() -> App {
    App::default()
}

fn instantiate_contract(app: &mut App, sender: Addr) -> Addr {
    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let msg = InstantiateMsg {
        fee_params: FeeParams {
            buyer_percentage_fee: "0.01".to_string(),
            seller_percentage_fee: "0.01".to_string(),
        },
    };

    app.instantiate_contract(code_id, sender, &msg, &[], "Regen Marketplace", None)
        .unwrap()
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked("owner");
    instantiate_contract(&mut app, sender);
}

#[test]
fn test_sell_order() {
    let mut app = mock_app();
    let sender = Addr::unchecked("seller");
    let contract_addr = instantiate_contract(&mut app, sender.clone());

    let sell_msg = ExecuteMsg::Sell {
        orders: vec![SellOrderMsg {
            batch_denom: "eco.uC.001".to_string(),
            quantity: "100".to_string(),
            ask_price: Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(1000),
            },
            disable_auto_retire: false,
            expiration: None,
        }],
    };

    let res = app
        .execute_contract(sender.clone(), contract_addr.clone(), &sell_msg, &[])
        .unwrap();

    // Check if the sell order was created successfully
    assert!(res.events.len() > 0);

    // Query the sell order
    let query_msg = QueryMsg::SellOrder { sell_order_id: 1 };
    let res: SellOrderResponse = app
        .wrap()
        .query_wasm_smart(&contract_addr, &query_msg)
        .unwrap();

    assert_eq!(res.sell_order.seller, sender);
    assert_eq!(res.sell_order.quantity, "100");
    assert_eq!(res.sell_order.ask_amount, "1000");
}

#[test]
fn test_buy_direct() {
    let mut app = mock_app();
    let seller = Addr::unchecked("seller");
    let buyer = Addr::unchecked("buyer");
    let contract_addr = instantiate_contract(&mut app, seller.clone());

    // Create a sell order
    let sell_msg = ExecuteMsg::Sell {
        orders: vec![SellOrderMsg {
            batch_denom: "eco.uC.001".to_string(),
            quantity: "100".to_string(),
            ask_price: Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(1000),
            },
            disable_auto_retire: false,
            expiration: None,
        }],
    };

    app.execute_contract(seller.clone(), contract_addr.clone(), &sell_msg, &[])
        .unwrap();

    // Buy the credits
    let buy_msg = ExecuteMsg::BuyDirect {
        orders: vec![crate::msg::BuyOrderMsg {
            sell_order_id: 1,
            quantity: "50".to_string(),
            bid_price: Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(1000),
            },
            disable_auto_retire: false,
            retirement_jurisdiction: None,
            retirement_reason: None,
            max_fee_amount: Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(10),
            },
        }],
    };

    let res = app
        .execute_contract(buyer.clone(), contract_addr.clone(), &buy_msg, &[])
        .unwrap();

    // Check if the buy order was processed successfully
    assert!(res.events.len() > 0);

    // Query the updated sell order
    let query_msg = QueryMsg::SellOrder { sell_order_id: 1 };
    let res: SellOrderResponse = app
        .wrap()
        .query_wasm_smart(&contract_addr, &query_msg)
        .unwrap();

    assert_eq!(res.sell_order.quantity, "50");
}

#[test]
fn test_cancel_sell_order() {
    let mut app = mock_app();
    let seller = Addr::unchecked("seller");
    let contract_addr = instantiate_contract(&mut app, seller.clone());

    // Create a sell order
    let sell_msg = ExecuteMsg::Sell {
        orders: vec![SellOrderMsg {
            batch_denom: "eco.uC.001".to_string(),
            quantity: "100".to_string(),
            ask_price: Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(1000),
            },
            disable_auto_retire: false,
            expiration: None,
        }],
    };

    app.execute_contract(seller.clone(), contract_addr.clone(), &sell_msg, &[])
        .unwrap();

    // Cancel the sell order
    let cancel_msg = ExecuteMsg::CancelSellOrder { sell_order_id: 1 };
    let res = app
        .execute_contract(seller.clone(), contract_addr.clone(), &cancel_msg, &[])
        .unwrap();

    // Check if the sell order was cancelled successfully
    assert!(res.events.len() > 0);

    // Query all sell orders (should be empty now)
    let query_msg = QueryMsg::SellOrders {
        start_after: None,
        limit: None,
    };
    let res: SellOrdersResponse = app
        .wrap()
        .query_wasm_smart(&contract_addr, &query_msg)
        .unwrap();

    assert_eq!(res.sell_orders.len(), 0);
}
