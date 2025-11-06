//! Core types used throughout the orderbook implementation

use std::fmt;

#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

/// Unique identifier for orders
pub type OrderId = u64;

/// Price type - using u64 to represent price in smallest units (e.g., cents)
/// This avoids floating point precision issues in financial calculations
pub type Price = u64;

/// Quantity type - using u64 for order sizes
pub type Quantity = u64;

/// Timestamp type for order entry and event times
pub type Timestamp = u64;

/// Exchange identifier
pub type ExchangeId = u32;

/// Order side (Buy or Sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum Side {
    /// Buy order (bid)
    Buy,
    /// Sell order (ask)
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "Buy"),
            Side::Sell => write!(f, "Sell"),
        }
    }
}

impl Side {
    /// Returns true if this is a buy order
    pub fn is_buy(&self) -> bool {
        matches!(self, Side::Buy)
    }

    /// Returns true if this is a sell order
    pub fn is_sell(&self) -> bool {
        matches!(self, Side::Sell)
    }

    /// Returns the opposite side
    pub fn opposite(&self) -> Side {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

/// Trade information when orders are matched
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Trade {
    /// ID of the aggressive (incoming) order
    pub aggressor_order_id: OrderId,
    /// ID of the passive (resting) order
    pub passive_order_id: OrderId,
    /// Price at which the trade occurred
    pub price: Price,
    /// Quantity traded
    pub quantity: Quantity,
    /// Timestamp of the trade
    pub timestamp: Timestamp,
    /// Side of the aggressive order
    pub aggressor_side: Side,
}

impl Trade {
    /// Create a new trade
    pub fn new(
        aggressor_order_id: OrderId,
        passive_order_id: OrderId,
        price: Price,
        quantity: Quantity,
        timestamp: Timestamp,
        aggressor_side: Side,
    ) -> Self {
        Self {
            aggressor_order_id,
            passive_order_id,
            price,
            quantity,
            timestamp,
            aggressor_side,
        }
    }

    /// Calculate the trade value (price * quantity)
    pub fn value(&self) -> u128 {
        self.price as u128 * self.quantity as u128
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Trade: {} {} @ {} (aggressor: {}, passive: {}, time: {})",
            self.quantity, self.aggressor_side, self.price,
            self.aggressor_order_id, self.passive_order_id, self.timestamp
        )
    }
}
