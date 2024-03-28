// Account: a struct that stores the information of a single account/investor

use crate::types::{
    account_manager::{AccountUpdate, PotentialOrder},
    common::{AccountName, Cash, InvId, Password, Size, Ticker},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Account {
    pub inv_id: InvId,
    pub acc_name: AccountName,
    pub password: Password,
    pub cash: Cash,
    pub positions: HashMap<Ticker, Size>,
}

impl Account {
    // Init
    pub fn new(inv_id: InvId, acc_name: AccountName, password: Password, cash: Cash) -> Self {
        Account {
            inv_id,
            acc_name,
            password,
            cash,
            positions: HashMap::new(),
        }
    }

    // Used in initialization
    pub fn add_position(&mut self, ticker: Ticker, size: Size) {
        self.positions.insert(ticker, size);
    }

    // Check if the potential order is valid: enough cash or enough positions
    pub fn valid_potential_order(&self, p_order: &PotentialOrder) -> bool {
        match p_order {
            PotentialOrder::PotentialBuy(total_price) => &self.cash >= &total_price,
            PotentialOrder::PotentialSell(size, ticker) => self
                .positions
                .get(ticker)
                .map_or(false, |own_size| own_size >= &size),
        }
    }

    // Update account with account update: update cash or positions
    pub fn update(&mut self, update: AccountUpdate) {
        match update {
            AccountUpdate::UpdCash(_inv_id, delta_cash) => {
                self.cash += delta_cash;
            }
            AccountUpdate::AddPos(_inv_id, ticker, add_size) => {
                self.positions
                    .entry(ticker)
                    .and_modify(|size| *size += add_size)
                    .or_insert(add_size);
            }
            AccountUpdate::MinusPos(_inv_id, ticker, minus_size) => {
                self.positions
                    .entry(ticker)
                    .and_modify(|size| *size -= minus_size);
            }
        }
    }
}
