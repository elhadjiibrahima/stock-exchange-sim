use serde::Deserialize;

pub type OrderId = u64;
pub type Size = u32;
pub type Ticker = String;
pub type Timestamp = u64;
pub type Price = f32;

pub type Cash = f32;
pub type InvId = u64;
pub type AccountName = String;
pub type Password = String;

pub type StockName = String;
pub type SeqNum = u64;
pub type SubId = u64;

#[derive(Debug, PartialEq, Deserialize)]
pub enum LimitOrMarket {
    Limit,
    Market,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum TimeInForce {
    Day,
    IOC,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub enum Direction {
    Buy,
    Sell,
}
