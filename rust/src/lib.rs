//! # HFT Orderbook
//!
//! A high-frequency trading limit order book implementation in Rust.
//!
//! This implementation is based on WK Selph's design and provides O(1) operations
//! for add, cancel, and execute, with O(log M) for the first order at a new price level.
//!
//! ## Core Components
//!
//! - `Order`: Individual order with price, quantity, and metadata
//! - `Limit`: Price level containing orders at the same price (AVL tree node)
//! - `OrderBook`: Main order book managing buy and sell trees
//!
//! ## Performance Characteristics
//!
//! - Add Order: O(log M) for first order at price level, O(1) for subsequent
//! - Cancel Order: O(1)
//! - Execute Order: O(1)
//! - Get Best Bid/Ask: O(1)
//! - Get Volume at Limit: O(1)
//!
//! Where M is the number of price levels (typically << N orders).

pub mod order;
pub mod limit;
pub mod orderbook;
pub mod avl_tree;
pub mod types;

pub use order::Order;
pub use limit::Limit;
pub use orderbook::OrderBook;
pub use types::{OrderId, Price, Quantity, Side, Timestamp, Trade};

#[cfg(test)]
mod tests;

#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

/// Result type for orderbook operations
pub type Result<T> = std::result::Result<T, OrderBookError>;

/// Errors that can occur during orderbook operations
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum OrderBookError {
    /// Order with given ID already exists
    OrderAlreadyExists(OrderId),
    /// Order with given ID not found
    OrderNotFound(OrderId),
    /// Invalid price (must be positive)
    InvalidPrice(Price),
    /// Invalid quantity (must be positive)
    InvalidQuantity(Quantity),
    /// Limit level not found
    LimitNotFound(Price),
    /// Internal tree structure error
    TreeError(String),
}

impl std::fmt::Display for OrderBookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderBookError::OrderAlreadyExists(id) => write!(f, "Order {} already exists", id),
            OrderBookError::OrderNotFound(id) => write!(f, "Order {} not found", id),
            OrderBookError::InvalidPrice(price) => write!(f, "Invalid price: {}", price),
            OrderBookError::InvalidQuantity(qty) => write!(f, "Invalid quantity: {}", qty),
            OrderBookError::LimitNotFound(price) => write!(f, "Limit at price {} not found", price),
            OrderBookError::TreeError(msg) => write!(f, "Tree error: {}", msg),
        }
    }
}

impl std::error::Error for OrderBookError {}
