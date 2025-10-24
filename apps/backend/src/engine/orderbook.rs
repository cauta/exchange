// holds orderbook for all markets

use std::collections::BTreeMap;
use std::collections::HashMap;

struct Orderbook {
    market_id: String,
    bids: BTreeMap<u128, u128>,
    asks: BTreeMap<u128, u128>,
}

pub struct Orderbooks {
    orderbooks: HashMap<String, Orderbook>,
}

impl Orderbooks {
    pub fn new() -> Self {
        Self {
            orderbooks: HashMap::new(),
        }
    }
}
