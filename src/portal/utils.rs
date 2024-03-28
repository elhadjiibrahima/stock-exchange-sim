use super::{account::Account, order_info::OrderRecord, stock_manager::StockRecord};
use crate::types::{
    account_manager::AccountUpdate,
    common::{Direction, Size},
    config::{InvestorList, StockList},
    portal::OrderResponse,
};
use std::{fs::File, io::Read};

// Generate AccountUpdate instructions from one OrderResponse
pub fn orderresponse_to_acc_update(
    orderbook_log: OrderResponse,
    order_rec: &OrderRecord,
    resting_size: Option<Size>,
) -> Vec<AccountUpdate> {
    // convert orderbook log to account update
    let mut acc_updates: Vec<AccountUpdate> = vec![];
    match orderbook_log {
        OrderResponse::OrderFill(order_fill) => match order_rec.direction {
            Direction::Buy => {
                acc_updates.push(AccountUpdate::UpdCash(
                    order_rec.inv_id,
                    (order_fill.fill_size as f32) * (order_rec.limit_price - order_fill.fill_price),
                ));
                acc_updates.push(AccountUpdate::AddPos(
                    order_rec.inv_id,
                    order_rec.ticker.clone(),
                    order_fill.fill_size,
                ));
            }
            Direction::Sell => {
                acc_updates.push(AccountUpdate::UpdCash(
                    order_rec.inv_id,
                    (order_fill.fill_size as f32) * order_fill.fill_price,
                ));
            }
        },
        OrderResponse::OrderDead(_order_dead) => match order_rec.direction {
            Direction::Buy => {
                if let Some(resting_size) = resting_size {
                    acc_updates.push(AccountUpdate::UpdCash(
                        order_rec.inv_id,
                        (resting_size as f32) * order_rec.limit_price,
                    ));
                }
            }
            Direction::Sell => {
                if let Some(resting_size) = resting_size {
                    acc_updates.push(AccountUpdate::AddPos(
                        order_rec.inv_id,
                        order_rec.ticker.clone(),
                        resting_size,
                    ));
                }
            }
        },
    }
    acc_updates
}

pub fn load_investors_from_config(investor_config_file: String) -> Vec<Account> {
    let cur_dir = std::env::current_dir().unwrap();
    let path = cur_dir.join(investor_config_file);
    let mut file = File::open(path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let investors: InvestorList =
        serde_json::from_str(&contents).expect("Unable to load investor config");

    let mut accounts: Vec<Account> = vec![];
    for investor in investors.investors {
        let mut acc = Account::new(
            investor.inv_id,
            investor.account_name,
            investor.password,
            investor.cash_amount,
        );
        for (ticker, size) in investor.stocks {
            acc.add_position(ticker, size.as_f64().unwrap() as Size);
        }
        accounts.push(acc);
    }
    accounts
}

pub fn load_stocks_from_config(stock_config_file: String) -> Vec<(String, StockRecord)> {
    let cur_dir = std::env::current_dir().unwrap();
    let path = cur_dir.join(stock_config_file);
    let mut file = File::open(path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let stocks: StockList = serde_json::from_str(&contents).expect("Unable to load stock config");

    let mut stock_records: Vec<(String, StockRecord)> = vec![];

    for stock_config in stocks.stocks {
        let stock_record = StockRecord {
            close_price: stock_config.close_price,
            lot_size: stock_config.lot_size,
            mpf: stock_config.mpf,
            name: stock_config.name,
        };
        stock_records.push((stock_config.ticker, stock_record));
    }

    stock_records
}
