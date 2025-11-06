//! Simple tests for the new orderbook design

use crate::{OrderBook, MatchingEngine, Order, Side, OrderBookError};

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
fn test_matching_engine() {
    let mut book = OrderBook::new();
    let engine = MatchingEngine::new();
    book.set_time(1000);

    // Add a resting sell order
    let sell_order = Order::new(1, Side::Sell, 100, 5000, 1000, 1);
    book.add_order(sell_order).unwrap();

    // Add a crossing buy order using matching engine
    let buy_order = Order::new(2, Side::Buy, 50, 5000, 1001, 1);
    let trades = engine.process_order(&mut book, buy_order).unwrap();
    
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 50);
    assert_eq!(trades[0].price, 5000);
    assert_eq!(trades[0].aggressor_order_id, 2);
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
