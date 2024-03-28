use super::{
    common::{Direction, LimitOrMarket, OrderId, Price, Size, TimeInForce, Timestamp},
    event::Event,
    portal::OrderResponse,
};

// Request types for Orderbook API
pub enum OrderbookRequest {
    NewOrder(NewOrderRequest),
    CancelOrder(CancelOrderRequest),
}
pub struct NewOrderRequest {
    pub order_id: OrderId,
    pub direction: Direction,
    pub size: Size,
    pub price: Price,
    pub timestamp: Timestamp,
    pub limit_or_market: LimitOrMarket,
    pub time_in_force: TimeInForce,
}
pub struct CancelOrderRequest {
    pub order_id: OrderId,
}

#[derive(Debug, PartialEq)]
pub enum OrderbookLog {
    OrderLog(OrderResponse),
    EventLog(Event),
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderFillResponse {
    pub order_id: OrderId,
    pub fill_size: Size,
    pub fill_price: Price,
}
#[derive(Debug, PartialEq, Clone)]
pub struct OrderDeadResponse {
    pub order_id: OrderId,
}
