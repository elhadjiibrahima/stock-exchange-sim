use crate::types::common::{Cash, InvId, Price, Size, Ticker};

// Potential order: required cash or required positions for new order
//                  used to check if an order is valid
#[derive(Debug)]
pub enum PotentialOrder {
    PotentialBuy(Price),
    PotentialSell(Size, Ticker),
}

// AccountUpdate: used to update account's cash or positions
#[derive(Debug)]
pub enum AccountUpdate {
    UpdCash(InvId, Cash),
    AddPos(InvId, Ticker, Size),
    MinusPos(InvId, Ticker, Size),
}
