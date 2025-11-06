//! Order implementation for the HFT orderbook

use crate::types::{OrderId, Price, Quantity, Side, Timestamp, ExchangeId};
use std::fmt;

#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

/// Represents a single order in the orderbook
///
/// **Important Design Decision: Index-based vs Pointer-based**
///
/// Unlike the C/Python implementations which use raw pointers (`Order *nextOrder`),
/// this Rust implementation uses indices (`Option<usize>`) to reference other orders
/// and limits. This is because:
///
/// 1. **Memory Safety**: Rust's ownership system doesn't allow circular references
///    with mutable access, which is required for doubly-linked lists with pointers.
///
/// 2. **Performance**: Index access is O(1) and just as fast as pointer dereferencing,
///    while providing better cache locality since data is stored in contiguous vectors.
///
/// 3. **Simplicity**: No need for lifetime annotations that would complicate the API.
///
/// 4. **Safety**: Index-based access is validated at access time, preventing
///    dangling pointer bugs that are common in C implementations.
///
/// The indices refer to positions in `OrderBook.orders` and `OrderBook.limits` vectors.
/// This pattern is known as "Slot Map" or "Arena Allocator" and is common in Rust
/// for similar data structures.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Order {
    /// Unique identifier for this order
    pub id: OrderId,
    /// Buy or Sell
    pub side: Side,
    /// Order quantity (shares)
    pub quantity: Quantity,
    /// Limit price
    pub price: Price,
    /// Time when order was created
    pub entry_time: Timestamp,
    /// Time when order was last updated
    pub event_time: Timestamp,
    /// Exchange identifier
    pub exchange_id: ExchangeId,
    /// Index of next order in the doubly-linked list (None if tail)
    /// 
    /// This is an index into `OrderBook.orders` vector, not a raw pointer.
    /// This allows safe circular references without violating Rust's borrow rules.
    pub(crate) next_order_index: Option<usize>,
    /// Index of previous order in the doubly-linked list (None if head)
    /// 
    /// This is an index into `OrderBook.orders` vector, not a raw pointer.
    pub(crate) prev_order_index: Option<usize>,
    /// Index of the parent limit level
    /// 
    /// This is an index into `OrderBook.limits` vector, not a raw pointer.
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
            price,
            entry_time,
            event_time: entry_time,
            exchange_id,
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
        self.quantity == 0
    }

    /// Calculate the total value of the order (price * quantity)
    pub fn value(&self) -> u128 {
        self.price as u128 * self.quantity as u128
    }

    /// Fill the order by the specified quantity
    /// Returns the actual quantity filled (may be less than requested)
    pub fn fill(&mut self, quantity: Quantity, event_time: Timestamp) -> Quantity {
        let fill_quantity = quantity.min(self.quantity);
        self.quantity -= fill_quantity;
        self.event_time = event_time;
        fill_quantity
    }

    /// Cancel the order
    pub fn cancel(&mut self, event_time: Timestamp) {
        self.event_time = event_time;
    }

    /// Update the order quantity (for order modifications)
    pub fn update_quantity(&mut self, new_quantity: Quantity, event_time: Timestamp) -> bool {
        if new_quantity == 0 {
            return false;
        }

        self.quantity = new_quantity;
        self.event_time = event_time;
        true
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Order[{}]: {} {} @ {}",
            self.id, self.side, self.quantity, self.price
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
        assert_eq!(order.price, 5000);
        assert!(!order.is_filled());
    }

    #[test]
    fn test_order_fill() {
        let mut order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
        
        // Partial fill
        let filled = order.fill(30, 1001);
        assert_eq!(filled, 30);
        assert_eq!(order.quantity, 70);
        assert!(!order.is_filled());
        
        // Complete fill
        let filled = order.fill(70, 1002);
        assert_eq!(filled, 70);
        assert_eq!(order.quantity, 0);
        assert!(order.is_filled());
    }

    #[test]
    fn test_order_overfill() {
        let mut order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
        
        // Try to fill more than available
        let filled = order.fill(150, 1001);
        assert_eq!(filled, 100);
        assert_eq!(order.quantity, 0);
        assert!(order.is_filled());
    }
}
