use crate::types::common::{Direction, OrderId, Price, Size, Ticker};

#[derive(Debug, PartialEq, Clone)]
pub enum Event {
    OrderAdded(OrderAdded),
    OrderExecuted(OrderExecuted),
    OrderRemoved(OrderRemoved),
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderAdded {
    pub order_id: OrderId,
    pub ticker: Ticker,
    pub direction: Direction,
    pub resting_size: Size,
    pub limit_price: Price,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderExecuted {
    pub order_id: OrderId,
    pub ticker: Ticker,
    pub execution_size: Size,
    pub execution_price: Price,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderRemoved {
    pub order_id: OrderId,
}
