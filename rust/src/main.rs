use hft_orderbook::{OrderBook, MatchingEngine, Order, Side};

fn main() {
    println!("HFT OrderBook Demo - Clean Implementation");
    println!("=========================================");

    // Create a new orderbook and matching engine
    let mut book = OrderBook::new();
    let engine = MatchingEngine::new();
    book.set_time(1000);

    println!("Created empty orderbook and matching engine");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());

    // Add some orders using pure orderbook operations
    let buy_order1 = Order::new(1, Side::Buy, 100, 4950, 1000, 1);
    let buy_order2 = Order::new(2, Side::Buy, 200, 4940, 1001, 1);
    let sell_order1 = Order::new(3, Side::Sell, 150, 5050, 1002, 1);
    let sell_order2 = Order::new(4, Side::Sell, 100, 5060, 1003, 1);

    println!("\n1. Adding orders to orderbook (no matching)...");

    match book.add_order(buy_order1) {
        Ok(()) => println!("Added buy order 1"),
        Err(e) => println!("Error adding buy order 1: {}", e),
    }

    match book.add_order(buy_order2) {
        Ok(()) => println!("Added buy order 2"),
        Err(e) => println!("Error adding buy order 2: {}", e),
    }

    match book.add_order(sell_order1) {
        Ok(()) => println!("Added sell order 1"),
        Err(e) => println!("Error adding sell order 1: {}", e),
    }

    match book.add_order(sell_order2) {
        Ok(()) => println!("Added sell order 2"),
        Err(e) => println!("Error adding sell order 2: {}", e),
    }

    println!("\nOrderbook state after adding orders:");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());
    println!("Spread: {:?}", book.spread());
    println!("Mid price: {:?}", book.mid_price());
    println!("Total orders: {}", book.total_orders());
    println!("Total levels: {}", book.total_levels());

    // Show price levels
    let (bids, asks) = book.get_levels(Some(5));
    println!("Bids: {:?}", bids);
    println!("Asks: {:?}", asks);

    // Use matching engine for crossing order
    println!("\n2. Using matching engine for crossing order...");
    let crossing_order = Order::new(5, Side::Buy, 75, 5055, 1004, 1);

    match engine.process_order(&mut book, crossing_order) {
        Ok(trades) => {
            println!("Processed crossing order, generated {} trades:", trades.len());
            for trade in trades {
                println!("  {}", trade);
            }
        }
        Err(e) => println!("Error processing crossing order: {}", e),
    }

    println!("\n3. Pure orderbook operations...");

    // Update an order
    match book.update_order(1, 150) {
        Ok(()) => println!("Updated order 1 quantity to 150"),
        Err(e) => println!("Error updating order: {}", e),
    }

    // Remove an order
    match book.remove_order(2) {
        Ok(cancelled) => println!("Removed order: {}", cancelled),
        Err(e) => println!("Error removing order: {}", e),
    }

    println!("\nFinal orderbook state:");
    println!("Best bid: {:?}", book.best_bid());
    println!("Best ask: {:?}", book.best_ask());
    println!("Spread: {:?}", book.spread());
    println!("Total orders: {}", book.total_orders());
    println!("Total levels: {}", book.total_levels());

    // Final price levels
    let (bids, asks) = book.get_levels(None);
    println!("Final bids: {:?}", bids);
    println!("Final asks: {:?}", asks);
}
