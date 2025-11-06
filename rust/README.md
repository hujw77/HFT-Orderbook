# HFT OrderBook - Rust Implementation

A high-performance limit order book implementation in Rust, following the C/Python design pattern with separated concerns.

## Design Philosophy

This implementation follows the **pure data structure** approach, similar to the original C and Python implementations:

- **OrderBook**: Pure data structure for managing orders and price levels
- **MatchingEngine**: Separate component for order matching logic
- **Clean separation**: Data management vs. business logic

## Features

- **O(1) operations** for add, cancel, and update orders
- **O(log M)** for the first order at a new price level (where M is the number of price levels)
- **AVL tree** for efficient price level management
- **Memory efficient** with pre-allocated pools
- **Python-style API** with `process_order()` method
- **Flexible matching** with pluggable matching engines

## Architecture

### Core Components

- **Order**: Individual buy/sell orders with price, quantity, and metadata
- **Limit**: Price levels containing linked lists of orders at the same price
- **OrderBook**: Pure data structure managing orders and limits using AVL trees
- **MatchingEngine**: Separate component handling order matching logic
- **AVL Tree**: Self-balancing binary search tree for O(log n) price level operations

### Design Pattern

```
┌─────────────────┐    ┌─────────────────┐
│   OrderBook     │    │ MatchingEngine  │
│  (Data Only)    │    │ (Logic Only)    │
├─────────────────┤    ├─────────────────┤
│ • add_order()   │    │ • process_order │
│ • remove_order()│    │ • match_buy()   │
│ • update_order()│    │ • match_sell()  │
│ • best_bid()    │    │ • generate_trades│
│ • best_ask()    │    │                 │
└─────────────────┘    └─────────────────┘
```

## Performance Characteristics

| Operation | Time Complexity | Description |
|-----------|----------------|-------------|
| Add Order | O(1) amortized | O(log M) for new price level |
| Remove Order | O(1) | Direct order removal |
| Update Order | O(1) | In-place quantity update |
| Best Bid/Ask | O(1) | Cached values |
| Market Data | O(1) | Real-time statistics |
| Order Matching | O(k) | Where k is number of matched orders |

## Usage

### Pure OrderBook Operations

```rust
use hft_orderbook::{OrderBook, Order, Side};

// Create a new order book
let mut book = OrderBook::new();

// Add orders (no matching)
let buy_order = Order::new(1, Side::Buy, 100, 4950, 1000, 1);
book.add_order(buy_order)?;

// Query market data
println!("Best bid: {:?}", book.best_bid());
println!("Best ask: {:?}", book.best_ask());
println!("Spread: {:?}", book.spread());

// Python-style processing
let order = Order::new(2, Side::Sell, 50, 5050, 1001, 1);
book.process_order(order)?; // Add/update/remove based on quantity
```

### With Matching Engine

```rust
use hft_orderbook::{OrderBook, MatchingEngine, Order, Side};

let mut book = OrderBook::new();
let engine = MatchingEngine::new();

// Add resting orders
book.add_order(Order::new(1, Side::Sell, 100, 5000, 1000, 1))?;

// Process crossing order with matching
let crossing_order = Order::new(2, Side::Buy, 75, 5000, 1001, 1);
let trades = engine.process_order(&mut book, crossing_order)?;

println!("Generated {} trades", trades.len());
```

## API Reference

### OrderBook Methods

- `add_order(order)` - Add order to book (no matching)
- `remove_order(order_id)` - Remove order from book
- `update_order(order_id, new_quantity)` - Update order quantity
- `process_order(order)` - Python-style add/update/remove
- `best_bid()` - Get best bid price and quantity
- `best_ask()` - Get best ask price and quantity
- `get_levels(depth)` - Get price levels up to depth
- `volume_at_price(price)` - Get total volume at price
- `orders_at_price(price)` - Get order count at price

### MatchingEngine Methods

- `process_order(book, order)` - Process order with matching logic

## Examples

Run the examples to see the orderbook in action:

```bash
# Basic demo
cargo run --example clean_demo

# Simple trading simulation
cargo run --example simple_trading

# Pure orderbook demo
cargo run --example simple_orderbook_demo

# Main program
cargo run
```

## Testing

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench
```

## Design Consistency

This implementation maintains consistency with the original C and Python versions:

- **Pure data structure**: OrderBook only manages data, no matching logic
- **External matching**: Matching logic is handled by separate MatchingEngine
- **Python-style API**: `process_order()` method for smart order handling
- **Performance**: Same O(1) and O(log M) characteristics as original

This design makes the codebase more maintainable, testable, and flexible while preserving the high-performance characteristics of the original implementation.
