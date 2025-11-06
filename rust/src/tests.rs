//! Simple tests for the new orderbook design

use crate::{OrderBook, Order, Side, OrderBookError};

#[test]
fn test_empty_orderbook() {
    let book = OrderBook::new();
    assert_eq!(book.total_orders(), 0);
    assert_eq!(book.total_levels(), 0);
    assert_eq!(book.best_bid(), None);
    assert_eq!(book.best_ask(), None);
    assert_eq!(book.spread(), None);
    assert_eq!(book.mid_price(), None);
}

#[test]
fn test_add_single_order() {
    let mut book = OrderBook::new();
    book.set_time(1000);

    let order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
    book.add_order(order).unwrap();
    
    assert_eq!(book.total_orders(), 1);
    assert_eq!(book.total_levels(), 1);
    assert_eq!(book.best_bid(), Some((5000, 100)));
    assert_eq!(book.best_ask(), None);
    assert!(book.contains_order(1));
}

#[test]
fn test_add_multiple_orders() {
    let mut book = OrderBook::new();
    book.set_time(1000);

    let buy_order = Order::new(1, Side::Buy, 100, 4950, 1000, 1);
    let sell_order = Order::new(2, Side::Sell, 150, 5050, 1001, 1);

    book.add_order(buy_order).unwrap();
    book.add_order(sell_order).unwrap();
    
    assert_eq!(book.total_orders(), 2);
    assert_eq!(book.total_levels(), 2);
    assert_eq!(book.best_bid(), Some((4950, 100)));
    assert_eq!(book.best_ask(), Some((5050, 150)));
    assert_eq!(book.spread(), Some(100));
    assert_eq!(book.mid_price(), Some(5000));
}

#[test]
fn test_remove_order() {
    let mut book = OrderBook::new();
    book.set_time(1000);

    let order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
    book.add_order(order).unwrap();
    
    assert_eq!(book.total_orders(), 1);
    assert!(book.contains_order(1));

    let removed = book.remove_order(1).unwrap();
    assert_eq!(removed.id, 1);
    assert_eq!(book.total_orders(), 0);
    assert!(!book.contains_order(1));
    assert_eq!(book.best_bid(), None);
}

#[test]
fn test_update_order() {
    let mut book = OrderBook::new();
    book.set_time(1000);

    let order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
    book.add_order(order).unwrap();
    
    assert_eq!(book.volume_at_price(5000), Some(100));

    book.update_order(1, 150).unwrap();
    assert_eq!(book.volume_at_price(5000), Some(150));
}

#[test]
fn test_process_order() {
    let mut book = OrderBook::new();
    book.set_time(1000);

    // Add new order
    let order1 = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
    book.process_order(order1).unwrap();
    assert_eq!(book.total_orders(), 1);

    // Update existing order
    let order2 = Order::new(1, Side::Buy, 150, 5000, 1001, 1);
    book.process_order(order2).unwrap();
    assert_eq!(book.total_orders(), 1);
    assert_eq!(book.volume_at_price(5000), Some(150));

    // Remove order (quantity = 0)
    let order3 = Order::new(1, Side::Buy, 0, 5000, 1002, 1);
    book.process_order(order3).unwrap();
    assert_eq!(book.total_orders(), 0);
}


#[test]
fn test_price_levels() {
    let mut book = OrderBook::new();
    book.set_time(1000);

    // Add multiple orders at different prices
    book.add_order(Order::new(1, Side::Buy, 100, 4950, 1000, 1)).unwrap();
    book.add_order(Order::new(2, Side::Buy, 200, 4940, 1001, 1)).unwrap();
    book.add_order(Order::new(3, Side::Sell, 150, 5050, 1002, 1)).unwrap();
    book.add_order(Order::new(4, Side::Sell, 100, 5060, 1003, 1)).unwrap();

    let (bids, asks) = book.get_levels(None);
    
    assert_eq!(bids.len(), 2);
    assert_eq!(asks.len(), 2);
    
    // Bids should be sorted descending (highest first)
    assert_eq!(bids[0], (4950, 100));
    assert_eq!(bids[1], (4940, 200));
    
    // Asks should be sorted ascending (lowest first)
    assert_eq!(asks[0], (5050, 150));
    assert_eq!(asks[1], (5060, 100));
}

#[test]
fn test_error_cases() {
    let mut book = OrderBook::new();
    book.set_time(1000);

    // Invalid price
    let invalid_price_order = Order::new(1, Side::Buy, 100, 0, 1000, 1);
    assert!(matches!(book.add_order(invalid_price_order), Err(OrderBookError::InvalidPrice(0))));

    // Invalid quantity
    let invalid_qty_order = Order::new(1, Side::Buy, 0, 5000, 1000, 1);
    assert!(matches!(book.add_order(invalid_qty_order), Err(OrderBookError::InvalidQuantity(0))));

    // Add valid order
    let order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
    book.add_order(order).unwrap();

    // Duplicate order ID
    let duplicate_order = Order::new(1, Side::Sell, 50, 5100, 1001, 1);
    assert!(matches!(book.add_order(duplicate_order), Err(OrderBookError::OrderAlreadyExists(1))));

    // Remove non-existent order
    assert!(matches!(book.remove_order(999), Err(OrderBookError::OrderNotFound(999))));
}

// ============================================================================
// Tests based on Python implementation (orderbook_tests.py)
// ============================================================================

#[test]
fn test_adding_new_order_works() {
    // Based on Python test: test_adding_a_new_order_works
    let mut book = OrderBook::new();
    book.set_time(1000);

    let bid_order = Order::new(1, Side::Buy, 5, 100, 1000, 1);
    let ask_order = Order::new(2, Side::Sell, 5, 200, 1001, 1);
    
    book.add_order(bid_order).unwrap();
    book.add_order(ask_order).unwrap();
    
    assert_eq!(book.best_ask(), Some((200, 5)));
    assert_eq!(book.best_bid(), Some((100, 5)));
    
    // Check that orders exist
    assert!(book.contains_order(1));
    assert!(book.contains_order(2));
    
    // Check volume at price levels
    assert_eq!(book.volume_at_price(100), Some(5));
    assert_eq!(book.volume_at_price(200), Some(5));
    
    // Test updating an order
    book.update_order(1, 4).unwrap();
    assert_eq!(book.volume_at_price(100), Some(4));
    
    // Add another order at the same price level
    let bid_order_2 = Order::new(3, Side::Buy, 5, 100, 1002, 1);
    book.add_order(bid_order_2).unwrap();
    
    // Check that volume increased and we have 2 orders at this price
    assert_eq!(book.volume_at_price(100), Some(9));
    assert_eq!(book.total_orders(), 3);
}

#[test]
fn test_removing_orders_works() {
    // Based on Python test: test_removing_orders_works
    let mut book = OrderBook::new();
    book.set_time(1000);

    let bid_order = Order::new(1, Side::Buy, 5, 100, 1000, 1);
    let bid_order_2 = Order::new(2, Side::Buy, 10, 100, 1001, 1);
    let ask_order = Order::new(3, Side::Sell, 10, 200, 1002, 1);
    let ask_order_2 = Order::new(4, Side::Sell, 10, 200, 1003, 1);
    
    book.add_order(bid_order).unwrap();
    book.add_order(bid_order_2).unwrap();
    book.add_order(ask_order).unwrap();
    book.add_order(ask_order_2).unwrap();
    
    // Check initial state
    assert_eq!(book.volume_at_price(100), Some(15));
    assert_eq!(book.total_orders(), 4);
    
    // Remove first bid order
    book.remove_order(1).unwrap();
    assert_eq!(book.volume_at_price(100), Some(10));
    assert_eq!(book.total_orders(), 3);
    assert!(!book.contains_order(1));
    assert!(book.contains_order(2));
    
    // Remove the last order at price level - should remove the limit
    book.remove_order(2).unwrap();
    assert_eq!(book.volume_at_price(100), None);
    assert_eq!(book.total_orders(), 2);
    assert_eq!(book.best_bid(), None);
    
    // Asks should still be there
    assert_eq!(book.volume_at_price(200), Some(20));
}

#[test]
fn test_querying_levels_works() {
    // Based on Python test: test_querying_levels_works
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    // Load book with multiple price levels
    let orders = vec![
        Order::new(1, Side::Buy, 5, 100, 1000, 1),
        Order::new(2, Side::Buy, 5, 95, 1001, 1),
        Order::new(3, Side::Buy, 5, 90, 1002, 1),
        Order::new(4, Side::Sell, 5, 200, 1003, 1),
        Order::new(5, Side::Sell, 5, 205, 1004, 1),
        Order::new(6, Side::Sell, 5, 210, 1005, 1),
    ];
    
    for order in orders {
        book.add_order(order).unwrap();
    }
    
    let (bids, asks) = book.get_levels(None);
    
    // Check that we have correct number of levels
    assert_eq!(bids.len(), 3);
    assert_eq!(asks.len(), 3);
    
    // Check that bids are sorted descending (highest first)
    assert_eq!(bids[0].0, 100);
    assert_eq!(bids[1].0, 95);
    assert_eq!(bids[2].0, 90);
    
    // Check that asks are sorted ascending (lowest first)
    assert_eq!(asks[0].0, 200);
    assert_eq!(asks[1].0, 205);
    assert_eq!(asks[2].0, 210);
}

#[test]
fn test_querying_levels_limit_depth() {
    // Based on Python test: test_querying_levels_limit_depth
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    // Load book with multiple price levels
    let orders = vec![
        Order::new(1, Side::Buy, 5, 100, 1000, 1),
        Order::new(2, Side::Buy, 5, 95, 1001, 1),
        Order::new(3, Side::Buy, 5, 90, 1002, 1),
        Order::new(4, Side::Sell, 5, 200, 1003, 1),
        Order::new(5, Side::Sell, 5, 205, 1004, 1),
        Order::new(6, Side::Sell, 5, 210, 1005, 1),
    ];
    
    for order in orders {
        book.add_order(order).unwrap();
    }
    
    // Query with depth limit of 2
    let (bids, asks) = book.get_levels(Some(2));
    
    assert_eq!(bids.len(), 2);
    assert_eq!(asks.len(), 2);
    
    // Check that we got the top 2 levels
    assert_eq!(bids[0].0, 100);
    assert_eq!(bids[1].0, 95);
    assert_eq!(asks[0].0, 200);
    assert_eq!(asks[1].0, 205);
}

#[test]
fn test_update_order_changes_side() {
    // Based on Python test where order side changes during update
    let mut book = OrderBook::new();
    book.set_time(1000);

    let ask_order = Order::new(2, Side::Sell, 5, 200, 1000, 1);
    book.add_order(ask_order).unwrap();
    
    assert_eq!(book.best_ask(), Some((200, 5)));
    assert_eq!(book.volume_at_price(200), Some(5));
    
    // Note: In Rust implementation, we can't change side directly,
    // but we can test updating quantity and verify volume updates
    book.update_order(2, 4).unwrap();
    assert_eq!(book.volume_at_price(200), Some(4));
}

// ============================================================================
// Tests based on C implementation (testCases.c)
// ============================================================================

#[test]
fn test_order_pushing_multiple_orders() {
    // Based on C test: TestOrderPushing
    // Test adding multiple orders at the same price level
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    let order_a = Order::new(1, Side::Buy, 10, 1000, 1000, 1);
    let order_b = Order::new(2, Side::Buy, 20, 1000, 1001, 1);
    let order_c = Order::new(3, Side::Buy, 30, 1000, 1002, 1);
    
    // Push first order
    book.add_order(order_a).unwrap();
    assert_eq!(book.volume_at_price(1000), Some(10));
    assert_eq!(book.total_orders(), 1);
    
    // Push second order
    book.add_order(order_b).unwrap();
    assert_eq!(book.volume_at_price(1000), Some(30));
    assert_eq!(book.total_orders(), 2);
    
    // Push third order
    book.add_order(order_c).unwrap();
    assert_eq!(book.volume_at_price(1000), Some(60));
    assert_eq!(book.total_orders(), 3);
    
    // All orders should still exist
    assert!(book.contains_order(1));
    assert!(book.contains_order(2));
    assert!(book.contains_order(3));
}

#[test]
fn test_order_removal_from_middle() {
    // Based on C test: TestRemoveOrder
    // Test removing orders from middle of linked list
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    let order_a = Order::new(1, Side::Buy, 10, 1000, 1000, 1);
    let order_b = Order::new(2, Side::Buy, 20, 1000, 1001, 1);
    let order_c = Order::new(3, Side::Buy, 30, 1000, 1002, 1);
    
    book.add_order(order_a).unwrap();
    book.add_order(order_b).unwrap();
    book.add_order(order_c).unwrap();
    
    assert_eq!(book.volume_at_price(1000), Some(60));
    assert_eq!(book.total_orders(), 3);
    
    // Remove middle order
    book.remove_order(2).unwrap();
    assert_eq!(book.volume_at_price(1000), Some(40));
    assert_eq!(book.total_orders(), 2);
    assert!(!book.contains_order(2));
    assert!(book.contains_order(1));
    assert!(book.contains_order(3));
    
    // Remove first order
    book.remove_order(1).unwrap();
    assert_eq!(book.volume_at_price(1000), Some(30));
    assert_eq!(book.total_orders(), 1);
    assert!(book.contains_order(3));
    
    // Remove last order
    book.remove_order(3).unwrap();
    assert_eq!(book.volume_at_price(1000), None);
    assert_eq!(book.total_orders(), 0);
    assert_eq!(book.best_bid(), None);
}

#[test]
fn test_multiple_price_levels() {
    // Based on C test structure for tree operations
    // Test adding limits at different prices
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    // Add orders at different prices
    let order1 = Order::new(1, Side::Buy, 10, 100, 1000, 1);
    let order2 = Order::new(2, Side::Buy, 20, 200, 1001, 1);
    let order3 = Order::new(3, Side::Buy, 30, 50, 1002, 1);
    let order4 = Order::new(4, Side::Buy, 40, 45, 1003, 1);
    
    book.add_order(order1).unwrap();
    book.add_order(order2).unwrap();
    book.add_order(order3).unwrap();
    book.add_order(order4).unwrap();
    
    assert_eq!(book.total_levels(), 4);
    assert_eq!(book.total_orders(), 4);
    
    // Check that best bid is highest price
    assert_eq!(book.best_bid(), Some((200, 20)));
    
    // Check all price levels exist
    assert_eq!(book.volume_at_price(100), Some(10));
    assert_eq!(book.volume_at_price(200), Some(20));
    assert_eq!(book.volume_at_price(50), Some(30));
    assert_eq!(book.volume_at_price(45), Some(40));
}

#[test]
fn test_duplicate_price_levels() {
    // Test that adding duplicate price levels doesn't create new limits
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    let order1 = Order::new(1, Side::Buy, 10, 100, 1000, 1);
    let order2 = Order::new(2, Side::Buy, 20, 100, 1001, 1);
    
    book.add_order(order1).unwrap();
    book.add_order(order2).unwrap();
    
    // Should only have one price level
    assert_eq!(book.total_levels(), 1);
    assert_eq!(book.total_orders(), 2);
    assert_eq!(book.volume_at_price(100), Some(30));
}

#[test]
fn test_best_price_updates() {
    // Test that best bid/ask updates correctly when orders are added/removed
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    // Add bids in descending price order
    book.add_order(Order::new(1, Side::Buy, 10, 100, 1000, 1)).unwrap();
    assert_eq!(book.best_bid(), Some((100, 10)));
    
    book.add_order(Order::new(2, Side::Buy, 20, 95, 1001, 1)).unwrap();
    assert_eq!(book.best_bid(), Some((100, 10))); // Still best
    
    book.add_order(Order::new(3, Side::Buy, 30, 105, 1002, 1)).unwrap();
    assert_eq!(book.best_bid(), Some((105, 30))); // New best
    
    // Remove best bid
    book.remove_order(3).unwrap();
    assert_eq!(book.best_bid(), Some((100, 10))); // Back to previous best
    
    // Add asks in ascending price order
    book.add_order(Order::new(4, Side::Sell, 10, 200, 1003, 1)).unwrap();
    assert_eq!(book.best_ask(), Some((200, 10)));
    
    book.add_order(Order::new(5, Side::Sell, 20, 205, 1004, 1)).unwrap();
    assert_eq!(book.best_ask(), Some((200, 10))); // Still best
    
    book.add_order(Order::new(6, Side::Sell, 30, 195, 1005, 1)).unwrap();
    assert_eq!(book.best_ask(), Some((195, 30))); // New best
}

#[test]
fn test_spread_and_mid_price() {
    // Test spread and mid price calculations
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    // Empty book
    assert_eq!(book.spread(), None);
    assert_eq!(book.mid_price(), None);
    
    // Only bid
    book.add_order(Order::new(1, Side::Buy, 10, 100, 1000, 1)).unwrap();
    assert_eq!(book.spread(), None);
    assert_eq!(book.mid_price(), None);
    
    // Only ask
    book.remove_order(1).unwrap();
    book.add_order(Order::new(2, Side::Sell, 10, 200, 1001, 1)).unwrap();
    assert_eq!(book.spread(), None);
    assert_eq!(book.mid_price(), None);
    
    // Both bid and ask
    book.add_order(Order::new(1, Side::Buy, 10, 100, 1002, 1)).unwrap();
    assert_eq!(book.spread(), Some(100));
    assert_eq!(book.mid_price(), Some(150));
    
    // Update prices
    book.remove_order(1).unwrap();
    book.add_order(Order::new(3, Side::Buy, 10, 150, 1003, 1)).unwrap();
    assert_eq!(book.spread(), Some(50));
    assert_eq!(book.mid_price(), Some(175));
}

#[test]
fn test_order_update_quantity() {
    // Test updating order quantities
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    let order = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
    book.add_order(order).unwrap();
    
    assert_eq!(book.volume_at_price(5000), Some(100));
    
    // Increase quantity
    book.update_order(1, 150).unwrap();
    assert_eq!(book.volume_at_price(5000), Some(150));
    
    // Decrease quantity
    book.update_order(1, 75).unwrap();
    assert_eq!(book.volume_at_price(5000), Some(75));
    
    // Update to 0 should fail (use remove instead)
    assert!(book.update_order(1, 0).is_err());
}

#[test]
fn test_process_order_add_update_remove() {
    // Test process_order for add, update, and remove operations
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    // Add new order
    let order1 = Order::new(1, Side::Buy, 100, 5000, 1000, 1);
    book.process_order(order1).unwrap();
    assert_eq!(book.total_orders(), 1);
    assert_eq!(book.volume_at_price(5000), Some(100));
    
    // Update existing order (same ID, different quantity)
    let order2 = Order::new(1, Side::Buy, 150, 5000, 1001, 1);
    book.process_order(order2).unwrap();
    assert_eq!(book.total_orders(), 1);
    assert_eq!(book.volume_at_price(5000), Some(150));
    
    // Remove order (quantity = 0)
    let order3 = Order::new(1, Side::Buy, 0, 5000, 1002, 1);
    book.process_order(order3).unwrap();
    assert_eq!(book.total_orders(), 0);
    assert_eq!(book.volume_at_price(5000), None);
}

#[test]
fn test_comprehensive_orderbook_operations() {
    // Comprehensive test combining multiple operations
    let mut book = OrderBook::new();
    book.set_time(1000);
    
    // Build a realistic orderbook
    let orders = vec![
        (Order::new(1, Side::Buy, 100, 4990, 1000, 1), true),
        (Order::new(2, Side::Buy, 200, 4980, 1001, 1), true),
        (Order::new(3, Side::Buy, 150, 4970, 1002, 1), true),
        (Order::new(4, Side::Sell, 100, 5010, 1003, 1), true),
        (Order::new(5, Side::Sell, 200, 5020, 1004, 1), true),
        (Order::new(6, Side::Sell, 150, 5030, 1005, 1), true),
    ];
    
    for (order, _) in orders {
        book.add_order(order).unwrap();
    }
    
    // Verify initial state
    assert_eq!(book.total_orders(), 6);
    assert_eq!(book.total_levels(), 6);
    assert_eq!(book.best_bid(), Some((4990, 100)));
    assert_eq!(book.best_ask(), Some((5010, 100)));
    assert_eq!(book.spread(), Some(20));
    
    // Update some orders
    book.update_order(1, 120).unwrap();
    assert_eq!(book.volume_at_price(4990), Some(120));
    
    // Remove some orders
    book.remove_order(2).unwrap();
    assert_eq!(book.total_orders(), 5);
    assert_eq!(book.total_levels(), 5);
    assert_eq!(book.best_bid(), Some((4990, 120)));
    
    // Remove best bid
    book.remove_order(1).unwrap();
    assert_eq!(book.best_bid(), Some((4970, 150)));
    
    // Remove best ask
    book.remove_order(4).unwrap();
    assert_eq!(book.best_ask(), Some((5020, 200)));
    
    // Verify final state
    assert_eq!(book.total_orders(), 3);
    assert_eq!(book.total_levels(), 3);
}
