use crate::types::{ServerTime, TradeId};

/// Options for getting trade history.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetTradeHistoryOptions {
    /// The max trades to request.
    pub max_trades: u32,
    /// Get trades that start after this time.
    pub start_after_time: Option<ServerTime>,
    /// Get trades that start after this tradeid.
    pub start_after_tradeid: Option<TradeId>,
    /// Whether we are navigating back or not.
    pub navigating_back: bool,
    /// Include failed traes.
    pub include_failed: bool,
}