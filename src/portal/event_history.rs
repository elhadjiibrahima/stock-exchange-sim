// EventHistory: a struct that stores all events

use crate::types::event::Event;

pub struct EventHistory {
    pub events: Vec<Event>,
}

impl EventHistory {
    pub fn new() -> Self {
        EventHistory { events: Vec::new() }
    }

    pub fn get_history(&self) -> Vec<Event> {
        self.events.clone()
    }

    fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn update_by_event(&mut self, event: Event) {
        match event {
            Event::OrderAdded(order_added) => self.add_event(Event::OrderAdded(order_added)),
            Event::OrderExecuted(order_executed) => {
                self.add_event(Event::OrderExecuted(order_executed))
            }
            Event::OrderRemoved(order_removed) => {
                self.add_event(Event::OrderRemoved(order_removed))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        common::Direction,
        event::{OrderAdded, OrderExecuted, OrderRemoved},
    };

    fn same_event_list(actual: Vec<Event>, expected: Vec<Event>) -> bool {
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
    fn test_event_history() {
        let mut event_history = EventHistory::new();
        let resps = vec![
            Event::OrderAdded(OrderAdded {
                order_id: 1,
                ticker: "AAPL".to_string(),
                direction: Direction::Buy,
                resting_size: 100,
                limit_price: 100.0,
            }),
            Event::OrderExecuted(OrderExecuted {
                order_id: 1,
                ticker: "AAPL".to_string(),
                execution_size: 100,
                execution_price: 100.0,
            }),
            Event::OrderRemoved(OrderRemoved { order_id: 1 }),
        ];
        event_history.update_by_event(resps[0].clone());
        event_history.update_by_event(resps[1].clone());
        event_history.update_by_event(resps[2].clone());
        let expected = vec![
            Event::OrderAdded(OrderAdded {
                order_id: 1,
                ticker: "AAPL".to_string(),
                direction: Direction::Buy,
                resting_size: 100,
                limit_price: 100.0,
            }),
            Event::OrderExecuted(OrderExecuted {
                order_id: 1,
                ticker: "AAPL".to_string(),
                execution_size: 100,
                execution_price: 100.0,
            }),
            Event::OrderRemoved(OrderRemoved { order_id: 1 }),
        ];
        let actual: Vec<Event> = event_history.get_history();
        assert!(same_event_list(actual, expected));
    }
}
