// Portal is the core logic of the server.
// -  contains the orderbook manager, event history, order info, account manager, and stock manager.
// -  provide APIs for server to process requests and return triggered tasks for server to dispatch.

use crate::types::account_manager::PotentialOrder;
use crate::types::common::{Direction, InvId, OrderId, Password, SeqNum, Ticker};
use crate::types::orderbook::{
    CancelOrderRequest, NewOrderRequest, OrderbookLog, OrderbookRequest,
};
use crate::types::portal::{PortalNewOrderRequest, PortalRequest, PortalTask};
use crate::utils::get_order_id;
use std::vec;

mod account;
mod account_manager;
mod event_history;
mod order_info;
mod orderbook;
mod orderbook_manager;
mod stock_manager;
mod utils;

use self::account::Account;
use self::account_manager::AccountManager;
use self::event_history::EventHistory;
use self::order_info::OrderInfo;
use self::orderbook_manager::OrderbookManager;
use self::stock_manager::{StockManager, StockRecord};
use self::utils::orderresponse_to_acc_update;
use self::utils::{load_investors_from_config, load_stocks_from_config};

pub struct Portal {
    orderbook_manager: OrderbookManager,
    event_history: EventHistory,
    order_info: OrderInfo,
    account_manager: AccountManager,
    stock_manager: StockManager,
    last_order_id: u64,
}

impl Portal {
    // Initialize a portal with stocks and investors
    pub fn new(investor_config: String, stock_config: String) -> Self {
        let mut orderbook_manager = OrderbookManager::new();
        let event_history = EventHistory::new();
        let order_info = OrderInfo::new();
        let mut account_manager = AccountManager::new();
        let mut stock_manager = StockManager::new();

        // configure stocks
        let stocks: Vec<(Ticker, StockRecord)> = load_stocks_from_config(stock_config);
        for (ticker, stock_rec) in stocks {
            stock_manager.bind_stock(ticker.clone(), stock_rec);
            orderbook_manager.add_orderbook(ticker.clone());
        }
        // configure investors
        let investors: Vec<Account> = load_investors_from_config(investor_config);
        for inv in investors {
            account_manager.add_account(inv.inv_id, inv);
        }
        Portal {
            orderbook_manager,
            event_history,
            order_info,
            account_manager,
            stock_manager,
            last_order_id: 0,
        }
    }

    // Process a log, update portal, and return triggered tasks
    fn process_log(&mut self, log: OrderbookLog) -> PortalTask {
        match log {
            OrderbookLog::OrderLog(order_resp) => {
                let order_id = get_order_id(&order_resp);
                let resting_size = self.order_info.get_resting(&order_id);
                let order_rec = self.order_info.get_order_record(&order_id).unwrap();
                // convert to PortalTask
                let task = PortalTask::OrderResponse(order_rec.inv_id.clone(), order_resp.clone());
                // update portal
                let updates = orderresponse_to_acc_update(order_resp, order_rec, resting_size);
                for upd in updates {
                    self.account_manager.update(upd);
                }
                task
            }
            OrderbookLog::EventLog(event) => {
                // convert to PortalTask
                let task = PortalTask::IncrementalEvent(event.clone());
                // update portal
                self.event_history.update_by_event(event.clone());
                self.order_info.update_by_event(event.clone());
                task
            }
        }
    }

    fn process_logs(&mut self, logs: Vec<OrderbookLog>) -> Vec<PortalTask> {
        let mut tasks = vec![];
        for log in logs {
            tasks.push(self.process_log(log));
        }
        tasks
    }

    fn find_ticker_by_order_id(&self, order_id: u64) -> Option<Ticker> {
        self.order_info
            .get_order_record(&order_id)
            .map(|order_rec| order_rec.ticker.clone())
    }

    fn generate_order_id(&mut self) -> u64 {
        self.last_order_id += 1;
        self.last_order_id
    }

    // Try to login with inv_id and password
    pub fn try_login(&mut self, inv_id: InvId, password: &Password) -> bool {
        self.account_manager.try_login(inv_id, password)
    }

    // Make a potential order from a new order request
    fn make_potential_order(&self, req: &PortalNewOrderRequest) -> PotentialOrder {
        match req.direction {
            Direction::Buy => PotentialOrder::PotentialBuy(req.price * req.size as f32),
            Direction::Sell => PotentialOrder::PotentialSell(req.size.clone(), req.ticker.clone()),
        }
    }

    // Use best price to fill in req
    fn fill_in_market_order(&mut self, req: PortalNewOrderRequest) -> PortalNewOrderRequest {
        match req.direction {
            Direction::Buy => {
                let best_price = self
                    .orderbook_manager
                    .best_sell_price(&req.ticker)
                    .unwrap_or_else(|| self.stock_manager.get_close_price(&req.ticker).unwrap());

                PortalNewOrderRequest {
                    price: best_price,
                    ..req
                }
            }
            Direction::Sell => {
                let best_price = self
                    .orderbook_manager
                    .best_buy_price(&req.ticker)
                    .unwrap_or_else(|| self.stock_manager.get_close_price(&req.ticker).unwrap());

                PortalNewOrderRequest {
                    price: best_price,
                    ..req
                }
            }
        }
    }

    // process a request and return list of triggered tasks
    pub fn process_request(&mut self, seqnum: SeqNum, req: PortalRequest) -> Vec<PortalTask> {
        match req {
            PortalRequest::EventHistory(sub_id) => {
                let events = self.event_history.get_history();
                vec![PortalTask::EventHistory(sub_id, events)]
            }
            PortalRequest::NewOrder(inv_id, req) => {
                self.process_portal_new_order(inv_id, seqnum, req)
            }
            PortalRequest::CancelOrder(inv_id, order_id) => {
                self.process_portal_cancel_order(inv_id, seqnum, order_id)
            }
        }
    }

    // check if the new order request is valid and process it
    fn process_portal_new_order(
        &mut self,
        inv_id: InvId,
        seqnum: SeqNum,
        req: PortalNewOrderRequest,
    ) -> Vec<PortalTask> {
        if self
            .stock_manager
            .check_valid_order(&req.ticker, &req.price, &req.size)
        {
            let req = self.fill_in_market_order(req);
            let p_order: PotentialOrder = self.make_potential_order(&req);
            if self
                .account_manager
                .valid_potential_order(&inv_id, &p_order)
            {
                // valid new order request
                let order_id = self.generate_order_id();
                let mut tasks: Vec<PortalTask> =
                    vec![PortalTask::OrderAck(inv_id.clone(), seqnum, order_id)];
                self.account_manager
                    .update_by_potential_order(inv_id, p_order);
                self.order_info.add_new_order(&order_id, &inv_id, &req);
                tasks.extend(self.process_new_order(order_id, req));
                tasks
            } else {
                // invalid new order request: insufficient cash or insufficient lot
                vec![PortalTask::OrderReject(
                    inv_id,
                    seqnum,
                    "Invalid new order request: Insufficient cash or lot to complete the order"
                        .to_string(),
                )]
            }
        } else {
            // invalid new order request: invalid price or size
            vec![PortalTask::OrderReject(
                inv_id,
                seqnum,
                "Invalid new order request: Invalid price or size".to_string(),
            )]
        }
    }

    // check if the cancel order request is valid and process it
    fn process_portal_cancel_order(
        &mut self,
        inv_id: InvId,
        seqnum: SeqNum,
        order_id: OrderId,
    ) -> Vec<PortalTask> {
        if self.order_info.valid_cancel_order(&order_id, &inv_id) {
            // valid cancel order request
            let req = OrderbookRequest::CancelOrder(CancelOrderRequest { order_id });
            let ticker = self.find_ticker_by_order_id(order_id).unwrap();
            let logs = self.orderbook_manager.handle_orderbook_request(ticker, req);
            self.process_logs(logs)
        } else {
            // invalid cancel order request
            vec![PortalTask::CancelReject(
                inv_id,
                seqnum,
                "Invalid cancel order request".to_string(),
            )]
        }
    }

    // process a valid new order request and return list of triggered tasks
    fn process_new_order(
        &mut self,
        order_id: OrderId,
        req: PortalNewOrderRequest,
    ) -> Vec<PortalTask> {
        let order_book_req = OrderbookRequest::NewOrder(NewOrderRequest {
            order_id,
            direction: req.direction,
            size: req.size,
            price: req.price,
            limit_or_market: req.limit_or_market,
            time_in_force: req.time_in_force,
            timestamp: req.timestamp,
        });
        let logs = self
            .orderbook_manager
            .handle_orderbook_request(req.ticker, order_book_req);
        self.process_logs(logs)
    }
}
