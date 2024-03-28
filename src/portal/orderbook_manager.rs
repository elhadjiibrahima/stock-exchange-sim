// OrderbookManager: stores and manages all orderbooks by ticker

use super::orderbook::OrderBook;
use crate::types::{
    common::{Price, Ticker},
    orderbook::{OrderbookLog, OrderbookRequest},
};
use std::collections::HashMap;

pub struct OrderbookManager {
    pub bind: HashMap<Ticker, OrderBook>,
}

impl OrderbookManager {
    pub fn new() -> Self {
        OrderbookManager {
            bind: HashMap::new(),
        }
    }
    // Initialize an orderbook for a ticker
    pub fn add_orderbook(&mut self, ticker: Ticker) {
        self.bind.insert(ticker.clone(), OrderBook::new(ticker));
    }

    // Handle orderbook request, return orderbook logs
    pub fn handle_orderbook_request(
        &mut self,
        ticker: Ticker,
        req: OrderbookRequest,
    ) -> Vec<OrderbookLog> {
        self.bind
            .get_mut(&ticker)
            .map_or(vec![], |orderbook| orderbook.handle_request(req))
    }

    // Get best buy price of orderbook
    pub fn best_buy_price(&mut self, ticker: &Ticker) -> Option<Price> {
        self.bind
            .get_mut(ticker)
            .map_or(None, |orderbook| orderbook.best_buy_price())
    }

    // Get best sell price of orderbook
    pub fn best_sell_price(&mut self, ticker: &Ticker) -> Option<Price> {
        self.bind
            .get_mut(ticker)
            .map_or(None, |orderbook| orderbook.best_sell_price())
    }
}
