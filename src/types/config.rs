use serde::Deserialize;

// Config struct for Investor config file
#[derive(Debug, Deserialize)]
pub struct InvestorConfig {
    pub inv_id: u64,
    pub account_name: String,
    pub password: String,
    pub stocks: serde_json::Map<String, serde_json::Value>,
    pub cash_amount: f32,
}
#[derive(Debug, Deserialize)]
pub struct InvestorList {
    pub investors: Vec<InvestorConfig>,
}

// Config struct for Stock config file
#[derive(Debug, Deserialize)]
pub struct StockConfig {
    pub ticker: String,
    pub close_price: f32,
    pub lot_size: u32,
    pub mpf: f32,
    pub name: String,
}
#[derive(Debug, Deserialize)]
pub struct StockList {
    pub stocks: Vec<StockConfig>,
}
