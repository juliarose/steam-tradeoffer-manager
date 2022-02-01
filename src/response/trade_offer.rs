use std::fmt;
use crate::{
    time::ServerTime,
    TradeOfferState,
    ConfirmationMethod,
    classinfo_cache::ClassInfoCache,
    MissingClassInfoError,
    types::TradeOfferId
};
use super::{
    Asset,
    raw::{
        RawAsset,
        RawTradeOffer
    }
};
use steamid_ng::SteamID;

#[derive(Debug)]
pub struct TradeOffer {
    pub tradeofferid: TradeOfferId,
    pub partner: SteamID,
    pub message: Option<String>,
    pub items_to_receive: Vec<Asset>,
    pub items_to_give: Vec<Asset>,
    pub is_our_offer: bool,
    pub from_real_time_trade: bool,
    pub expiration_time: ServerTime,
    pub time_created: ServerTime,
    pub time_updated: ServerTime,
    pub trade_offer_state: TradeOfferState,
    pub escrow_end_date: ServerTime,
    pub confirmation_method: ConfirmationMethod,
}

impl fmt::Display for TradeOffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}:{}]", u64::from(self.partner), self.tradeofferid)
    }
}

impl TradeOffer {
    pub fn is_glitched(&self) -> bool {
        self.items_to_receive.len() == 0 && self.items_to_receive.len() == 0
    }

    pub fn from(offer: RawTradeOffer, cache: &ClassInfoCache) -> Result<Self, MissingClassInfoError> {
        fn collect_items(assets: Vec<RawAsset>, cache: &ClassInfoCache) -> Result<Vec<Asset>, MissingClassInfoError> {
            let mut items = Vec::new();
            
            for asset in assets {
                if let Some(classinfo) = cache.get_classinfo(&(asset.appid, asset.classid, asset.instanceid)) {
                    items.push(Asset {
                        classinfo,
                        appid: asset.appid,
                        contextid: asset.contextid,
                        assetid: asset.assetid,
                        amount: asset.amount,
                    });
                } else {
                    // todo use a less broad error for this
                    return Err(MissingClassInfoError {
                        appid: asset.appid,
                        classid: asset.classid,
                        instanceid: asset.instanceid,
                    });
                }
            }
            
            Ok(items)
        }
        
        fn steamid_from_accountid(accountid: u32) -> SteamID {
            SteamID::new(
                accountid,
                steamid_ng::Instance::Desktop,
                steamid_ng::AccountType::Individual,
                steamid_ng::Universe::Public
            )
        }
        
        let items_to_give = collect_items(offer.items_to_give, cache)?;
        let items_to_receive = collect_items(offer.items_to_receive, cache)?;
        
        Ok(Self {
            items_to_give,
            items_to_receive,
            tradeofferid: offer.tradeofferid,
            trade_offer_state: offer.trade_offer_state,
            partner: steamid_from_accountid(offer.accountid_other),
            message: offer.message,
            is_our_offer: offer.is_our_offer,
            from_real_time_trade: offer.from_real_time_trade,
            expiration_time: offer.expiration_time,
            time_updated: offer.time_updated,
            time_created: offer.time_created,
            escrow_end_date: offer.escrow_end_date,
            confirmation_method: offer.confirmation_method,
        })
    }

    // pub async fn accept(&'a self) -> Result<response::AcceptedOffer, APIError> {
    //     if self.is_our_offer {
    //         return Err(APIError::ParameterError("Cannot accept an offer that is ours"));
    //     } else if self.trade_offer_state != TradeOfferState::Active {
    //         return Err(APIError::ParameterError("Cannot accept an offer that is not active"));
    //     }

    //     self.api.accept_offer(self.tradeofferid, &self.partner).await
    // }
    
    // pub async fn cancel(&'a self) -> Result<(), APIError> {
    //     if !self.is_our_offer {
    //         return Err(APIError::ParameterError("Cannot cancel an offer we did not create"));
    //     }
        
    //     self.api.cancel_offer(self.tradeofferid).await
    // }
    
    // pub async fn decline(&'a self) -> Result<(), APIError> {
    //     if self.is_our_offer {
    //         return Err(APIError::ParameterError("Cannot decline an offer we created"));
    //     }
        
    //     self.api.decline_offer(self.tradeofferid).await
    // }

    // pub async fn update(&'a mut self) -> Result<(), APIError> {
    //     let offer = self.api.get_trade_offer(self.tradeofferid).await?;

    //     self.trade_offer_state = offer.trade_offer_state;
    //     self.time_updated = offer.time_updated;
    //     self.expiration_time = offer.expiration_time;
    //     self.escrow_end_date = offer.escrow_end_date;
    //     self.confirmation_method = offer.confirmation_method;

    //     Ok(())
    // }
}