// utils: contains helper functions for parsing and wrapping rpc proto types

use crate::server::stock_exchange::{
    rpc_order_request::{self, CancelOrder, NewOrder},
    rpc_order_response::{CancelRej, OrderAck, OrderDead, OrderFill, OrderRej, Response},
    rpc_subscribe_response, RpcOrderRequest, RpcOrderResponse, RpcSubscribeResponse,
};
use crate::types::{
    account_manager::AccountUpdate,
    common::{Direction, InvId, LimitOrMarket, OrderId, SeqNum, SubId, TimeInForce},
    event::Event,
    orderbook::{OrderDeadResponse, OrderFillResponse},
    portal::{OrderResponse, PortalNewOrderRequest, PortalRequest},
};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_inv_id(acc_upd: &AccountUpdate) -> InvId {
    match acc_upd {
        AccountUpdate::UpdCash(inv_id, _) => inv_id.clone(),
        AccountUpdate::AddPos(inv_id, ..) => inv_id.clone(),
        AccountUpdate::MinusPos(inv_id, ..) => inv_id.clone(),
    }
}

pub fn get_order_id(order_resp: &OrderResponse) -> OrderId {
    match order_resp {
        OrderResponse::OrderFill(order_fill) => order_fill.order_id,
        OrderResponse::OrderDead(order_dead) => order_dead.order_id,
    }
}

fn get_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs()
}

// Parse: rpc proto type -> portal type
// Wrap: portal type -> rpc proto type
fn parse_direction(value: i32) -> Direction {
    match value {
        0 => Direction::Buy,
        1 => Direction::Sell,
        _ => panic!("invalid direction"),
    }
}
fn wrap_direction(direction: Direction) -> i32 {
    match direction {
        Direction::Buy => 0,
        Direction::Sell => 1,
    }
}
fn parse_limit_or_market(value: i32) -> LimitOrMarket {
    match value {
        0 => LimitOrMarket::Limit,
        1 => LimitOrMarket::Market,
        _ => panic!("invalid limit or market"),
    }
}
fn parse_time_in_force(value: i32) -> TimeInForce {
    match value {
        0 => TimeInForce::Day,
        1 => TimeInForce::IOC,
        _ => panic!("invalid time in force"),
    }
}

// Send Order rpc

// parse rpc new order request to portal request
fn parse_new_order_request(inv_id: InvId, new_order: NewOrder) -> PortalRequest {
    let req = PortalNewOrderRequest {
        ticker: new_order.ticker,
        direction: parse_direction(new_order.direction),
        size: new_order.size,
        price: new_order.price,
        limit_or_market: parse_limit_or_market(new_order.limit_or_market),
        time_in_force: parse_time_in_force(new_order.time_in_force),
        timestamp: get_timestamp(),
    };
    PortalRequest::NewOrder(inv_id, req)
}

// parse rpc cancel order request to portal request
fn parse_cancel_order_request(inv_id: InvId, cancel_order: CancelOrder) -> PortalRequest {
    PortalRequest::CancelOrder(inv_id, cancel_order.order_id)
}

// parse RpcOrderRequest to PortalRequest
pub fn parse_order_request(inv_id: InvId, request: RpcOrderRequest) -> PortalRequest {
    let request: rpc_order_request::Request = request.request.unwrap();
    match request {
        rpc_order_request::Request::NewOrder(new_order) => {
            parse_new_order_request(inv_id, new_order)
        }
        rpc_order_request::Request::CancelOrder(cancel_order) => {
            parse_cancel_order_request(inv_id, cancel_order)
        }
        _ => panic!("parse_order_request: invalid request"),
    }
}

// parse seqnum from rpc request
pub fn parse_seqnum(request: &RpcOrderRequest) -> SeqNum {
    match &request.request {
        Some(rpc_order_request::Request::Login(login)) => login.seqnum.clone(),
        Some(rpc_order_request::Request::NewOrder(new_order)) => new_order.seqnum.clone(),
        Some(rpc_order_request::Request::CancelOrder(cancel_order)) => cancel_order.seqnum.clone(),
        _ => panic!("invalid request"),
    }
}

// Wrap OrderResponse to RpcOrderResponse
pub fn wrap_order_response(response: OrderResponse) -> RpcOrderResponse {
    match response {
        OrderResponse::OrderFill(order_fill) => wrap_order_fill_response(order_fill),
        OrderResponse::OrderDead(order_dead) => wrap_order_dead_response(order_dead),
    }
}

pub fn wrap_order_reject(seqnum: u64, reason: String) -> RpcOrderResponse {
    RpcOrderResponse {
        response: Some(Response::Rej(OrderRej { seqnum, reason })),
    }
}

pub fn wrap_cancel_reject(seqnum: u64, reason: String) -> RpcOrderResponse {
    RpcOrderResponse {
        response: Some(Response::CancelRej(CancelRej { seqnum, reason })),
    }
}

pub fn wrap_order_ack(seqnum: SeqNum, order_id: OrderId) -> RpcOrderResponse {
    RpcOrderResponse {
        response: Some(Response::Ack(OrderAck {
            seqnum: seqnum,
            order_id: order_id,
        })),
    }
}
fn wrap_order_fill_response(response: OrderFillResponse) -> RpcOrderResponse {
    RpcOrderResponse {
        response: Some(Response::Fill(OrderFill {
            order_id: response.order_id,
            price: response.fill_price,
            size: response.fill_size,
        })),
    }
}
fn wrap_order_dead_response(response: OrderDeadResponse) -> RpcOrderResponse {
    RpcOrderResponse {
        response: Some(Response::Dead(OrderDead {
            order_id: response.order_id,
        })),
    }
}

// Subscribe rpc

// parse rpc subscribe request to portal request
pub fn parse_subscribe_request(sub_id: SubId) -> PortalRequest {
    PortalRequest::EventHistory(sub_id)
}

// parse rpc unsubscribe request to portal request
pub fn wrap_event(event: Event) -> RpcSubscribeResponse {
    match event {
        Event::OrderAdded(added) => RpcSubscribeResponse {
            response: Some(rpc_subscribe_response::Response::Added(
                rpc_subscribe_response::OrderAdded {
                    order_id: added.order_id,
                    ticker: added.ticker,
                    direction: wrap_direction(added.direction),
                    limit_price: added.limit_price,
                    size: added.resting_size,
                },
            )),
        },
        Event::OrderExecuted(executed) => RpcSubscribeResponse {
            response: Some(rpc_subscribe_response::Response::Executed(
                rpc_subscribe_response::OrderExecuted {
                    order_id: executed.order_id,
                    ticker: executed.ticker,
                    execution_price: executed.execution_price,
                    execution_size: executed.execution_size,
                },
            )),
        },
        Event::OrderRemoved(removed) => RpcSubscribeResponse {
            response: Some(rpc_subscribe_response::Response::Removed(
                rpc_subscribe_response::OrderRemoved {
                    order_id: removed.order_id,
                },
            )),
        },
    }
}
