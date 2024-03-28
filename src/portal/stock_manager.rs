// StockManager: store all static information of stocks: e.g. close price, lot size, mpf, etc.

use crate::types::common::{Price, Size, StockName, Ticker};
use std::collections::HashMap;

#[derive(Debug)]
pub struct StockRecord {
    pub close_price: Price,
    pub lot_size: Size,
    pub mpf: Price,
    pub name: StockName,
}

pub struct StockManager {
    pub bind: HashMap<Ticker, StockRecord>,
}

impl StockManager {
    // Initialize an empty stock manager
    pub fn new() -> Self {
        StockManager {
            bind: HashMap::new(),
        }
    }
    // Used for initialization
    pub fn bind_stock(&mut self, ticker: Ticker, stock_rec: StockRecord) {
        // println!("bind stock: {:?}", stock_rec); // for debug
        self.bind.insert(ticker, stock_rec);
    }

    fn check_valid_price(p: &Price, mpf: &Price) -> bool {
        let ratio = p / mpf;
        let epsilon = std::f32::EPSILON;
        (ratio - ratio.floor()).abs() < epsilon
    }
    fn check_valid_size(size: &Size, lot_size: &Size) -> bool {
        size % lot_size == 0 && size > &0
    }

    pub fn check_valid_order(&self, ticker: &Ticker, price: &Price, size: &Size) -> bool {
        self.bind.get(ticker).map_or(false, |stock_rec| {
            Self::check_valid_price(price, &stock_rec.mpf)
                && Self::check_valid_size(size, &stock_rec.lot_size)
        })
    }

    pub fn get_close_price(&self, ticker: &Ticker) -> Option<Price> {
        self.bind.get(ticker).map(|stock_rec| stock_rec.close_price)
    }
}
