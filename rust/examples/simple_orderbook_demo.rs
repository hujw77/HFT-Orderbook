use hft_orderbook::{OrderBook, MatchingEngine, Order, Side};

fn main() {
    println!("Simple OrderBook Demo - Pure Data Structure");
    println!("===========================================");

    // Create orderbook (pure data structure) and matching engine (separate)
    let mut book = OrderBook::new();
    let engine = MatchingEngine::new();
    book.set_time(1000);

    println!("Created empty orderbook and matching engine");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());

    // Add orders using pure orderbook operations (no matching)
    println!("\n1. Adding orders directly to orderbook (no matching):");
    
    let buy_order1 = Order::new(1, Side::Buy, 100, 4950, 1000, 1);
    let buy_order2 = Order::new(2, Side::Buy, 200, 4940, 1001, 1);
    let sell_order1 = Order::new(3, Side::Sell, 150, 5050, 1002, 1);
    let sell_order2 = Order::new(4, Side::Sell, 100, 5060, 1003, 1);

    book.add_order(buy_order1).unwrap();
    book.add_order(buy_order2).unwrap();
    book.add_order(sell_order1).unwrap();
    book.add_order(sell_order2).unwrap();

    println!("Added 4 orders directly to book");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());
    println!("Spread: {:?}", book.spread());
    println!("Total orders: {}", book.total_orders());

    // Show price levels
    let (bids, asks) = book.get_levels(Some(5));
    println!("\nPrice levels:");
    println!("Bids: {:?}", bids);
    println!("Asks: {:?}", asks);

    // Use matching engine for order processing
    println!("\n2. Using matching engine for order processing:");
    
    let crossing_order = Order::new(5, Side::Buy, 75, 5055, 1004, 1);
    
    match engine.process_order(&mut book, crossing_order) {
        Ok(trades) => {
            println!("Processed crossing order, generated {} trades:", trades.len());
            for trade in trades {
                println!("  {}", trade);
            }
        }
        Err(e) => println!("Error processing order: {}", e),
    }

    println!("\nAfter matching:");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());
    println!("Total orders: {}", book.total_orders());

    // Demonstrate pure orderbook operations
    println!("\n3. Pure orderbook operations:");

    // Process order (Python-style)
    let update_order = Order::new(6, Side::Sell, 50, 5040, 1005, 1);
    book.process_order(update_order).unwrap();
    println!("Processed new order using process_order method");

    // Update existing order
    book.update_order(1, 150).unwrap();
    println!("Updated order 1 quantity to 150");

    // Remove order
    let removed = book.remove_order(2).unwrap();
    println!("Removed order: {}", removed);

    println!("\nFinal state:");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());
    println!("Total orders: {}", book.total_orders());
    println!("Total levels: {}", book.total_levels());

    // Show final price levels
    let (bids, asks) = book.get_levels(None);
    println!("\nFinal price levels:");
    println!("Bids: {:?}", bids);
    println!("Asks: {:?}", asks);

    // Demonstrate volume queries
    println!("\n4. Volume queries:");
    println!("Volume at 4950: {:?}", book.volume_at_price(4950));
    println!("Volume at 5040: {:?}", book.volume_at_price(5040));
    println!("Orders at 4950: {:?}", book.orders_at_price(4950));
    println!("Orders at 5040: {:?}", book.orders_at_price(5040));
}
