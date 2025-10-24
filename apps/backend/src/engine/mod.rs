// process
// price time priority

pub mod executor;
pub mod matcher;
pub mod orderbook;

use crate::db::Db;
use crate::models::domain::{EngineRequest, EngineResponse};
use executor::Executor;
use matcher::Matcher;
use orderbook::Orderbooks;

use tokio::sync::{broadcast, mpsc};

pub struct MatchingEngine {
    db: Db,
    orderbooks: Orderbooks,
    matcher: Matcher,
    executor: Executor,

    engine_rx: mpsc::Receiver<EngineRequest>,
    response_tx: broadcast::Sender<EngineResponse>,
}

impl MatchingEngine {
    pub fn new(
        db: Db,
        engine_rx: mpsc::Receiver<EngineRequest>,
        response_tx: broadcast::Sender<EngineResponse>,
    ) -> Self {
        Self {
            db: db.clone(),
            orderbooks: Orderbooks::new(),
            matcher: Matcher::new(),
            executor: Executor::new(db),
            engine_rx,
            response_tx,
        }
    }

    pub async fn run(self) {
        todo!()
    }
}

// process order
// use orderbook, get orderbook
// use matcher, get matches -> returns execution requests
// use executor, execute matches
// update orderbook?
