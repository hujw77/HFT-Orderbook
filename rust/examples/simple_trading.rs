use hft_orderbook::{OrderBook, Order, Side};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn main() {
    println!("Simple OrderBook Simulation");
    println!("===========================");

    let mut book = OrderBook::with_capacity(1000, 100);
    let mut rng = StdRng::seed_from_u64(42);
    let mut order_id = 1u64;
    let mut timestamp = 1000u64;

    book.set_time(timestamp);

    // Phase 1: Build initial book with fewer orders
    println!("\nPhase 1: Building initial order book...");
    
    // Add buy orders (bids) from 4990 to 4999
    for price in 4990..5000 {
        let quantity = rng.gen_range(100..500);
        let order = Order::new(order_id, Side::Buy, quantity, price, timestamp, 1);
        book.add_order(order).unwrap();
        order_id += 1;
        timestamp += 1;
    }

    // Add sell orders (asks) from 5001 to 5010
    for price in 5001..5011 {
        let quantity = rng.gen_range(100..500);
        let order = Order::new(order_id, Side::Sell, quantity, price, timestamp, 1);
        book.add_order(order).unwrap();
        order_id += 1;
        timestamp += 1;
    }

    print_market_state(&book, "Initial Market State");

    // Phase 2: Add more orders
    println!("\nPhase 2: Adding more orders...");
    
    println!("Adding large buy order (2000 @ 4995)...");
    let large_buy_order = Order::new(order_id, Side::Buy, 2000, 4995, timestamp, 1);
    book.add_order(large_buy_order).unwrap();
    order_id += 1;
    timestamp += 1;

    print_market_state(&book, "After Adding Large Buy Order");

    // Phase 3: Add sell orders
    println!("\nPhase 3: Adding sell orders...");
    
    println!("Adding large sell order (1500 @ 5005)...");
    let large_sell_order = Order::new(order_id, Side::Sell, 1500, 5005, timestamp, 1);
    book.add_order(large_sell_order).unwrap();

    print_market_state(&book, "Final Market State");

    // Final statistics
    println!("\nFinal Statistics:");
    println!("Total orders in book: {}", book.total_orders());
    println!("Total price levels: {}", book.total_levels());

    // Calculate total volumes from price levels
    let (bids, asks) = book.get_levels(None);
    let total_buy_volume: u64 = bids.iter().map(|(_, qty)| *qty).sum();
    let total_sell_volume: u64 = asks.iter().map(|(_, qty)| *qty).sum();
    println!("Total buy volume: {}", total_buy_volume);
    println!("Total sell volume: {}", total_sell_volume);
}

fn print_market_state(book: &OrderBook, title: &str) {
    println!("\n{}", title);
    println!("{}", "=".repeat(title.len()));
    println!("Best Bid: {:?}", book.best_bid());
    println!("Best Ask: {:?}", book.best_ask());
    println!("Spread: {:?}", book.spread());
    println!("Mid Price: {:?}", book.mid_price());
    println!("Orders: {}, Levels: {}", book.total_orders(), book.total_levels());
}
