// Orderbook: stores and maintains all resting order for a ticker
// - stores all resting orders in two hashmaps (one for all buy orders, one for all sell orders)

use crate::types::common::*;
use crate::types::event::{Event, OrderAdded, OrderExecuted, OrderRemoved};
use crate::types::order::{BuyOrder, SellOrder};
use crate::types::orderbook::*;
use crate::types::portal::OrderResponse;
use std::collections::{BinaryHeap, HashSet};

pub struct OrderBook {
    ticker: Ticker,
    buy_orders: BinaryHeap<BuyOrder>,
    sell_orders: BinaryHeap<SellOrder>,
    lazy_deleted: HashSet<OrderId>,
}

impl OrderBook {
    pub fn new(ticker: Ticker) -> OrderBook {
        OrderBook {
            ticker,
            buy_orders: BinaryHeap::new(),
            sell_orders: BinaryHeap::new(),
            lazy_deleted: HashSet::new(),
        }
    }

    // Find the best buy order and pop it off the heap
    fn get_best_buy_order(&mut self) -> Option<BuyOrder> {
        while let Some(best_order) = self.buy_orders.pop() {
            if !self.lazy_deleted.contains(&best_order.order_id) {
                return Some(best_order);
            } else {
                self.lazy_deleted.remove(&best_order.order_id);
                continue;
            }
        }
        None
    }

    // Find the best sell order and pop it off the heap
    fn get_best_sell_order(&mut self) -> Option<SellOrder> {
        while let Some(best_order) = self.sell_orders.pop() {
            if !self.lazy_deleted.contains(&best_order.order_id) {
                return Some(best_order);
            } else {
                self.lazy_deleted.remove(&best_order.order_id);
                continue;
            }
        }
        None
    }

    // Helper function to generate logs for ONE order in a trade: OrderFill and OrderExecuted
    fn generate_trade_log(
        &self,
        order_id: OrderId,
        fill_size: Size,
        fill_price: Price,
    ) -> Vec<OrderbookLog> {
        vec![
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id,
                fill_size,
                fill_price,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id,
                ticker: self.ticker.clone(),
                execution_size: fill_size,
                execution_price: fill_price,
            })),
        ]
    }

    // Handle a new buy order. Could result in multiple trades and/or a new resting buy order and/or dead order for itself/other orders
    fn handle_new_buy_order(&mut self, req: NewOrderRequest) -> Vec<OrderbookLog> {
        let mut responses: Vec<OrderbookLog> = vec![];
        let mut left_size: Size = req.size;

        // try matching with resting sell orders
        while let Some(best_sell_order) = self.get_best_sell_order() {
            if best_sell_order.price > req.price || left_size == 0 {
                self.sell_orders.push(best_sell_order);
                break;
            }
            let fill_size: Size = std::cmp::min(req.size, best_sell_order.size);
            let fill_price: Price = best_sell_order.price;

            // modify resting order
            responses.extend(self.generate_trade_log(
                best_sell_order.order_id,
                fill_size,
                fill_price,
            ));
            if fill_size < best_sell_order.size {
                self.sell_orders.push(SellOrder {
                    order_id: best_sell_order.order_id,
                    size: best_sell_order.size - fill_size,
                    price: best_sell_order.price,
                    timestamp: best_sell_order.timestamp,
                });
            } else {
                responses.push(OrderbookLog::OrderLog(OrderResponse::OrderDead(
                    OrderDeadResponse {
                        order_id: best_sell_order.order_id,
                    },
                )));
            }

            // modify incoming order
            responses.extend(self.generate_trade_log(req.order_id, fill_size, fill_price));
            left_size -= fill_size;
        }
        // deal with remaining active buy order
        let should_insert = left_size > 0
            && req.limit_or_market == LimitOrMarket::Limit
            && req.time_in_force == TimeInForce::Day;
        if should_insert {
            self.buy_orders.push(BuyOrder {
                order_id: req.order_id,
                size: left_size,
                price: req.price,
                timestamp: req.timestamp,
            });
            responses.push(OrderbookLog::EventLog(Event::OrderAdded(OrderAdded {
                order_id: req.order_id,
                ticker: self.ticker.clone(),
                direction: req.direction,
                resting_size: left_size,
                limit_price: req.price,
            })))
        } else {
            responses.push(OrderbookLog::OrderLog(OrderResponse::OrderDead(
                OrderDeadResponse {
                    order_id: req.order_id,
                },
            )));
        }

        responses
    }

    // Handle a new sell order. Could result in multiple trades and/or a new resting sell order and/or dead order for itself/other orders
    fn handle_new_sell_order(&mut self, req: NewOrderRequest) -> Vec<OrderbookLog> {
        let mut responses: Vec<OrderbookLog> = Vec::new();
        let mut left_size: Size = req.size;

        // try matching with resting buy orders
        while let Some(best_buy_order) = self.get_best_buy_order() {
            if best_buy_order.price < req.price || left_size == 0 {
                self.buy_orders.push(best_buy_order);
                break;
            }
            let fill_size: Size = std::cmp::min(req.size, best_buy_order.size);
            let fill_price: Price = best_buy_order.price;

            // modify resting order
            responses.extend(self.generate_trade_log(
                best_buy_order.order_id,
                fill_size,
                fill_price,
            ));
            if fill_size < best_buy_order.size {
                self.buy_orders.push(BuyOrder {
                    order_id: best_buy_order.order_id,
                    size: best_buy_order.size - fill_size,
                    price: best_buy_order.price,
                    timestamp: best_buy_order.timestamp,
                });
            } else {
                responses.push(OrderbookLog::OrderLog(OrderResponse::OrderDead(
                    OrderDeadResponse {
                        order_id: best_buy_order.order_id,
                    },
                )));
            }

            // modify incoming order
            responses.extend(self.generate_trade_log(req.order_id, fill_size, fill_price));
            left_size -= fill_size;
        }
        // deal with remaining active sell order
        let should_insert = left_size > 0
            && req.limit_or_market == LimitOrMarket::Limit
            && req.time_in_force == TimeInForce::Day;
        if should_insert {
            self.sell_orders.push(SellOrder {
                order_id: req.order_id,
                size: left_size,
                price: req.price,
                timestamp: req.timestamp,
            });
            responses.push(OrderbookLog::EventLog(Event::OrderAdded(OrderAdded {
                order_id: req.order_id,
                ticker: self.ticker.clone(),
                direction: req.direction,
                resting_size: left_size,
                limit_price: req.price,
            })))
        } else {
            responses.push(OrderbookLog::OrderLog(OrderResponse::OrderDead(
                OrderDeadResponse {
                    order_id: req.order_id,
                },
            )));
        }

        responses
    }

    fn handle_new_order(&mut self, req: NewOrderRequest) -> Vec<OrderbookLog> {
        match req.direction {
            Direction::Buy => self.handle_new_buy_order(req),
            Direction::Sell => self.handle_new_sell_order(req),
        }
    }

    fn handle_cancel_order(&mut self, req: CancelOrderRequest) -> Vec<OrderbookLog> {
        self.lazy_deleted.insert(req.order_id);
        vec![
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: req.order_id,
            })),
            OrderbookLog::EventLog(Event::OrderRemoved(OrderRemoved {
                order_id: req.order_id,
            })),
        ]
    }

    // Handle a request from the portal
    pub fn handle_request(&mut self, req: OrderbookRequest) -> Vec<OrderbookLog> {
        match req {
            OrderbookRequest::NewOrder(new_order_req) => self.handle_new_order(new_order_req),
            OrderbookRequest::CancelOrder(cancel_order_req) => {
                self.handle_cancel_order(cancel_order_req)
            }
        }
    }

    // Get the best buy price without modifying the orderbook
    pub fn best_buy_price(&mut self) -> Option<Price> {
        if let Some(best_buy_order) = self.get_best_buy_order() {
            let best_price = best_buy_order.price.clone();
            self.buy_orders.push(best_buy_order);
            Some(best_price)
        } else {
            None
        }
    }

    // Get the best sell price without modifying the orderbook
    pub fn best_sell_price(&mut self) -> Option<Price> {
        if let Some(best_sell_order) = self.get_best_sell_order() {
            let best_price = best_sell_order.price.clone();
            self.sell_orders.push(best_sell_order);
            Some(best_price)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::event::{OrderAdded, OrderExecuted, OrderRemoved};

    use super::*;

    fn same_response_list(actual: Vec<OrderbookLog>, expected: Vec<OrderbookLog>) -> bool {
        // now we only check the exact same order of responses
        // could be improved by only checking the content regardless of order
        if actual == expected {
            true
        } else {
            // DBG
            println!("Expected: {:?}", expected);
            println!("Actual: {:?}", actual);
            false
        }
    }

    #[test]
    fn test_best_order() {
        // 101 buy 100 @ 10.0 1
        // 102 buy 100 @ 15.0 2
        // 103 sell 50 @ 5.0 3
        // 104 buy 80 @ 5.0 4
        // cancel 101
        // 105 sell 200 @ 5.0 5
        let mut order_book = OrderBook::new("AAPL".to_string());
        let req1 = NewOrderRequest {
            order_id: 101,
            direction: Direction::Buy,
            size: 100,
            price: 10.0,
            timestamp: 1,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let req2 = NewOrderRequest {
            order_id: 102,
            direction: Direction::Buy,
            size: 100,
            price: 15.0,
            timestamp: 2,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let req3 = NewOrderRequest {
            order_id: 103,
            direction: Direction::Sell,
            size: 50,
            price: 5.0,
            timestamp: 3,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let req4 = NewOrderRequest {
            order_id: 104,
            direction: Direction::Buy,
            size: 80,
            price: 5.0,
            timestamp: 4,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let req5 = CancelOrderRequest { order_id: 101 };
        let req6 = NewOrderRequest {
            order_id: 105,
            direction: Direction::Sell,
            size: 200,
            price: 5.0,
            timestamp: 5,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let _ = order_book.handle_new_order(req1);
        assert!(order_book.best_buy_price().unwrap() == 10.0);
        assert!(order_book.best_sell_price().is_none());
        let _ = order_book.handle_new_order(req2);
        assert!(order_book.best_buy_price().unwrap() == 15.0);
        assert!(order_book.best_sell_price().is_none());
        let _ = order_book.handle_new_order(req3);
        assert!(order_book.best_buy_price().unwrap() == 15.0);
        assert!(order_book.best_sell_price().is_none());

        let _ = order_book.handle_new_order(req4);
        assert!(order_book.best_buy_price().unwrap() == 15.0);
        assert!(order_book.best_sell_price().is_none());

        let _ = order_book.handle_cancel_order(req5);
        assert!(order_book.best_buy_price().unwrap() == 15.0);
        assert!(order_book.best_sell_price().is_none());

        let _ = order_book.handle_new_order(req6);
        assert!(order_book.best_buy_price().is_none());
        assert!(order_book.best_sell_price().unwrap() == 5.0);
    }

    #[test]
    fn test_basic_matching() {
        // 101 buy 100 @ 10.0 1
        // 102 buy 100 @ 6.0 2
        // cancel 101
        // 103 sell 50 @ 5.0 3
        // 104 sell 100 @ 4.0 4
        // 105 buy 100 @ 7.0 5

        let mut order_book = OrderBook::new("AAPL".to_string());

        let req1 = NewOrderRequest {
            order_id: 101,
            direction: Direction::Buy,
            size: 100,
            price: 10.0,
            timestamp: 1,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let req2 = NewOrderRequest {
            order_id: 102,
            direction: Direction::Buy,
            size: 100,
            price: 6.0,
            timestamp: 2,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let req3 = CancelOrderRequest { order_id: 101 };
        let req4 = NewOrderRequest {
            order_id: 103,
            direction: Direction::Sell,
            size: 50,
            price: 5.0,
            timestamp: 3,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let req5 = NewOrderRequest {
            order_id: 104,
            direction: Direction::Sell,
            size: 100,
            price: 4.0,
            timestamp: 4,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let req6 = NewOrderRequest {
            order_id: 105,
            direction: Direction::Buy,
            size: 100,
            price: 7.0,
            timestamp: 5,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };

        let resp1: Vec<OrderbookLog> = order_book.handle_new_order(req1);
        let expected_resp1 = vec![OrderbookLog::EventLog(Event::OrderAdded(OrderAdded {
            order_id: 101,
            ticker: "AAPL".to_string(),
            direction: Direction::Buy,
            resting_size: 100,
            limit_price: 10.0,
        }))];
        assert!(same_response_list(resp1, expected_resp1));

        let resp2: Vec<OrderbookLog> = order_book.handle_new_order(req2);
        let expected_resp2 = vec![OrderbookLog::EventLog(Event::OrderAdded(OrderAdded {
            order_id: 102,
            ticker: "AAPL".to_string(),
            direction: Direction::Buy,
            resting_size: 100,
            limit_price: 6.0,
        }))];
        assert!(same_response_list(resp2, expected_resp2));

        let resp3: Vec<OrderbookLog> = order_book.handle_cancel_order(req3);
        let expected_resp3 = vec![
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 101,
            })),
            OrderbookLog::EventLog(Event::OrderRemoved(OrderRemoved { order_id: 101 })),
        ];
        assert!(same_response_list(resp3, expected_resp3));

        let resp4: Vec<OrderbookLog> = order_book.handle_new_order(req4);
        let expected_resp4 = vec![
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 102,
                fill_size: 50,
                fill_price: 6.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 102,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 6.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 103,
                fill_size: 50,
                fill_price: 6.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 103,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 6.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 103,
            })),
        ];
        assert!(same_response_list(resp4, expected_resp4));

        let resp5: Vec<OrderbookLog> = order_book.handle_new_order(req5);
        let expected_resp5 = vec![
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 102,
                fill_size: 50,
                fill_price: 6.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 102,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 6.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 102,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 104,
                fill_size: 50,
                fill_price: 6.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 104,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 6.0,
            })),
            OrderbookLog::EventLog(Event::OrderAdded(OrderAdded {
                order_id: 104,
                ticker: "AAPL".to_string(),
                direction: Direction::Sell,
                resting_size: 50,
                limit_price: 4.0,
            })),
        ];
        assert!(same_response_list(resp5, expected_resp5));

        let resp6: Vec<OrderbookLog> = order_book.handle_new_order(req6);
        let expected_resp6 = vec![
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 104,
                fill_size: 50,
                fill_price: 4.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 104,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 4.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 104,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 105,
                fill_size: 50,
                fill_price: 4.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 105,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 4.0,
            })),
            OrderbookLog::EventLog(Event::OrderAdded(OrderAdded {
                order_id: 105,
                ticker: "AAPL".to_string(),
                direction: Direction::Buy,
                resting_size: 50,
                limit_price: 7.0,
            })),
        ];
        assert!(same_response_list(resp6, expected_resp6));
    }

    #[test]
    fn test_market_sell_order() {
        // 101 buy 100 @ 10.0 1
        // 102 sell 50 @ market 2
        // 103 sell 100 @ market 3

        let mut order_book = OrderBook::new("AAPL".to_string());
        let req1 = NewOrderRequest {
            order_id: 101,
            direction: Direction::Buy,
            size: 100,
            price: 10.0,
            timestamp: 1,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let resp1: Vec<OrderbookLog> = order_book.handle_new_order(req1);
        let expected_resp1 = vec![OrderbookLog::EventLog(Event::OrderAdded(OrderAdded {
            order_id: 101,
            ticker: "AAPL".to_string(),
            direction: Direction::Buy,
            resting_size: 100,
            limit_price: 10.0,
        }))];
        assert!(same_response_list(resp1, expected_resp1));

        let req2 = NewOrderRequest {
            order_id: 102,
            direction: Direction::Sell,
            size: 50,
            price: order_book.best_buy_price().unwrap(),
            timestamp: 2,
            limit_or_market: LimitOrMarket::Market,
            time_in_force: TimeInForce::Day,
        };
        let resp2: Vec<OrderbookLog> = order_book.handle_new_order(req2);
        let expected_resp2 = vec![
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 101,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 101,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 102,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 102,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 102,
            })),
        ];
        assert!(same_response_list(resp2, expected_resp2));

        let req3 = NewOrderRequest {
            order_id: 103,
            direction: Direction::Sell,
            size: 100,
            price: order_book.best_buy_price().unwrap(),
            timestamp: 3,
            limit_or_market: LimitOrMarket::Market,
            time_in_force: TimeInForce::Day,
        };
        let resp3: Vec<OrderbookLog> = order_book.handle_new_order(req3);
        let expected_resp3 = vec![
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 101,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 101,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 101,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 103,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 103,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 103,
            })),
        ];
        assert!(same_response_list(resp3, expected_resp3));
    }

    #[test]
    fn test_market_buy_order() {
        // 101 sell 100 @ 10.0 1
        // 102 buy 50 @ market 2
        // 103 buy 100 @ market 3
        let mut order_book = OrderBook::new("AAPL".to_string());
        let req1 = NewOrderRequest {
            order_id: 101,
            direction: Direction::Sell,
            size: 100,
            price: 10.0,
            timestamp: 1,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let resp1: Vec<OrderbookLog> = order_book.handle_new_order(req1);
        let expected_resp1 = vec![OrderbookLog::EventLog(Event::OrderAdded(OrderAdded {
            order_id: 101,
            ticker: "AAPL".to_string(),
            direction: Direction::Sell,
            resting_size: 100,
            limit_price: 10.0,
        }))];
        assert!(same_response_list(resp1, expected_resp1));

        let req2 = NewOrderRequest {
            order_id: 102,
            direction: Direction::Buy,
            size: 50,
            price: order_book.best_sell_price().unwrap(),
            timestamp: 2,
            limit_or_market: LimitOrMarket::Market,
            time_in_force: TimeInForce::Day,
        };
        let resp2: Vec<OrderbookLog> = order_book.handle_new_order(req2);
        let expected_resp2 = vec![
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 101,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 101,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 102,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 102,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 102,
            })),
        ];
        assert!(same_response_list(resp2, expected_resp2));

        let req3 = NewOrderRequest {
            order_id: 103,
            direction: Direction::Buy,
            size: 100,
            price: order_book.best_sell_price().unwrap(),
            timestamp: 3,
            limit_or_market: LimitOrMarket::Market,
            time_in_force: TimeInForce::Day,
        };
        let resp3: Vec<OrderbookLog> = order_book.handle_new_order(req3);
        let expected_resp3 = vec![
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 101,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 101,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 101,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 103,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 103,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 103,
            })),
        ];
        assert!(same_response_list(resp3, expected_resp3));
    }

    #[test]
    fn test_ioc_order() {
        // 101 buy 50 @ 10.0 1
        // 102 sell 100 @ 15.0 2 IOC
        // 103 sell 100 @ 6.0 3 IOC
        // 104 buy 100 @ 20.0 4
        let mut order_book = OrderBook::new("AAPL".to_string());
        let req1 = NewOrderRequest {
            order_id: 101,
            direction: Direction::Buy,
            size: 50,
            price: 10.0,
            timestamp: 1,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::Day,
        };
        let resp1: Vec<OrderbookLog> = order_book.handle_new_order(req1);
        let expected_resp1 = vec![OrderbookLog::EventLog(Event::OrderAdded(OrderAdded {
            order_id: 101,
            ticker: "AAPL".to_string(),
            direction: Direction::Buy,
            resting_size: 50,
            limit_price: 10.0,
        }))];
        assert!(same_response_list(resp1, expected_resp1));

        let req2 = NewOrderRequest {
            order_id: 102,
            direction: Direction::Sell,
            size: 100,
            price: 15.0,
            timestamp: 2,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::IOC,
        };
        let resp2: Vec<OrderbookLog> = order_book.handle_new_order(req2);
        let expected_resp2 = vec![OrderbookLog::OrderLog(OrderResponse::OrderDead(
            OrderDeadResponse { order_id: 102 },
        ))];
        assert!(same_response_list(resp2, expected_resp2));

        let req3 = NewOrderRequest {
            order_id: 103,
            direction: Direction::Sell,
            size: 100,
            price: 6.0,
            timestamp: 3,
            limit_or_market: LimitOrMarket::Limit,
            time_in_force: TimeInForce::IOC,
        };
        let resp3: Vec<OrderbookLog> = order_book.handle_new_order(req3);
        let expected_resp3 = vec![
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 101,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 101,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 101,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderFill(OrderFillResponse {
                order_id: 103,
                fill_size: 50,
                fill_price: 10.0,
            })),
            OrderbookLog::EventLog(Event::OrderExecuted(OrderExecuted {
                order_id: 103,
                ticker: "AAPL".to_string(),
                execution_size: 50,
                execution_price: 10.0,
            })),
            OrderbookLog::OrderLog(OrderResponse::OrderDead(OrderDeadResponse {
                order_id: 103,
            })),
        ];
        assert!(same_response_list(resp3, expected_resp3));
    }
}
