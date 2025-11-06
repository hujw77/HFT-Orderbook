//! Order implementation for the HFT orderbook

use crate::types::{OrderId, Price, Quantity, Side, Timestamp, ExchangeId, OrderStatus};
use std::fmt;

#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

/// Represents a single order in the orderbook
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Order {
    /// Unique identifier for this order
    pub id: OrderId,
    /// Buy or Sell
    pub side: Side,
    /// Order quantity (shares)
    pub quantity: Quantity,
    /// Remaining quantity (for partial fills)
    pub remaining_quantity: Quantity,
    /// Limit price
    pub price: Price,
    /// Time when order was created
    pub entry_time: Timestamp,
    /// Time when order was last updated
    pub event_time: Timestamp,
    /// Exchange identifier
    pub exchange_id: ExchangeId,
    /// Current status of the order
    pub status: OrderStatus,
    /// Index of next order in the doubly-linked list (None if tail)
    pub(crate) next_order_index: Option<usize>,
    /// Index of previous order in the doubly-linked list (None if head)
    pub(crate) prev_order_index: Option<usize>,
    /// Index of the parent limit level
    pub(crate) parent_limit_index: Option<usize>,
}

impl Order {
    /// Create a new order
    pub fn new(
        id: OrderId,
        side: Side,
        quantity: Quantity,
        price: Price,
        entry_time: Timestamp,
        exchange_id: ExchangeId,
    ) -> Self {
        Self {
            id,
            side,
            quantity,
            remaining_quantity: quantity,
            price,
            entry_time,
            event_time: entry_time,
            exchange_id,
            status: OrderStatus::Active,
            next_order_index: None,
            prev_order_index: None,
            parent_limit_index: None,
        }
    }

    /// Check if this is a buy order
    pub fn is_buy(&self) -> bool {
        self.side.is_buy()
    }

    /// Check if this is a sell order
    pub fn is_sell(&self) -> bool {
        self.side.is_sell()
    }

    /// Check if the order is completely filled
    pub fn is_filled(&self) -> bool {
        self.remaining_quantity == 0
    }

    /// Check if the order is partially filled
    pub fn is_partially_filled(&self) -> bool {
        self.remaining_quantity > 0 && self.remaining_quantity < self.quantity
    }

    /// Get the filled quantity
    pub fn filled_quantity(&self) -> Quantity {
        self.quantity - self.remaining_quantity
    }

    /// Calculate the total value of the order (price * quantity)
    pub fn value(&self) -> u128 {
        self.price as u128 * self.quantity as u128
    }

    /// Calculate the remaining value of the order (price * remaining_quantity)
    pub fn remaining_value(&self) -> u128 {
        self.price as u128 * self.remaining_quantity as u128
    }

    /// Fill the order by the specified quantity
    /// Returns the actual quantity filled (may be less than requested)
    pub fn fill(&mut self, quantity: Quantity, event_time: Timestamp) -> Quantity {
        let fill_quantity = quantity.min(self.remaining_quantity);
        self.remaining_quantity -= fill_quantity;
        self.event_time = event_time;
        
        if self.remaining_quantity == 0 {
            self.status = OrderStatus::Filled;
        } else if self.remaining_quantity < self.quantity {
            self.status = OrderStatus::PartiallyFilled;
        }
        
        fill_quantity
    }

    /// Cancel the order
    pub fn cancel(&mut self, event_time: Timestamp) {
        self.status = OrderStatus::Cancelled;
        self.event_time = event_time;
    }

    /// Update the order quantity (for order modifications)
    pub fn update_quantity(&mut self, new_quantity: Quantity, event_time: Timestamp) -> bool {
        let filled = self.filled_quantity();
        if new_quantity < filled {
            // Cannot reduce quantity below filled amount
            return false;
        }

        self.quantity = new_quantity;
        self.remaining_quantity = new_quantity - filled;
        self.event_time = event_time;
        
        if self.remaining_quantity == 0 {
            self.status = OrderStatus::Filled;
        } else if self.filled_quantity() > 0 {
            self.status = OrderStatus::PartiallyFilled;
        } else {
            self.status = OrderStatus::Active;
        }
        
        true
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Order[{}]: {} {} @ {} (remaining: {}, status: {})",
            self.id, self.side, self.quantity, self.price, self.remaining_quantity, self.status
        )
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Orders are compared by entry time for FIFO ordering within the same price level
        self.entry_time.cmp(&other.entry_time)
    }
}

impl Eq for Order {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
        assert_eq!(order.id, 1);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.quantity, 100);
        assert_eq!(order.remaining_quantity, 100);
        assert_eq!(order.price, 5000);
        assert_eq!(order.status, OrderStatus::Active);
        assert!(!order.is_filled());
        assert!(!order.is_partially_filled());
    }

    #[test]
    fn test_order_fill() {
        let mut order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
        
        // Partial fill
        let filled = order.fill(30, 1001);
        assert_eq!(filled, 30);
        assert_eq!(order.remaining_quantity, 70);
        assert_eq!(order.filled_quantity(), 30);
        assert!(order.is_partially_filled());
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        
        // Complete fill
        let filled = order.fill(70, 1002);
        assert_eq!(filled, 70);
        assert_eq!(order.remaining_quantity, 0);
        assert_eq!(order.filled_quantity(), 100);
        assert!(order.is_filled());
        assert_eq!(order.status, OrderStatus::Filled);
    }

    #[test]
    fn test_order_overfill() {
        let mut order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
        
        // Try to fill more than available
        let filled = order.fill(150, 1001);
        assert_eq!(filled, 100);
        assert_eq!(order.remaining_quantity, 0);
        assert!(order.is_filled());
    }
}
