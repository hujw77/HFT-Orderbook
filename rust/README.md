# HFT OrderBook - Rust Implementation

A high-performance limit order book implementation in Rust, following the C/Python design pattern with separated concerns.

## Design Philosophy

This implementation follows the **pure data structure** approach, exactly like the original C and Python implementations:

- **OrderBook**: Pure data structure for managing orders and price levels
- **No matching logic**: Just data structure operations (add, remove, update, query)
- **Consistent with C/Python**: Same design pattern as the reference implementations

## Features

- **O(1) operations** for add, cancel, and update orders
- **O(log M)** for the first order at a new price level (where M is the number of price levels)
- **AVL tree** for efficient price level management
- **Memory efficient** with pre-allocated pools
- **Python-style API** with `process_order()` method
- **Pure data structure**: No matching logic, just order management

## Architecture

### Core Components

- **Order**: Individual buy/sell orders with price, quantity, and metadata
- **Limit**: Price levels containing linked lists of orders at the same price
- **OrderBook**: Pure data structure managing orders and limits using AVL trees
- **AVL Tree**: Self-balancing binary search tree for O(log n) price level operations

### Design Pattern

```
┌─────────────────┐
│   OrderBook     │
│  (Data Only)    │
├─────────────────┤
│ • add_order()   │
│ • remove_order()│
│ • update_order()│
│ • process_order()│
│ • best_bid()    │
│ • best_ask()    │
│ • get_levels()  │
└─────────────────┘
```

## Performance Characteristics

| Operation | Time Complexity | Description |
|-----------|----------------|-------------|
| Add Order | O(1) amortized | O(log M) for new price level |
| Remove Order | O(1) | Direct order removal |
| Update Order | O(1) | In-place quantity update |
| Best Bid/Ask | O(1) | Cached values |
| Market Data | O(1) | Real-time statistics |
| Query Levels | O(M) | Where M is number of price levels |

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

### Order Processing

```rust
use hft_orderbook::{OrderBook, Order, Side};

let mut book = OrderBook::new();

// Add orders
book.add_order(Order::new(1, Side::Sell, 100, 5000, 1000, 1))?;

// Update order (Python-style)
let updated = Order::new(1, Side::Sell, 150, 5000, 1001, 1);
book.process_order(updated)?; // Updates existing order

// Remove order (set quantity to 0)
let removed = Order::new(1, Side::Sell, 0, 5000, 1002, 1);
book.process_order(removed)?; // Removes order
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

- **Pure data structure**: OrderBook only manages data, no matching logic (exactly like C/Python)
- **Python-style API**: `process_order()` method for smart add/update/remove (same as Python)
- **Performance**: Same O(1) and O(log M) characteristics as original
- **Data structure only**: No matching engine, just pure order book operations

This design exactly matches the C and Python implementations - a pure data structure for managing orders and price levels, without any matching logic.
