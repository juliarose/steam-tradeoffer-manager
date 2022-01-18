mod trade_offer;
mod sent_offer;
mod classinfo;
mod asset;
mod user_details;
pub mod deserializers;

pub use user_details::UserDetails;
pub use asset::Asset;
pub use trade_offer::TradeOffer;
pub use sent_offer::SentOffer;
pub use classinfo::{ClassInfoMap, Action, ClassInfo, Description, Tag};

pub type Inventory = Vec<Asset>;