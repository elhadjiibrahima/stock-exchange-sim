// OrderInfo: stores and manages all orders: resting size and static properties (OrderRecord) by order_id

use crate::types::{common::*, event::*, portal::PortalNewOrderRequest};
use std::collections::HashMap;

// Static properties once order is added
pub struct OrderRecord {
    pub inv_id: InvId,
    pub ticker: Ticker,
    pub direction: Direction,
    pub limit_price: Price,
    pub initial_size: Size,
}

pub struct OrderInfo {
    pub bind: HashMap<OrderId, OrderRecord>, // static properties
    pub resting: HashMap<OrderId, Size>,     // mutable properties
}

impl OrderInfo {
    pub fn new() -> Self {
        OrderInfo {
            bind: HashMap::new(),
            resting: HashMap::new(),
        }
    }

    // Get resting size of an order
    pub fn get_resting(&mut self, order_id: &OrderId) -> Option<Size> {
        if let Some((size, should_remove)) = self
            .resting
            .get_mut(&order_id)
            .map(|size| (*size, *size == 0))
        {
            if should_remove {
                self.resting.remove(&order_id);
                None
            } else {
                Some(size)
            }
        } else {
            None
        }
    }

    // Check if an order is valid to cancel: order exists and inv_id matches
    pub fn valid_cancel_order(&mut self, order_id: &OrderId, inv_id: &InvId) -> bool {
        if let Some(_) = self.get_resting(order_id) {
            self.bind
                .get(&order_id)
                .map_or(false, |p| p.inv_id == *inv_id)
        } else {
            false
        }
    }

    // Bind an order with its immutable properties
    fn bind_order(&mut self, order_id: OrderId, order_rec: OrderRecord) {
        self.bind.insert(order_id, order_rec);
    }

    // Update resting size by event
    pub fn update_by_event(&mut self, event: Event) {
        match event {
            Event::OrderAdded(order_added) => {
                self.resting
                    .insert(order_added.order_id, order_added.resting_size);
            }
            Event::OrderExecuted(order_executed) => {
                if let Some(size) = self.get_resting(&order_executed.order_id) {
                    self.resting.insert(
                        order_executed.order_id,
                        size - order_executed.execution_size,
                    );
                }
            }
            Event::OrderRemoved(order_removed) => {
                self.resting.remove(&order_removed.order_id);
            }
        }
    }

    // Get order static properties
    pub fn get_order_record(&self, order_id: &OrderId) -> Option<&OrderRecord> {
        self.bind.get(&order_id)
    }

    // Add a new order
    pub fn add_new_order(
        &mut self,
        order_id: &OrderId,
        inv_id: &InvId,
        req: &PortalNewOrderRequest,
    ) {
        self.bind_order(
            order_id.clone(),
            OrderRecord {
                inv_id: inv_id.clone(),
                ticker: req.ticker.clone(),
                direction: req.direction.clone(),
                limit_price: req.price.clone(),
                initial_size: req.size.clone(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_order_record(inv_id: InvId) -> OrderRecord {
        OrderRecord {
            inv_id,
            ticker: "AAPL".to_string(),
            direction: Direction::Buy,
            limit_price: 100.0,
            initial_size: 100,
        }
    }

    // tests on resting size
    #[test]
    fn test_added() {
        let mut order_info = OrderInfo::new();
        order_info.bind_order(1, make_order_record(1));
        order_info.update_by_event(Event::OrderAdded(OrderAdded {
            order_id: 1,
            ticker: "AAPL".to_string(),
            direction: Direction::Buy,
            resting_size: 100,
            limit_price: 100.0,
        }));
    }
    #[test]
    fn test_executed() {
        let mut order_info = OrderInfo::new();
        order_info.bind_order(1, make_order_record(1));
        order_info.update_by_event(Event::OrderAdded(OrderAdded {
            order_id: 1,
            ticker: "AAPL".to_string(),
            direction: Direction::Buy,
            resting_size: 100,
            limit_price: 100.0,
        }));
        assert_eq!(order_info.valid_cancel_order(&1, &1), true);

        order_info.update_by_event(Event::OrderExecuted(OrderExecuted {
            order_id: 1,
            ticker: "AAPL".to_string(),
            execution_size: 100,
            execution_price: 100.0,
        }));
        assert_eq!(order_info.valid_cancel_order(&1, &1), false);
    }
    #[test]
    fn test_removed() {
        let mut order_info = OrderInfo::new();
        order_info.bind_order(1, make_order_record(1));
        order_info.update_by_event(Event::OrderAdded(OrderAdded {
            order_id: 1,
            ticker: "AAPL".to_string(),
            direction: Direction::Buy,
            resting_size: 100,
            limit_price: 100.0,
        }));
        assert_eq!(order_info.valid_cancel_order(&1, &1), true);

        order_info.update_by_event(Event::OrderRemoved(OrderRemoved { order_id: 1 }));
        assert_eq!(order_info.valid_cancel_order(&1, &1), false);
    }
    #[test]
    fn test_invalid() {
        let mut order_info = OrderInfo::new();
        order_info.bind_order(1, make_order_record(1));
        order_info.bind_order(2, make_order_record(2));

        // order not exist
        assert_eq!(order_info.valid_cancel_order(&4, &1), false);
        // inv not exist
        assert_eq!(order_info.valid_cancel_order(&1, &3), false);
        // not the owner
        assert_eq!(order_info.valid_cancel_order(&1, &2), false);

        order_info.update_by_event(Event::OrderAdded(OrderAdded {
            order_id: 1,
            ticker: "AAPL".to_string(),
            direction: Direction::Buy,
            resting_size: 100,
            limit_price: 100.0,
        }));
        order_info.update_by_event(Event::OrderExecuted(OrderExecuted {
            order_id: 1,
            ticker: "AAPL".to_string(),
            execution_size: 100,
            execution_price: 100.0,
        }));

        assert_eq!(order_info.valid_cancel_order(&1, &1), false);
    }
}
