use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hft_orderbook::{OrderBook, Order, Side};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn benchmark_add_orders(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_orders");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("sequential", size), size, |b, &size| {
            b.iter(|| {
                let mut book = OrderBook::with_capacity(size, size / 10);
                book.set_time(1000);
                
                for i in 0..size {
                    let order = Order::new(
                        i as u64,
                        if i % 2 == 0 { Side::Buy } else { Side::Sell },
                        100,
                        5000 + (i % 100) as u64,
                        1000 + i as u64,
                        1,
                    );
                    black_box(book.add_order(order).unwrap());
                }
                black_box(book);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("random", size), size, |b, &size| {
            b.iter(|| {
                let mut book = OrderBook::with_capacity(size, size / 10);
                let mut rng = StdRng::seed_from_u64(42);
                book.set_time(1000);
                
                for i in 0..size {
                    let order = Order::new(
                        i as u64,
                        if rng.gen_bool(0.5) { Side::Buy } else { Side::Sell },
                        rng.gen_range(1..1000),
                        rng.gen_range(4900..5100),
                        1000 + i as u64,
                        1,
                    );
                    black_box(book.add_order(order).unwrap());
                }
                black_box(book);
            });
        });
    }
    group.finish();
}

fn benchmark_cancel_orders(c: &mut Criterion) {
    let mut group = c.benchmark_group("cancel_orders");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("cancel", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut book = OrderBook::with_capacity(size, size / 10);
                    let mut rng = StdRng::seed_from_u64(42);
                    book.set_time(1000);
                    
                    let mut order_ids = Vec::new();
                    for i in 0..size {
                        let order_id = i as u64;
                        let order = Order::new(
                            order_id,
                            if rng.gen_bool(0.5) { Side::Buy } else { Side::Sell },
                            rng.gen_range(1..1000),
                            rng.gen_range(4900..5100),
                            1000 + i as u64,
                            1,
                        );
                        book.add_order(order).unwrap();
                        order_ids.push(order_id);
                    }
                    (book, order_ids)
                },
                |(mut book, order_ids)| {
                    for &order_id in &order_ids {
                        if book.contains_order(order_id) {
                            black_box(book.cancel_order(order_id).unwrap());
                        }
                    }
                    black_box(book);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn benchmark_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("matching");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("cross_orders", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut book = OrderBook::with_capacity(size * 2, size / 5);
                    let mut rng = StdRng::seed_from_u64(42);
                    book.set_time(1000);
                    
                    // Add resting orders
                    for i in 0..size {
                        let order = Order::new(
                            i as u64,
                            Side::Sell,
                            rng.gen_range(1..1000),
                            5000 + (i % 100) as u64,
                            1000 + i as u64,
                            1,
                        );
                        book.add_order(order).unwrap();
                    }
                    book
                },
                |mut book| {
                    // Add crossing orders
                    let mut rng = StdRng::seed_from_u64(43);
                    for i in 0..size {
                        let order = Order::new(
                            (size + i) as u64,
                            Side::Buy,
                            rng.gen_range(1..1000),
                            5050, // Cross the spread
                            2000 + i as u64,
                            1,
                        );
                        black_box(book.add_order(order).unwrap());
                    }
                    black_box(book);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn benchmark_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("queries");
    
    // Setup a book with many orders
    let mut book = OrderBook::with_capacity(10000, 1000);
    let mut rng = StdRng::seed_from_u64(42);
    book.set_time(1000);
    
    for i in 0..10000 {
        let order = Order::new(
            i as u64,
            if rng.gen_bool(0.5) { Side::Buy } else { Side::Sell },
            rng.gen_range(1..1000),
            rng.gen_range(4900..5100),
            1000 + i as u64,
            1,
        );
        book.add_order(order).unwrap();
    }
    
    group.bench_function("best_bid", |b| {
        b.iter(|| black_box(book.best_bid()));
    });
    
    group.bench_function("best_ask", |b| {
        b.iter(|| black_box(book.best_ask()));
    });
    
    group.bench_function("spread", |b| {
        b.iter(|| black_box(book.spread()));
    });
    
    group.bench_function("mid_price", |b| {
        b.iter(|| black_box(book.mid_price()));
    });
    
    group.bench_function("volume_at_price", |b| {
        b.iter(|| black_box(book.volume_at_price(5000)));
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_add_orders,
    benchmark_cancel_orders,
    benchmark_matching,
    benchmark_queries
);
criterion_main!(benches);
