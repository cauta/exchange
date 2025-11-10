use crate::db::TestDb;
use crate::helpers;
use backend::db::Db;
use backend::engine::MatchingEngine;
use backend::models::domain::{EngineEvent, EngineRequest, Order, OrderStatus, OrderType, Side};
use chrono::Utc;
use tokio::sync::{broadcast, mpsc, oneshot};
use uuid::Uuid;

/// Helper for matching engine testing
///
/// This exposes internal engine channels and domain types
/// for backend tests that need to test matching logic directly.
#[allow(dead_code)]
pub struct TestEngine {
    pub db: Db,
    pub engine_tx: mpsc::Sender<EngineRequest>,
    pub event_rx: broadcast::Receiver<EngineEvent>,
    event_tx: broadcast::Sender<EngineEvent>,
}

#[allow(dead_code)]
impl TestEngine {
    pub async fn new(test_db: &TestDb) -> Self {
        Self::new_with_users(test_db, true).await
    }

    /// Create a new TestEngine, optionally creating common test users
    pub async fn new_with_users(test_db: &TestDb, create_users: bool) -> Self {
        // Create common test users for engine tests only
        if create_users {
            let users = vec![
                "buyer",
                "seller",
                "seller1",
                "seller2",
                "seller3",
                "buyer1",
                "buyer2",
                "buyer3",
                "alice",
                "bob",
                "charlie",
                "dave",
                "user1",
                "user2",
                "attacker",
                "big_buyer",
            ];

            for user in &users {
                let _ = helpers::create_user(test_db, user).await;
            }

            // Give each user generous balances for various common test tokens
            // Using realistic decimals: 8 for base tokens, 6 for quote tokens
            let test_tokens = vec![
                ("BTC", 8, 1_000_000_000u128),     // 10 BTC
                ("ETH", 8, 10_000_000_000),        // 100 ETH
                ("SOL", 8, 100_000_000_000),       // 1,000 SOL
                ("LINK", 8, 1_000_000_000_000),    // 10,000 LINK
                ("ADA", 8, 10_000_000_000_000),    // 100,000 ADA
                ("MATIC", 8, 100_000_000_000_000), // 1,000,000 MATIC
                ("ATOM", 8, 10_000_000_000_000),   // 100,000 ATOM
                ("AVAX", 8, 10_000_000_000_000),   // 100,000 AVAX
                ("DOT", 8, 10_000_000_000_000),    // 100,000 DOT
                ("UNI", 8, 10_000_000_000_000),    // 100,000 UNI
                ("USDC", 6, 10_000_000_000_000),   // 10,000,000 USDC
                ("USDT", 6, 10_000_000_000_000),   // 10,000,000 USDT
                ("DAI", 6, 10_000_000_000_000),    // 10,000,000 DAI
            ];

            for user in &users {
                for (ticker, _decimals, amount) in &test_tokens {
                    // Create token if it doesn't exist (ignore errors if it already exists)
                    if test_db.db.get_token(ticker).await.is_err() {
                        let _ = test_db
                            .db
                            .create_token(
                                ticker.to_string(),
                                *_decimals,
                                format!("{} Token", ticker),
                            )
                            .await;
                    }

                    // Add balance to user
                    let _ = test_db.db.add_balance(user, ticker, *amount).await;
                }
            }
        }

        let (engine_tx, engine_rx) = mpsc::channel::<EngineRequest>(100);
        let (event_tx, event_rx) = broadcast::channel::<EngineEvent>(1000);

        let engine = MatchingEngine::new(test_db.db.clone(), engine_rx, event_tx.clone());

        // Spawn engine in background
        tokio::spawn(async move {
            engine.run().await;
        });

        Self {
            db: test_db.db.clone(),
            engine_tx,
            event_rx,
            event_tx,
        }
    }

    /// Get a clone of the event sender for HTTP server state
    pub fn event_tx(&self) -> broadcast::Sender<EngineEvent> {
        self.event_tx.clone()
    }

    /// Helper to place an order and get the response
    pub async fn place_order(
        &self,
        order: Order,
    ) -> Result<backend::models::api::OrderPlaced, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.engine_tx
            .send(EngineRequest::PlaceOrder { order, response_tx })
            .await
            .map_err(|e| format!("Failed to send order: {}", e))?;

        response_rx
            .await
            .map_err(|e| format!("Failed to receive response: {}", e))?
            .map_err(|e| format!("Order placement failed: {}", e))
    }

    /// Helper to cancel an order
    pub async fn cancel_order(
        &self,
        order_id: Uuid,
        user_address: String,
    ) -> Result<backend::models::api::OrderCancelled, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.engine_tx
            .send(EngineRequest::CancelOrder {
                order_id,
                user_address,
                response_tx,
            })
            .await
            .map_err(|e| format!("Failed to send cancel request: {}", e))?;

        response_rx
            .await
            .map_err(|e| format!("Failed to receive response: {}", e))?
            .map_err(|e| format!("Order cancellation failed: {}", e))
    }

    /// Helper to create a test order
    pub fn create_order(
        user_address: &str,
        market_id: &str,
        side: Side,
        order_type: OrderType,
        price: u128,
        size: u128,
    ) -> Order {
        Order {
            id: Uuid::new_v4(),
            user_address: user_address.to_string(),
            market_id: market_id.to_string(),
            price,
            size,
            side,
            order_type,
            status: OrderStatus::Pending,
            filled_size: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
