use super::common::{OrderId, Price, Size, Timestamp};
use std::cmp::Ordering;

#[derive(Debug)]
pub struct BuyOrder {
    pub order_id: OrderId,
    pub size: Size,
    pub price: Price,
    pub timestamp: Timestamp,
}

impl Ord for BuyOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.partial_cmp(&other.price).unwrap() {
            Ordering::Equal => match self.timestamp.cmp(&other.timestamp) {
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Less,
                Ordering::Less => Ordering::Greater,
            },
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
        }
    }
}

impl PartialOrd for BuyOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BuyOrder {
    fn eq(&self, other: &Self) -> bool {
        self.order_id == other.order_id
    }
}

impl Eq for BuyOrder {}

#[derive(Debug)]
pub struct SellOrder {
    pub order_id: OrderId,
    pub size: Size,
    pub price: Price,
    pub timestamp: Timestamp,
}

impl Ord for SellOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.partial_cmp(&other.price).unwrap() {
            Ordering::Equal => match self.timestamp.cmp(&other.timestamp) {
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Less,
                Ordering::Less => Ordering::Greater,
            },
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

impl PartialOrd for SellOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SellOrder {
    fn eq(&self, other: &Self) -> bool {
        self.order_id == other.order_id
    }
}

impl Eq for SellOrder {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BinaryHeap;

    // Test the ordering of buy orders and sell orders
    #[test]
    fn test_buyorder_ordering() {
        let buy_order1 = BuyOrder {
            order_id: 1,
            size: 100,
            price: 100.0,
            timestamp: 5,
        };
        let buy_order2 = BuyOrder {
            order_id: 2,
            size: 100,
            price: 100.0,
            timestamp: 6,
        };
        let buy_order3 = BuyOrder {
            order_id: 3,
            size: 100,
            price: 101.0,
            timestamp: 7,
        };
        let buy_order4 = BuyOrder {
            order_id: 4,
            size: 100,
            price: 99.0,
            timestamp: 8,
        };
        assert_eq!(buy_order1.cmp(&buy_order2), Ordering::Greater);
        assert_eq!(buy_order1.cmp(&buy_order3), Ordering::Less);
        assert_eq!(buy_order1.cmp(&buy_order4), Ordering::Greater);
        assert_eq!(buy_order2.cmp(&buy_order3), Ordering::Less);
        assert_eq!(buy_order2.cmp(&buy_order4), Ordering::Greater);
        assert_eq!(buy_order3.cmp(&buy_order4), Ordering::Greater);

        let mut buy_orders = BinaryHeap::new();
        buy_orders.push(buy_order4);
        buy_orders.push(buy_order3);
        buy_orders.push(buy_order2);
        buy_orders.push(buy_order1);
        assert_eq!(buy_orders.pop().unwrap().order_id, 3);
        assert_eq!(buy_orders.pop().unwrap().order_id, 1);
        assert_eq!(buy_orders.pop().unwrap().order_id, 2);
        assert_eq!(buy_orders.pop().unwrap().order_id, 4);
    }

    #[test]
    fn test_sellorder_ordering() {
        let sell_order1 = SellOrder {
            order_id: 1,
            size: 100,
            price: 100.0,
            timestamp: 5,
        };
        let sell_order2 = SellOrder {
            order_id: 2,
            size: 100,
            price: 100.0,
            timestamp: 6,
        };
        let sell_order3 = SellOrder {
            order_id: 3,
            size: 100,
            price: 101.0,
            timestamp: 7,
        };
        let sell_order4 = SellOrder {
            order_id: 4,
            size: 100,
            price: 99.0,
            timestamp: 8,
        };
        assert_eq!(sell_order1.cmp(&sell_order2), Ordering::Greater);
        assert_eq!(sell_order1.cmp(&sell_order3), Ordering::Greater);
        assert_eq!(sell_order1.cmp(&sell_order4), Ordering::Less);
        assert_eq!(sell_order2.cmp(&sell_order3), Ordering::Greater);
        assert_eq!(sell_order2.cmp(&sell_order4), Ordering::Less);
        assert_eq!(sell_order3.cmp(&sell_order4), Ordering::Less);

        let mut sell_orders = BinaryHeap::new();
        sell_orders.push(sell_order4);
        sell_orders.push(sell_order3);
        sell_orders.push(sell_order2);
        sell_orders.push(sell_order1);
        assert_eq!(sell_orders.pop().unwrap().order_id, 4);
        assert_eq!(sell_orders.pop().unwrap().order_id, 1);
        assert_eq!(sell_orders.pop().unwrap().order_id, 2);
        assert_eq!(sell_orders.pop().unwrap().order_id, 3);
    }
}
