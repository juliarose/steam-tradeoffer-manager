//! # steam-tradeoffer-manager
//! 
//! Makes Steam trade offers easy!
//! 
//! Based on the excellent [`node-steam-tradeoffer-manager`](https://github.com/DoctorMcKay/node-steam-tradeoffer-manager).
//! 
//! ## Features
//! 
//! - Richly-featured API for creating, accepting, cancelling, and declining trade offers.
//! - Manages account trade offer state.
//! - Mobile confirmations.
//! - Loading inventories.
//! - Trade history.
//! - Automatically cancels offers past a set duration during polls.
//! - Loads descriptions (classinfos) for assets. Classinfos are cached to file and read when available. The manager holds a [Least frequently used (LFU) cache](https://en.wikipedia.org/wiki/Least_frequently_used) of classinfos in memory to reduce file reads.
//! - Uses [tokio](https://crates.io/crates/tokio) asynchronous runtime for performing polling.
//! - And of course the blazingly fast performance and data integrity you'd expect from Rust.
//! 
//! ## Usage
//!
//! See [examples](https://github.com/juliarose/steam-tradeoffers/tree/main/examples).

mod manager;
mod serialize;
mod mobile_api;
mod helpers;
mod classinfo_cache;
mod time;

pub mod api;
pub mod enums;
pub mod types;
pub mod request;
pub mod response;
pub mod error;

pub use classinfo_cache::ClassInfoCache;
pub use time::ServerTime;
pub use manager::{
    TradeOfferManager,
    TradeOfferManagerBuilder,
};

pub mod polling {
    pub use super::manager::{Poll, PollResult, PollType, PollOptions};
}

pub use reqwest;
pub use reqwest_middleware;
pub use chrono;
pub use steamid_ng::{self, SteamID};