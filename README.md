# Regen Network Marketplace Smart Contract

This CosmWasm smart contract implements a marketplace for eco-credits, mirroring the functionality of the Regen Network Cosmos SDK marketplace module. It allows users to create and manage sell orders for eco-credits, as well as purchase credits directly.

## Features

- Create sell orders for eco-credits
- Update existing sell orders
- Cancel sell orders
- Buy credits directly from sell orders
- Manage allowed denominations for trading
- Set and update marketplace fee parameters
- Query sell orders and allowed denominations

## Contract Structure

The contract is organized into several Rust files:

- `src/contract.rs`: Main entry point for the contract, handling instantiation, execution, and queries.
- `src/execute.rs`: Contains the implementation of all execute functions.
- `src/query.rs`: Contains the implementation of all query functions.
- `src/state.rs`: Defines the contract's state and storage.
- `src/msg.rs`: Defines the message types for contract interaction.
- `src/error.rs`: Defines custom error types for the contract.

## Usage

### Instantiation

To instantiate the contract, provide the initial fee parameters:

```rust
InstantiateMsg {
    fee_params: FeeParams {
        buyer_percentage_fee: "0.01".to_string(),
        seller_percentage_fee: "0.01".to_string(),
    },
}
```

### Execute Messages

The contract supports the following execute messages:

1. `Sell`: Create new sell orders for eco-credits.
2. `UpdateSellOrders`: Update existing sell orders.
3. `CancelSellOrder`: Cancel a specific sell order.
4. `BuyDirect`: Buy credits directly from specified sell orders.
5. `AddAllowedDenom`: Add a new allowed denomination for trading.
6. `RemoveAllowedDenom`: Remove an allowed denomination.
7. `GovSetFeeParams`: Set new fee parameters (governance function).
8. `GovSendFromFeePool`: Send coins from the fee pool (governance function).

### Query Messages

The contract supports the following query messages:

1. `SellOrder`: Query a specific sell order by ID.
2. `SellOrders`: Query a list of all sell orders.
3. `SellOrdersByBatch`: Query sell orders for a specific batch.
4. `SellOrdersBySeller`: Query sell orders for a specific seller.
5. `AllowedDenoms`: Query the list of allowed denominations.

## Development

To set up the development environment:

1. Install Rust and Cargo.
2. Install [cargo-generate](https://github.com/cargo-generate/cargo-generate) and [cosmwasm-check](https://github.com/CosmWasm/cosmwasm-check).
3. Clone the repository and navigate to the project directory.
4. Run `cargo build` to compile the contract.
5. Run `cargo test` to execute the test suite.

## Deployment

To deploy the contract:

1. Compile the contract: `cargo wasm`
2. Optimize the wasm binary: `docker run --rm -v "$(pwd)":/code \
--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
cosmwasm/rust-optimizer:0.12.6`
3. Deploy the optimized wasm file to your chosen CosmWasm-enabled blockchain.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
