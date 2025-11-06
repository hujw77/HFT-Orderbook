//! Matching Engine - Separate from OrderBook Data Structure
//! 
//! This module implements the order matching logic that operates on the
//! pure OrderBook data structure. This separation follows the C/Python
//! design pattern where the orderbook is just a data structure.

use crate::orderbook::OrderBook;
use crate::order::Order;
use crate::types::{Price, Side, Trade};
use crate::{OrderBookError, Result};

/// External matching engine that operates on OrderBook
pub struct MatchingEngine {
    // In a real system, this might be a separate service
    // For now, it's stateless
}

impl MatchingEngine {
    /// Create a new matching engine
    pub fn new() -> Self {
        Self {}
    }

    /// Process an order with matching logic
    /// 
    /// This method combines the pure orderbook operations with matching logic:
    /// 1. Try to match the incoming order against existing orders
    /// 2. Add any remaining quantity to the book
    /// 3. Return the list of trades generated
    pub fn process_order(&self, book: &mut OrderBook, mut order: Order) -> Result<Vec<Trade>> {
        // Try to match the order first
        let trades = if order.side == Side::Buy {
            self.match_buy_order(book, &mut order)?
        } else {
            self.match_sell_order(book, &mut order)?
        };

        // If there's remaining quantity, add to book
        if order.quantity > 0 {
            book.add_order(order)?;
        }

        Ok(trades)
    }

    /// Match a buy order against existing sell orders
    fn match_buy_order(&self, book: &mut OrderBook, order: &mut Order) -> Result<Vec<Trade>> {
        let mut trades = Vec::new();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10000; // Safety limit

        while order.quantity > 0 {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(OrderBookError::TreeError(
                    "Too many iterations in match_buy_order - possible infinite loop".to_string()
                ));
            }

            // Get best ask
            let (ask_price, _) = match book.best_ask() {
                Some(ask) => ask,
                None => break, // No asks available
            };

            // Check if prices cross
            if order.price < ask_price {
                break; // No more matches possible
            }

            // Execute trade at the best ask price
            if let Some(trade) = self.execute_at_price(book, order, ask_price)? {
                trades.push(trade);
            } else {
                // No trade occurred, break to avoid infinite loop
                break;
            }
        }

        Ok(trades)
    }

    /// Match a sell order against existing buy orders
    fn match_sell_order(&self, book: &mut OrderBook, order: &mut Order) -> Result<Vec<Trade>> {
        let mut trades = Vec::new();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10000; // Safety limit

        while order.quantity > 0 {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(OrderBookError::TreeError(
                    "Too many iterations in match_sell_order - possible infinite loop".to_string()
                ));
            }

            // Get best bid
            let (bid_price, _) = match book.best_bid() {
                Some(bid) => bid,
                None => break, // No bids available
            };

            // Check if prices cross
            if order.price > bid_price {
                break; // No more matches possible
            }

            // Execute trade at the best bid price
            if let Some(trade) = self.execute_at_price(book, order, bid_price)? {
                trades.push(trade);
            } else {
                // No trade occurred, break to avoid infinite loop
                break;
            }
        }

        Ok(trades)
    }

    /// Execute a trade at a specific price level
    /// 
    /// This is a simplified implementation that assumes we can find and match
    /// orders at the given price. In a real implementation, this would need
    /// to interact more closely with the orderbook's internal structure.
    fn execute_at_price(&self, book: &mut OrderBook, incoming_order: &mut Order, price: Price) -> Result<Option<Trade>> {
        // This is a simplified implementation
        // In reality, we'd need access to the orderbook's internal order management
        // For now, we'll simulate a trade by reducing the incoming order quantity
        
        let available_quantity = book.volume_at_price(price).unwrap_or(0);
        if available_quantity == 0 {
            return Ok(None);
        }

        let trade_quantity = incoming_order.quantity.min(available_quantity);
        
        // Create a dummy passive order ID for the trade
        // In a real implementation, we'd get this from the actual order being matched
        let passive_order_id = 999999; // Placeholder
        
        let trade = Trade::new(
            incoming_order.id,
            passive_order_id,
            price,
            trade_quantity,
            book.current_time(),
            incoming_order.side,
        );

        // Update the incoming order
        incoming_order.fill(trade_quantity, book.current_time());

        // Note: In a real implementation, we would also need to:
        // 1. Find the actual passive order(s) at this price level
        // 2. Update/remove them from the book
        // 3. Handle partial fills correctly
        // This simplified version is just for demonstration

        Ok(Some(trade))
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}
