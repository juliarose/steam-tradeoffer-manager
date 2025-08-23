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
//! - Loads descriptions (classinfos) for assets. Classinfos are cached to file and read when
//!   available. The manager holds a [Least frequently used (LFU) cache](https://en.wikipedia.org/wiki/Least_frequently_used)
//!   of classinfos in memory to reduce file reads.
//! - Uses [tokio](https://crates.io/crates/tokio) asynchronous runtime for performing polling.
//! - Trade items <em>blazingly fast!</em>
//! 
//! ## Usage
//! 
//! All tasks relating to trade offers can be interfaced through [`TradeOfferManager`]. If more
//! direct control is needed, the underlying API's can be found in [`api`] and [`mobile_api`].
//!
//! See [examples](https://github.com/juliarose/steam-tradeoffers/tree/main/examples).

#![warn(missing_docs)]
extern crate lazy_static;

// Internal modules
mod manager;
mod serialize;
mod helpers;
mod classinfo_cache;
mod time;
mod session;
mod static_functions;

// Public modules
pub mod error;
pub mod request;
pub mod response;
pub mod enums;
pub mod types;
pub mod api;
pub mod mobile_api;

// Re-exports for convenience
pub use static_functions::get_inventory;
pub use classinfo_cache::ClassInfoCache;
pub use manager::{TradeOfferManager, TradeOfferManagerBuilder};

// Polling-related exports in a dedicated submodule
pub mod polling {
    //! Models related to polling trade offers.
    pub use super::manager::polling::{
        Poll,
        Result,
        PollAction,
        PollType,
        PollOptions,
        PollReceiver,
        PollSender,
    };
}

// External crate re-exports
pub use reqwest;
pub use reqwest_middleware;
pub use chrono;
pub use steamid_ng;
pub use steamid_ng::SteamID;
pub use another_steam_totp::get_steam_server_time_offset;
