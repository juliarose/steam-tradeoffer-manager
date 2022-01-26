mod trade_offer;
mod sent_offer;
mod classinfo;
mod asset;
mod user_details;
mod raw;
pub mod deserializers;

pub use user_details::UserDetails;
pub use asset::Asset;
pub use trade_offer::TradeOffer;
pub use sent_offer::SentOffer;
pub use classinfo::{
    ClassInfo,
    Action,
    Description,
    Tag,
    AppId,
    ClassId,
    InstanceId,
    ClassInfoMap,
    ClassInfoAppMap,
    ClassInfoClass,
    ClassInfoAppClass
};
pub use raw::{
    RawAsset,
    RawAssetOld,
    RawTradeOffer
};

pub type Inventory = Vec<Asset>;