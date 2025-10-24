// holds orderbook for all markets

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::models::domain::Order;

struct Orderbook {
    market_id: String,
    bids: BTreeMap<u128, VecDeque<Order>>,
    asks: BTreeMap<u128, VecDeque<Order>>,
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
