use hft_orderbook::{OrderBook, MatchingEngine, Order, Side};

fn main() {
    println!("Clean OrderBook Demo");
    println!("===================");

    // Create orderbook and matching engine
    let mut book = OrderBook::new();
    let engine = MatchingEngine::new();
    book.set_time(1000);

    println!("Created empty orderbook");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());

    // Add orders using pure orderbook operations
    println!("\n1. Adding orders to orderbook (no matching):");
    
    let orders = vec![
        Order::new(1, Side::Buy, 100, 4950, 1000, 1),
        Order::new(2, Side::Buy, 200, 4940, 1001, 1),
        Order::new(3, Side::Sell, 150, 5050, 1002, 1),
        Order::new(4, Side::Sell, 100, 5060, 1003, 1),
    ];

    for order in orders {
        let order_id = order.id;
        match book.add_order(order) {
            Ok(()) => println!("Added order {}", order_id),
            Err(e) => println!("Error adding order: {}", e),
        }
    }

    println!("\nOrderbook state:");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());
    println!("Spread: {:?}", book.spread());
    println!("Total orders: {}", book.total_orders());

    // Show price levels
    let (bids, asks) = book.get_levels(Some(5));
    println!("Bids: {:?}", bids);
    println!("Asks: {:?}", asks);

    // Use matching engine for crossing order
    println!("\n2. Using matching engine for crossing order:");
    let crossing_order = Order::new(5, Side::Buy, 75, 5055, 1004, 1);

    match engine.process_order(&mut book, crossing_order) {
        Ok(trades) => {
            println!("Generated {} trades:", trades.len());
            for trade in trades {
                println!("  {}", trade);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Pure orderbook operations
    println!("\n3. Pure orderbook operations:");
    
    // Update order
    match book.update_order(1, 150) {
        Ok(()) => println!("Updated order 1 to quantity 150"),
        Err(e) => println!("Error updating order: {}", e),
    }

    // Remove order
    match book.remove_order(2) {
        Ok(removed) => println!("Removed order: {}", removed),
        Err(e) => println!("Error removing order: {}", e),
    }

    // Process order (Python-style)
    let new_order = Order::new(6, Side::Sell, 50, 5040, 1005, 1);
    match book.process_order(new_order) {
        Ok(()) => println!("Processed new order using process_order"),
        Err(e) => println!("Error processing order: {}", e),
    }

    println!("\nFinal state:");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());
    println!("Total orders: {}", book.total_orders());
    println!("Total levels: {}", book.total_levels());

    // Final price levels
    let (bids, asks) = book.get_levels(None);
    println!("Final bids: {:?}", bids);
    println!("Final asks: {:?}", asks);

    // Volume queries
    println!("\n4. Volume queries:");
    println!("Volume at 4950: {:?}", book.volume_at_price(4950));
    println!("Volume at 5040: {:?}", book.volume_at_price(5040));
    println!("Orders at 4950: {:?}", book.orders_at_price(4950));
    println!("Orders at 5040: {:?}", book.orders_at_price(5040));
}
