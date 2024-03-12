//! Models for making requests.

use crate::types::{ServerTime, TradeId};

/// Options for getting trade offers.
#[derive(Debug, Clone)]
pub struct GetTradeOffersOptions {
    /// Whether to get only active trade offers.
    pub active_only: bool,
    /// Whether to get only historical trade offers.
    pub historical_only: bool,
    /// Whether to get sent trade offers.
    pub get_sent_offers: bool,
    /// Whether to get received trade offers.
    pub get_received_offers: bool,
    /// Whether to get descriptions for items in the trade offers.
    pub get_descriptions: bool,
    /// The time to get trade offers from.
    pub historical_cutoff: Option<ServerTime>,
}

/// Options for getting trade history.
pub(crate) struct GetTradeHistoryRequestOptions {
    /// The number of trades to get.
    pub max_trades: u32,
    /// The time to start getting trades after.
    pub start_after_time: Option<ServerTime>,
    /// The trade ID to start getting trades after.
    pub start_after_tradeid: Option<TradeId>,
    /// Whether we are navigating backwards in the trade history.
    pub navigating_back: bool,
    /// Whether to get descriptions for items in the trade history.
    pub get_descriptions: bool,
    /// Whether to include failed trades in the response.
    pub include_failed: bool,
    /// Whether to include the total number of trades in the response.
    pub include_total: bool,
}