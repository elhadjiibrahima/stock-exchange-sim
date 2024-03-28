// AccountManager: stores and manages the information of all accounts/investors

use super::account::Account;
use crate::types::account_manager::{AccountUpdate, PotentialOrder};
use crate::types::common::*;
use crate::utils::get_inv_id;
use std::collections::{HashMap, HashSet};

pub struct AccountManager {
    accounts: HashMap<InvId, Account>,
    login_accs: HashSet<InvId>,
}

impl AccountManager {
    pub fn new() -> Self {
        AccountManager {
            accounts: HashMap::new(),
            login_accs: HashSet::new(),
        }
    }

    // Used in initialization
    pub fn add_account(&mut self, inv_id: InvId, account: Account) {
        self.accounts.insert(inv_id, account);
    }

    // Check if the potential order is valid: enough cash or enough positions
    pub fn valid_potential_order(&self, inv_id: &InvId, p_order: &PotentialOrder) -> bool {
        self.accounts
            .get(&inv_id)
            .map_or(false, |acc| acc.valid_potential_order(p_order))
    }

    // Update account with account update: update cash or positions
    pub fn update(&mut self, update: AccountUpdate) {
        let inv_id = get_inv_id(&update);
        self.accounts
            .get_mut(&inv_id)
            .map_or((), |acc: &mut Account| acc.update(update))
    }

    // Try to login with inv_id and password. The second time login will fail.
    pub fn try_login(&mut self, inv_id: InvId, password: &Password) -> bool {
        if let Some(account) = self.accounts.get(&inv_id) {
            if account.password == *password && !self.login_accs.contains(&inv_id) {
                self.login_accs.insert(inv_id);
                return true;
            }
        }
        false
    }

    // Advance drawdown: update cash and positions by potential order
    pub fn update_by_potential_order(&mut self, inv_id: InvId, p_order: PotentialOrder) {
        if let Some(_acc) = self.accounts.get(&inv_id) {
            match p_order {
                PotentialOrder::PotentialBuy(total_price) => {
                    self.update(AccountUpdate::UpdCash(inv_id, -total_price));
                }
                PotentialOrder::PotentialSell(size, ticker) => {
                    self.update(AccountUpdate::MinusPos(inv_id, ticker, size));
                }
            }
        }
    }
}
