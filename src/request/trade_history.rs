use crate::types::TradeId;

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct GetTradeHistoryOptions {
    pub max_trades: u32,
    pub start_after_time: Option<u32>,
    pub start_after_tradeid: Option<TradeId>,
    pub navigating_back: bool,
    pub include_failed: bool,
}