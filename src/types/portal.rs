use super::{
    common::{
        Direction, InvId, LimitOrMarket, OrderId, Price, SeqNum, Size, SubId, TimeInForce,
        Timestamp,
    },
    event::Event,
    orderbook::{OrderDeadResponse, OrderFillResponse},
};
#[derive(Debug)]
pub enum PortalRequest {
    EventHistory(SubId),
    NewOrder(InvId, PortalNewOrderRequest),
    CancelOrder(InvId, OrderId),
}

#[derive(Debug)]
pub struct PortalNewOrderRequest {
    pub ticker: String,
    pub direction: Direction,
    pub size: Size,
    pub price: Price,
    pub limit_or_market: LimitOrMarket,
    pub time_in_force: TimeInForce,
    pub timestamp: Timestamp,
}

pub enum PortalTask {
    EventHistory(SubId, Vec<Event>),
    IncrementalEvent(Event),
    OrderAck(InvId, SeqNum, OrderId),    // ack new order request
    OrderReject(InvId, SeqNum, String),  // reject new order request
    CancelReject(InvId, SeqNum, String), // reject cancel order request
    OrderResponse(InvId, OrderResponse),
}

#[derive(Debug, PartialEq, Clone)]
pub enum OrderResponse {
    OrderFill(OrderFillResponse),
    OrderDead(OrderDeadResponse),
}
