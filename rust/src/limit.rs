//! Limit (price level) implementation for the HFT orderbook

use crate::avl_tree::AvlNode;
use crate::types::{Price, Quantity, Side};
use std::fmt;

#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

/// Represents a price level in the orderbook
/// 
/// Each limit contains all orders at the same price level, organized as a doubly-linked list.
/// Limits are organized in an AVL tree structure for efficient price-based operations.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Limit {
    /// The price for this limit level
    pub price: Price,
    /// Total quantity at this price level
    pub size: Quantity,
    /// Total volume (price * size) at this price level
    pub total_volume: u128,
    /// Number of orders at this price level
    pub order_count: usize,
    /// Which side this limit belongs to (buy or sell tree)
    pub(crate) side: Side,
    /// AVL tree node information
    pub(crate) avl_node: AvlNode,
    /// Index of the first order in the doubly-linked list (None if empty)
    pub(crate) head_order_index: Option<usize>,
    /// Index of the last order in the doubly-linked list (None if empty)
    pub(crate) tail_order_index: Option<usize>,
}

impl Limit {
    /// Create a new limit at the specified price
    pub fn new(price: Price, side: Side) -> Self {
        Self {
            price,
            size: 0,
            total_volume: 0,
            order_count: 0,
            side,
            avl_node: AvlNode::new(),
            head_order_index: None,
            tail_order_index: None,
        }
    }

    /// Check if this limit has no orders
    pub fn is_empty(&self) -> bool {
        self.order_count == 0
    }

    /// Get the average price (should be the same as price for a limit)
    pub fn average_price(&self) -> Price {
        self.price
    }

    /// Add an order to this limit level
    /// This updates the statistics but doesn't manage the linked list structure
    pub fn add_order_stats(&mut self, quantity: Quantity) {
        self.size += quantity;
        self.total_volume += self.price as u128 * quantity as u128;
        self.order_count += 1;
    }

    /// Remove an order from this limit level
    /// This updates the statistics but doesn't manage the linked list structure
    pub fn remove_order_stats(&mut self, quantity: Quantity) {
        debug_assert!(self.size >= quantity, "Cannot remove more quantity than available");
        debug_assert!(self.order_count > 0, "Cannot remove order from empty limit");
        
        self.size -= quantity;
        self.total_volume -= self.price as u128 * quantity as u128;
        self.order_count -= 1;
    }

    /// Update statistics when an order quantity changes
    pub fn update_order_stats(&mut self, old_quantity: Quantity, new_quantity: Quantity) {
        if new_quantity > old_quantity {
            let diff = new_quantity - old_quantity;
            self.size += diff;
            self.total_volume += self.price as u128 * diff as u128;
        } else if old_quantity > new_quantity {
            let diff = old_quantity - new_quantity;
            self.size -= diff;
            self.total_volume -= self.price as u128 * diff as u128;
        }
    }

    /// Get the total value at this limit level
    pub fn total_value(&self) -> u128 {
        self.total_volume
    }

    /// Check if this limit is a leaf node in the AVL tree
    pub fn is_leaf(&self) -> bool {
        self.avl_node.is_leaf()
    }

    /// Check if this limit has only a left child
    pub fn has_only_left_child(&self) -> bool {
        self.avl_node.has_only_left_child()
    }

    /// Check if this limit has only a right child
    pub fn has_only_right_child(&self) -> bool {
        self.avl_node.has_only_right_child()
    }

    /// Check if this limit has both children
    pub fn has_both_children(&self) -> bool {
        self.avl_node.has_both_children()
    }

    /// Reset the limit to empty state (used when all orders are removed)
    pub fn reset(&mut self) {
        self.size = 0;
        self.total_volume = 0;
        self.order_count = 0;
        self.head_order_index = None;
        self.tail_order_index = None;
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Limit[{}]: {} orders, {} shares, volume: {}",
            self.price, self.order_count, self.size, self.total_volume
        )
    }
}

impl PartialOrd for Limit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Limit {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price.cmp(&other.price)
    }
}

impl Eq for Limit {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_creation() {
        let limit = Limit::new(5000, Side::Buy);
        assert_eq!(limit.price, 5000);
        assert_eq!(limit.size, 0);
        assert_eq!(limit.total_volume, 0);
        assert_eq!(limit.order_count, 0);
        assert!(limit.is_empty());
        assert!(limit.is_leaf());
    }

    #[test]
    fn test_limit_add_order_stats() {
        let mut limit = Limit::new(5000, Side::Buy);

        limit.add_order_stats(100);
        assert_eq!(limit.size, 100);
        assert_eq!(limit.total_volume, 500000);
        assert_eq!(limit.order_count, 1);
        assert!(!limit.is_empty());

        limit.add_order_stats(50);
        assert_eq!(limit.size, 150);
        assert_eq!(limit.total_volume, 750000);
        assert_eq!(limit.order_count, 2);
    }

    #[test]
    fn test_limit_remove_order_stats() {
        let mut limit = Limit::new(5000, Side::Buy);
        limit.add_order_stats(100);
        limit.add_order_stats(50);

        limit.remove_order_stats(50);
        assert_eq!(limit.size, 100);
        assert_eq!(limit.total_volume, 500000);
        assert_eq!(limit.order_count, 1);

        limit.remove_order_stats(100);
        assert_eq!(limit.size, 0);
        assert_eq!(limit.total_volume, 0);
        assert_eq!(limit.order_count, 0);
        assert!(limit.is_empty());
    }

    #[test]
    fn test_limit_update_order_stats() {
        let mut limit = Limit::new(5000, Side::Buy);
        limit.add_order_stats(100);

        // Increase quantity
        limit.update_order_stats(100, 150);
        assert_eq!(limit.size, 150);
        assert_eq!(limit.total_volume, 750000);

        // Decrease quantity
        limit.update_order_stats(150, 75);
        assert_eq!(limit.size, 75);
        assert_eq!(limit.total_volume, 375000);
    }
}
