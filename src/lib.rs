//! # steam-tradeoffer-manager
//! 
//! Makes Steam trade offers easy!
//! 
//! Based on the excellent [node-steam-tradeoffer-manager](https://github.com/DoctorMcKay/node-steam-tradeoffer-manager).
//! 
//! ## Features
//! 
//! - Richly-featured API for creating, accepting, cancelling, and declining trade offers.
//! - Manages account trade offer state.
//! - Mobile confirmations.
//! - Loading inventories.
//! - Trade history.
//! - Helper method for getting your Steam Web API key.
//! - Automatically cancels offers past a set duration during polls.
//! - Loads descriptions (classinfos) for assets. Classinfos are cached to file and read when available. The manager holds a [Least frequently used (LFU) cache](https://en.wikipedia.org/wiki/Least_frequently_used) of classinfos in memory to reduce file reads.
//! - Uses [tokio](https://crates.io/crates/tokio) asynchronous runtime for performing polling.
//! - Trade items <em>blazingly fast!</em>
//! 
//! ## Usage
//!
//! See [examples](https://github.com/juliarose/steam-tradeoffers/tree/main/examples).
//! 
//! ## Conventions
//! 
//! For the most part everything is straight-forward. You can find response structs in `response`, 
//! enums in `enums`, request parameter structs in `request`, errors in `error`, and types used
//! throughout are found in `types`.
//! 
//! For the most part all tasks relating to trade offers can be interfaced through 
//! [`TradeOfferManager`]. If more direct control is needed, the underlying API can be found in 
//! [`api`] and is also accessible as `api` on [`TradeOfferManager`] instances.

extern crate lazy_static;

mod manager;
mod serialize;
mod mobile_api;
mod helpers;
mod classinfo_cache;
mod time;
mod internal_types;
mod static_functions;

pub mod api;
pub mod enums;
pub mod types;
pub mod request;
pub mod response;
pub mod error;

pub use static_functions::get_inventory;
pub use classinfo_cache::ClassInfoCache;
pub use time::ServerTime;
pub use manager::{TradeOfferManager, TradeOfferManagerBuilder};

pub mod polling {
    //! Contains models related to polling trade offers.
    pub use super::manager::{Poll, PollResult, PollAction, PollType, PollOptions};
}

pub use reqwest;
pub use reqwest_middleware;
pub use chrono;
pub use steamid_ng;
/// Re-export from [`steamid_ng`]. A Steam ID.
pub use steamid_ng::SteamID;