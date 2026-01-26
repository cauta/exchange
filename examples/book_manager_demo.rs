//! Example demonstrating BookManagerAdapter with BookManagerStd integration
//!
//! This example shows how to:
//! 1. Create markets and add orders
//! 2. Use the BookManagerAdapter with integrated BookManagerStd
//! 3. Access both the adapter and BookManagerStd functionality
//! 4. Start the trade processor for centralized trade event handling

use exchange::engine::adapter::BookManagerAdapter;
use exchange::models::domain::{Market, Order, OrderType, Side, OrderStatus};
use uuid::Uuid;
use chrono::Utc;
use std::thread;
use std::time::Duration;

fn create_test_market(id: &str, tick_size: u128, lot_size: u128) -> Market {
    Market {
        id: id.to_string(),
        base_ticker: id.split('/').next().unwrap().to_string(),
        quote_ticker: id.split('/').nth(1).unwrap().to_string(),
        tick_size,
        lot_size,
        min_size: lot_size,
        maker_fee_bps: 5,
        taker_fee_bps: 10,
    }
}

fn create_test_order(market_id: &str, user: &str, side: Side, price: u128, size: u128) -> Order {
    Order {
        id: Uuid::new_v4(),
        user_address: user.to_string(),
        market_id: market_id.to_string(),
        price,
        size,
        side,
        order_type: OrderType::Limit,
        status: OrderStatus::Pending,
        filled_size: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BookManagerAdapter with BookManagerStd Integration Demo");
    
    // Create the BookManagerAdapter
    let manager = BookManagerAdapter::new();
    
    // Create test markets
    let btc_market = create_test_market("BTC/USDC", 1_000_000, 10_000); // 0.01 tick, 0.0001 lot
    let eth_market = create_test_market("ETH/USDC", 100_000, 100_000);  // 0.001 tick, 0.001 lot
    
    println!("\n📊 Adding orders to markets...");
    
    // Add orders to BTC/USDC
    let btc_buy = create_test_order("BTC/USDC", "alice", Side::Buy, 50_000_000_000, 100_000_000); // $50,000, 1 BTC
    let btc_sell = create_test_order("BTC/USDC", "bob", Side::Sell, 51_000_000_000, 50_000_000);  // $51,000, 0.5 BTC
    
    manager.add_order(&btc_market, &btc_buy);
    manager.add_order(&btc_market, &btc_sell);
    
    // Add orders to ETH/USDC
    let eth_buy = create_test_order("ETH/USDC", "charlie", Side::Buy, 3_000_000_000, 1_000_000_000); // $3,000, 1 ETH
    let eth_sell = create_test_order("ETH/USDC", "diana", Side::Sell, 3_100_000_000, 500_000_000);  // $3,100, 0.5 ETH
    
    manager.add_order(&eth_market, &eth_buy);
    manager.add_order(&eth_market, &eth_sell);
    
    println!("✅ Orders added successfully!");
    
    // Display adapter statistics
    println!("\n📈 Adapter Statistics:");
    println!("  - Managed markets: {}", manager.len());
    println!("  - Is empty: {}", manager.is_empty());
    
    // Display BookManagerStd statistics
    println!("\n🔧 BookManagerStd Statistics:");
    println!("  - Managed symbols: {:?}", manager.managed_symbols());
    println!("  - Book count: {}", manager.managed_book_count());
    
    // Generate snapshots
    println!("\n📸 Orderbook Snapshots:");
    let snapshots = manager.snapshots();
    for snapshot in snapshots {
        println!("  Market: {}", snapshot.market_id);
        println!("    Bids: {}", snapshot.bids.len());
        println!("    Asks: {}", snapshot.asks.len());
        if !snapshot.bids.is_empty() {
            println!("    Best Bid: {} @ {}", snapshot.bids[0].size, snapshot.bids[0].price);
        }
        if !snapshot.asks.is_empty() {
            println!("    Best Ask: {} @ {}", snapshot.asks[0].size, snapshot.asks[0].price);
        }
    }
    
    // Start the trade processor (this runs in background)
    println!("\n🔄 Starting trade processor...");
    let _processor_handle = manager.start_trade_processor();
    
    // Access the underlying BookManagerStd directly
    println!("\n🔍 Direct BookManagerStd Access:");
    {
        let manager_guard = manager.manager();
        println!("  - Has BTC/USDC book: {}", manager_guard.has_book("BTC/USDC"));
        println!("  - Has ETH/USDC book: {}", manager_guard.has_book("ETH/USDC"));
        
        // Access individual books
        if let Some(btc_book) = manager_guard.get_book("BTC/USDC") {
            println!("  - BTC Book best bid: {:?}", btc_book.best_bid());
            println!("  - BTC Book best ask: {:?}", btc_book.best_ask());
            if let (Some(bid), Some(ask)) = (btc_book.best_bid(), btc_book.best_ask()) {
                let spread = ask.saturating_sub(bid);
                println!("  - BTC Book spread: {} ticks", spread);
            }
        }
    }
    
    // Demonstrate order cancellation
    println!("\n❌ Cancelling an order...");
    let cancelled = manager.cancel_order(btc_buy.id, "alice")?;
    println!("  - Cancelled order: {} (size: {})", cancelled.id, cancelled.size);
    
    // Show updated statistics
    println!("\n📊 Updated Statistics:");
    let snapshots = manager.snapshots();
    for snapshot in snapshots {
        println!("  Market: {} - Bids: {}, Asks: {}", 
                snapshot.market_id, snapshot.bids.len(), snapshot.asks.len());
    }
    
    println!("\n✨ Demo completed successfully!");
    
    // In a real application, you'd want to keep the processor handle
    // and shut it down gracefully. For this demo, we'll just let it finish.
    thread::sleep(Duration::from_millis(100));
    
    Ok(())
}
