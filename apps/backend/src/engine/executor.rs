// holds orderbook
// executes trades

use crate::db::Db;
use crate::engine::orderbook::Orderbooks;

pub struct Executor {
    db: Db,
    orderbooks: Orderbooks,
}

impl Executor {
    pub fn new(db: Db) -> Self {
        Self {
            db,
            orderbooks: Orderbooks::new(),
        }
    }
}
