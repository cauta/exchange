// holds orderbook
// executes trades

use crate::db::Db;

pub struct Executor {
    db: Db,
}

impl Executor {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}
