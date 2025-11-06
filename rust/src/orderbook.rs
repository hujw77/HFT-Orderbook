//! Simple OrderBook implementation - Pure Data Structure
//! 
//! This implementation follows the C/Python design pattern:
//! - Pure order book data structure without matching logic
//! - Basic operations: add, remove, update orders
//! - Query operations: best prices, volume at levels, etc.
//! - External matching engine handles order matching

use crate::avl_tree::{AvlNode, AvlTree};
use crate::limit::Limit;
use crate::order::Order;
use crate::types::{OrderId, Price, Quantity, Side, Timestamp};
use crate::{OrderBookError, Result};
use std::collections::HashMap;

/// Pure limit order book data structure
/// 
/// This implementation provides:
/// - O(1) operations for add, cancel, and update orders
/// - O(log M) for the first order at a new price level
/// - O(1) queries for best bid/ask and volume at price levels
#[derive(Debug)]
pub struct OrderBook {
    /// All orders indexed by order ID
    orders: Vec<Option<Order>>,
    /// Free indices in the orders vector
    free_order_indices: Vec<usize>,
    /// Map from order ID to index in orders vector
    order_id_to_index: HashMap<OrderId, usize>,
    
    /// All limit levels
    limits: Vec<Option<Limit>>,
    /// Free indices in the limits vector
    free_limit_indices: Vec<usize>,
    /// Map from price to index in limits vector
    price_to_limit_index: HashMap<Price, usize>,
    
    /// Root of buy tree (highest prices first)
    buy_tree_root: Option<usize>,
    /// Root of sell tree (lowest prices first)  
    sell_tree_root: Option<usize>,
    
    /// Best bid (highest buy price)
    best_bid_index: Option<usize>,
    /// Best ask (lowest sell price)
    best_ask_index: Option<usize>,
    
    /// Current timestamp for operations
    current_time: Timestamp,
}

impl OrderBook {
    /// Create a new empty orderbook
    pub fn new() -> Self {
        Self {
            orders: Vec::new(),
            free_order_indices: Vec::new(),
            order_id_to_index: HashMap::new(),
            limits: Vec::new(),
            free_limit_indices: Vec::new(),
            price_to_limit_index: HashMap::new(),
            buy_tree_root: None,
            sell_tree_root: None,
            best_bid_index: None,
            best_ask_index: None,
            current_time: 0,
        }
    }

    /// Create a new orderbook with pre-allocated capacity
    pub fn with_capacity(order_capacity: usize, limit_capacity: usize) -> Self {
        Self {
            orders: Vec::with_capacity(order_capacity),
            free_order_indices: Vec::new(),
            order_id_to_index: HashMap::with_capacity(order_capacity),
            limits: Vec::with_capacity(limit_capacity),
            free_limit_indices: Vec::new(),
            price_to_limit_index: HashMap::with_capacity(limit_capacity),
            buy_tree_root: None,
            sell_tree_root: None,
            best_bid_index: None,
            best_ask_index: None,
            current_time: 0,
        }
    }

    /// Set the current timestamp
    pub fn set_time(&mut self, timestamp: Timestamp) {
        self.current_time = timestamp;
    }

    /// Get the current timestamp
    pub fn current_time(&self) -> Timestamp {
        self.current_time
    }

    /// Get the best bid price and quantity
    pub fn best_bid(&self) -> Option<(Price, Quantity)> {
        self.best_bid_index
            .and_then(|idx| self.limits[idx].as_ref())
            .map(|limit| (limit.price, limit.size))
    }

    /// Get the best ask price and quantity
    pub fn best_ask(&self) -> Option<(Price, Quantity)> {
        self.best_ask_index
            .and_then(|idx| self.limits[idx].as_ref())
            .map(|limit| (limit.price, limit.size))
    }

    /// Get the spread (difference between best ask and best bid)
    pub fn spread(&self) -> Option<Price> {
        match (self.best_ask(), self.best_bid()) {
            (Some((ask_price, _)), Some((bid_price, _))) => {
                if ask_price > bid_price {
                    Some(ask_price - bid_price)
                } else {
                    Some(0)
                }
            }
            _ => None,
        }
    }

    /// Get the mid price (average of best bid and ask)
    pub fn mid_price(&self) -> Option<Price> {
        match (self.best_ask(), self.best_bid()) {
            (Some((ask_price, _)), Some((bid_price, _))) => {
                Some((ask_price + bid_price) / 2)
            }
            _ => None,
        }
    }

    /// Get total volume at a specific price level
    pub fn volume_at_price(&self, price: Price) -> Option<Quantity> {
        self.price_to_limit_index
            .get(&price)
            .and_then(|&idx| self.limits[idx].as_ref())
            .map(|limit| limit.size)
    }

    /// Get number of orders at a specific price level
    pub fn orders_at_price(&self, price: Price) -> Option<usize> {
        self.price_to_limit_index
            .get(&price)
            .and_then(|&idx| self.limits[idx].as_ref())
            .map(|limit| limit.order_count)
    }

    /// Check if an order exists
    pub fn contains_order(&self, order_id: OrderId) -> bool {
        self.order_id_to_index.contains_key(&order_id)
    }

    /// Get an order by ID
    pub fn get_order(&self, order_id: OrderId) -> Option<&Order> {
        self.order_id_to_index
            .get(&order_id)
            .and_then(|&idx| self.orders[idx].as_ref())
    }

    /// Get total number of orders in the book
    pub fn total_orders(&self) -> usize {
        self.order_id_to_index.len()
    }

    /// Get total number of price levels
    pub fn total_levels(&self) -> usize {
        self.price_to_limit_index.len()
    }

    /// Add a new order to the book
    ///
    /// This is a pure data structure operation - no matching logic.
    /// The order is simply added to the appropriate price level.
    pub fn add_order(&mut self, mut order: Order) -> Result<()> {
        // Validate order
        if order.price == 0 {
            return Err(OrderBookError::InvalidPrice(order.price));
        }
        if order.quantity == 0 {
            return Err(OrderBookError::InvalidQuantity(order.quantity));
        }
        if self.contains_order(order.id) {
            return Err(OrderBookError::OrderAlreadyExists(order.id));
        }

        order.event_time = self.current_time;
        self.add_order_to_book(order)?;
        Ok(())
    }

    /// Process an order (similar to Python's process method)
    ///
    /// If the order's quantity is 0, it is removed from the book.
    /// If its quantity isn't zero and it exists within the book, the order is updated.
    /// If it doesn't exist, it will be added.
    pub fn process_order(&mut self, order: Order) -> Result<()> {
        if order.quantity == 0 {
            self.remove_order(order.id)?;
        } else if self.contains_order(order.id) {
            self.update_order(order.id, order.quantity)?;
        } else {
            self.add_order(order)?;
        }
        Ok(())
    }

    /// Remove an order from the book
    pub fn remove_order(&mut self, order_id: OrderId) -> Result<Order> {
        let order_idx = self.order_id_to_index
            .get(&order_id)
            .copied()
            .ok_or(OrderBookError::OrderNotFound(order_id))?;

        let order = self.orders[order_idx].as_mut().unwrap();
        let limit_idx = order.parent_limit_index.unwrap();

        // Mark order as cancelled
        order.cancel(self.current_time);
        let cancelled_order = order.clone();

        // Remove from limit
        self.remove_order_from_limit(order_idx, limit_idx)?;

        Ok(cancelled_order)
    }

    /// Update an order's quantity
    pub fn update_order(&mut self, order_id: OrderId, new_quantity: Quantity) -> Result<()> {
        if new_quantity == 0 {
            return Err(OrderBookError::InvalidQuantity(new_quantity));
        }

        let order_idx = self.order_id_to_index
            .get(&order_id)
            .copied()
            .ok_or(OrderBookError::OrderNotFound(order_id))?;

        let order = self.orders[order_idx].as_mut().unwrap();
        let limit_idx = order.parent_limit_index.unwrap();
        let old_quantity = order.remaining_quantity;

        // Update order quantity
        if !order.update_quantity(new_quantity, self.current_time) {
            return Err(OrderBookError::InvalidQuantity(new_quantity));
        }

        // Update limit statistics
        self.limits[limit_idx].as_mut().unwrap()
            .update_order_stats(old_quantity, order.remaining_quantity);

        Ok(())
    }

    /// Get price levels (similar to Python's levels method)
    /// Returns a vector of (price, quantity) tuples for each side
    pub fn get_levels(&self, depth: Option<usize>) -> (Vec<(Price, Quantity)>, Vec<(Price, Quantity)>) {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        // Collect all price levels
        let mut prices: Vec<Price> = self.price_to_limit_index.keys().copied().collect();
        prices.sort();

        // Separate bids and asks based on best bid/ask
        let _mid_price = self.mid_price().unwrap_or(0);

        for price in prices {
            if let Some(limit) = self.price_to_limit_index.get(&price)
                .and_then(|&idx| self.limits[idx].as_ref()) {

                if limit.side == Side::Buy {
                    bids.push((price, limit.size));
                } else {
                    asks.push((price, limit.size));
                }
            }
        }

        // Sort bids descending (highest first), asks ascending (lowest first)
        bids.sort_by(|a, b| b.0.cmp(&a.0));
        asks.sort_by(|a, b| a.0.cmp(&b.0));

        // Apply depth limit if specified
        if let Some(d) = depth {
            bids.truncate(d);
            asks.truncate(d);
        }

        (bids, asks)
    }

    // Internal helper methods

    /// Allocate a new order index
    fn allocate_order_index(&mut self) -> usize {
        if let Some(index) = self.free_order_indices.pop() {
            index
        } else {
            let index = self.orders.len();
            self.orders.push(None);
            index
        }
    }

    /// Free an order index
    fn free_order_index(&mut self, index: usize) {
        self.orders[index] = None;
        self.free_order_indices.push(index);
    }

    /// Allocate a new limit index
    fn allocate_limit_index(&mut self) -> usize {
        if let Some(index) = self.free_limit_indices.pop() {
            index
        } else {
            let index = self.limits.len();
            self.limits.push(None);
            index
        }
    }

    /// Free a limit index
    fn free_limit_index(&mut self, index: usize) {
        self.limits[index] = None;
        self.free_limit_indices.push(index);
    }

    /// Add an order to the book (internal implementation)
    fn add_order_to_book(&mut self, order: Order) -> Result<()> {
        let order_idx = self.allocate_order_index();
        let order_id = order.id;
        let price = order.price;
        let side = order.side;
        let quantity = order.remaining_quantity;

        // Store the order first
        self.orders[order_idx] = Some(order);
        self.order_id_to_index.insert(order_id, order_idx);

        // Get or create limit level
        let limit_idx = self.get_or_create_limit(price, side)?;

        // Add order to the limit's linked list
        self.add_order_to_limit(order_idx, limit_idx, quantity)?;

        // Update best bid/ask if necessary
        self.update_best_prices(limit_idx, side);

        Ok(())
    }

    /// Get or create a limit level at the specified price
    fn get_or_create_limit(&mut self, price: Price, side: Side) -> Result<usize> {
        if let Some(&limit_idx) = self.price_to_limit_index.get(&price) {
            Ok(limit_idx)
        } else {
            // Create new limit
            let limit_idx = self.allocate_limit_index();
            let limit = Limit::new(price, side);
            self.limits[limit_idx] = Some(limit);
            self.price_to_limit_index.insert(price, limit_idx);

            // Add to appropriate tree based on order side
            match side {
                Side::Buy => {
                    self.buy_tree_root = Some(self.insert_into_tree(self.buy_tree_root, limit_idx));
                }
                Side::Sell => {
                    self.sell_tree_root = Some(self.insert_into_tree(self.sell_tree_root, limit_idx));
                }
            }

            Ok(limit_idx)
        }
    }

    /// Add an order to a limit's linked list
    fn add_order_to_limit(&mut self, order_idx: usize, limit_idx: usize, quantity: Quantity) -> Result<()> {
        // Get the tail index before borrowing
        let tail_idx = self.limits[limit_idx].as_ref().unwrap().tail_order_index;

        // Update order's parent limit
        self.orders[order_idx].as_mut().unwrap().parent_limit_index = Some(limit_idx);

        // Add to tail of linked list (FIFO)
        if let Some(tail_idx) = tail_idx {
            // List is not empty - update the current tail to point to new order
            self.orders[tail_idx].as_mut().unwrap().next_order_index = Some(order_idx);
            self.orders[order_idx].as_mut().unwrap().prev_order_index = Some(tail_idx);
            self.limits[limit_idx].as_mut().unwrap().tail_order_index = Some(order_idx);
        } else {
            // List is empty
            let limit = self.limits[limit_idx].as_mut().unwrap();
            limit.head_order_index = Some(order_idx);
            limit.tail_order_index = Some(order_idx);
        }

        // Update limit statistics
        self.limits[limit_idx].as_mut().unwrap().add_order_stats(quantity);

        Ok(())
    }

    /// Remove an order from a limit's linked list
    fn remove_order_from_limit(&mut self, order_idx: usize, limit_idx: usize) -> Result<()> {
        // Extract order data before borrowing mutably
        let (prev_idx, next_idx, order_id, quantity) = {
            let order = self.orders[order_idx].as_ref().unwrap();
            (order.prev_order_index, order.next_order_index, order.id, order.remaining_quantity)
        };

        // Update linked list pointers
        if let Some(prev) = prev_idx {
            self.orders[prev].as_mut().unwrap().next_order_index = next_idx;
        } else {
            // This was the head
            self.limits[limit_idx].as_mut().unwrap().head_order_index = next_idx;
        }

        if let Some(next) = next_idx {
            self.orders[next].as_mut().unwrap().prev_order_index = prev_idx;
        } else {
            // This was the tail
            self.limits[limit_idx].as_mut().unwrap().tail_order_index = prev_idx;
        }

        // Update limit statistics
        self.limits[limit_idx].as_mut().unwrap().remove_order_stats(quantity);

        // Remove order from tracking
        self.order_id_to_index.remove(&order_id);
        self.free_order_index(order_idx);

        // If limit is now empty, remove it
        if self.limits[limit_idx].as_ref().unwrap().is_empty() {
            self.remove_empty_limit(limit_idx)?;
        }

        Ok(())
    }

    /// Remove an empty limit level
    fn remove_empty_limit(&mut self, limit_idx: usize) -> Result<()> {
        let limit = self.limits[limit_idx].as_ref().unwrap();
        let price = limit.price;
        let side = limit.side;

        // Remove from price mapping
        self.price_to_limit_index.remove(&price);

        // Remove from appropriate tree based on side
        match side {
            Side::Buy => {
                self.buy_tree_root = self.remove_from_tree(self.buy_tree_root, limit_idx);
            }
            Side::Sell => {
                self.sell_tree_root = self.remove_from_tree(self.sell_tree_root, limit_idx);
            }
        }

        // Update best prices if this was the best
        if Some(limit_idx) == self.best_bid_index {
            self.best_bid_index = self.find_new_best_bid();
        }
        if Some(limit_idx) == self.best_ask_index {
            self.best_ask_index = self.find_new_best_ask();
        }

        // Free the limit
        self.free_limit_index(limit_idx);

        Ok(())
    }

    /// Update best bid/ask prices
    fn update_best_prices(&mut self, limit_idx: usize, side: Side) {
        let price = self.limits[limit_idx].as_ref().unwrap().price;

        match side {
            Side::Buy => {
                if self.best_bid_index.is_none() ||
                   price > self.limits[self.best_bid_index.unwrap()].as_ref().unwrap().price {
                    self.best_bid_index = Some(limit_idx);
                }
            }
            Side::Sell => {
                if self.best_ask_index.is_none() ||
                   price < self.limits[self.best_ask_index.unwrap()].as_ref().unwrap().price {
                    self.best_ask_index = Some(limit_idx);
                }
            }
        }
    }

    /// Find new best bid after removal
    fn find_new_best_bid(&self) -> Option<usize> {
        self.find_max_in_tree_with_orders(self.buy_tree_root)
    }

    /// Find new best ask after removal
    fn find_new_best_ask(&self) -> Option<usize> {
        self.find_min_in_tree_with_orders(self.sell_tree_root)
    }

    /// Find maximum node in tree that has orders
    fn find_max_in_tree_with_orders(&self, root: Option<usize>) -> Option<usize> {
        self.find_max_with_orders_recursive(root)
    }

    /// Find minimum node in tree that has orders
    fn find_min_in_tree_with_orders(&self, root: Option<usize>) -> Option<usize> {
        self.find_min_with_orders_recursive(root)
    }

    /// Recursively find the maximum node with orders
    fn find_max_with_orders_recursive(&self, root: Option<usize>) -> Option<usize> {
        match root {
            None => None,
            Some(idx) => {
                // Check if this limit has orders
                if let Some(limit) = self.limits[idx].as_ref() {
                    if !limit.is_empty() {
                        // This node has orders, check if there's a larger one in right subtree
                        if let Some(right_result) = self.find_max_with_orders_recursive(limit.avl_node.right_child) {
                            Some(right_result)
                        } else {
                            Some(idx)
                        }
                    } else {
                        // This node is empty, check both subtrees
                        if let Some(right_result) = self.find_max_with_orders_recursive(limit.avl_node.right_child) {
                            Some(right_result)
                        } else {
                            self.find_max_with_orders_recursive(limit.avl_node.left_child)
                        }
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Recursively find the minimum node with orders
    fn find_min_with_orders_recursive(&self, root: Option<usize>) -> Option<usize> {
        match root {
            None => None,
            Some(idx) => {
                // Check if this limit has orders
                if let Some(limit) = self.limits[idx].as_ref() {
                    if !limit.is_empty() {
                        // This node has orders, check if there's a smaller one in left subtree
                        if let Some(left_result) = self.find_min_with_orders_recursive(limit.avl_node.left_child) {
                            Some(left_result)
                        } else {
                            Some(idx)
                        }
                    } else {
                        // This node is empty, check both subtrees
                        if let Some(left_result) = self.find_min_with_orders_recursive(limit.avl_node.left_child) {
                            Some(left_result)
                        } else {
                            self.find_min_with_orders_recursive(limit.avl_node.right_child)
                        }
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Insert a limit into the tree (simplified BST, no balancing)
    fn insert_into_tree(&mut self, root: Option<usize>, limit_idx: usize) -> usize {
        match root {
            None => limit_idx,
            Some(root_idx) => {
                let limit_price = self.limits[limit_idx].as_ref().unwrap().price;
                let root_price = self.limits[root_idx].as_ref().unwrap().price;

                if limit_price < root_price {
                    let new_left = self.insert_into_tree(
                        self.limits[root_idx].as_ref().unwrap().avl_node.left_child,
                        limit_idx
                    );
                    self.limits[root_idx].as_mut().unwrap().avl_node.left_child = Some(new_left);
                    self.limits[new_left].as_mut().unwrap().avl_node.parent = Some(root_idx);
                } else if limit_price > root_price {
                    let new_right = self.insert_into_tree(
                        self.limits[root_idx].as_ref().unwrap().avl_node.right_child,
                        limit_idx
                    );
                    self.limits[root_idx].as_mut().unwrap().avl_node.right_child = Some(new_right);
                    self.limits[new_right].as_mut().unwrap().avl_node.parent = Some(root_idx);
                }

                // Return root without balancing
                root_idx
            }
        }
    }

    /// Remove a limit from the tree (simplified)
    fn remove_from_tree(&mut self, root: Option<usize>, limit_idx: usize) -> Option<usize> {
        match root {
            None => None,
            Some(root_idx) => {
                if root_idx == limit_idx {
                    // This is the node to remove
                    let node = &self.limits[root_idx].as_ref().unwrap().avl_node;

                    match (node.left_child, node.right_child) {
                        (None, None) => None,
                        (Some(left), None) => {
                            self.limits[left].as_mut().unwrap().avl_node.parent = node.parent;
                            Some(left)
                        }
                        (None, Some(right)) => {
                            self.limits[right].as_mut().unwrap().avl_node.parent = node.parent;
                            Some(right)
                        }
                        (Some(_), Some(right)) => {
                            // Find successor (minimum in right subtree)
                            let successor_idx = self.find_min_in_subtree(right);

                            // Replace current node's data with successor's data
                            let successor_price = self.limits[successor_idx].as_ref().unwrap().price;
                            self.limits[root_idx].as_mut().unwrap().price = successor_price;

                            // Remove successor from right subtree
                            let new_right = self.remove_from_tree(Some(right), successor_idx);
                            self.limits[root_idx].as_mut().unwrap().avl_node.right_child = new_right;

                            Some(root_idx)
                        }
                    }
                } else {
                    let limit_price = self.limits[limit_idx].as_ref().unwrap().price;
                    let root_price = self.limits[root_idx].as_ref().unwrap().price;

                    if limit_price < root_price {
                        let new_left = self.remove_from_tree(
                            self.limits[root_idx].as_ref().unwrap().avl_node.left_child,
                            limit_idx
                        );
                        self.limits[root_idx].as_mut().unwrap().avl_node.left_child = new_left;
                    } else {
                        let new_right = self.remove_from_tree(
                            self.limits[root_idx].as_ref().unwrap().avl_node.right_child,
                            limit_idx
                        );
                        self.limits[root_idx].as_mut().unwrap().avl_node.right_child = new_right;
                    }

                    Some(root_idx)
                }
            }
        }
    }

    /// Find minimum node in subtree (for tree operations)
    fn find_min_in_subtree(&self, mut index: usize) -> usize {
        while let Some(left) = self.limits[index].as_ref().unwrap().avl_node.left_child {
            index = left;
        }
        index
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

impl AvlTree<Limit> for OrderBook {
    fn get_price(&self, index: usize) -> Price {
        self.limits[index].as_ref()
            .expect(&format!("Limit at index {} should exist", index))
            .price
    }

    fn get_node(&self, index: usize) -> &AvlNode {
        &self.limits[index].as_ref()
            .expect(&format!("Limit at index {} should exist", index))
            .avl_node
    }

    fn get_node_mut(&mut self, index: usize) -> &mut AvlNode {
        &mut self.limits[index].as_mut()
            .expect(&format!("Limit at index {} should exist", index))
            .avl_node
    }
}
