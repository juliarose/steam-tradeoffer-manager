mod builder;
mod polling;

pub use polling::{PollAction, Poll, PollResult, PollType, PollOptions, PollData};
pub use builder::TradeOfferManagerBuilder;

use std::{sync::Mutex, path::PathBuf, sync::Arc};
use crate::{
    time,
    response,
    request,
    error::Error,
    ServerTime,
    api::SteamTradeOfferAPI,
    helpers::get_default_middleware,
    enums::{OfferFilter, TradeOfferState},
    mobile_api::{MobileAPI, Confirmation},
    types::{AppId, ContextId, TradeOfferId},
};
use steamid_ng::SteamID;
use url::ParseError;
use tokio::{sync::mpsc, task::JoinHandle};
use reqwest::cookie::Jar;

pub const USER_AGENT_STRING: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.36";

/// Manager which includes functionality for interacting with trade offers, confirmations and 
/// inventories.
#[derive(Debug, Clone)]
pub struct TradeOfferManager {
    /// The account's SteamID.
    pub steamid: SteamID,
    /// The underlying API.
    api: SteamTradeOfferAPI,
    /// The underlying API for mobile confirmations.
    mobile_api: MobileAPI,
    /// The directory to store poll data and [`response::ClassInfo`] data.
    data_directory: PathBuf,
    /// The sender for sending messages to polling
    polling: Arc<Mutex<Option<(mpsc::Sender<PollAction>, JoinHandle<()>)>>>,
}

impl TradeOfferManager {
    /// Creates a new [`TradeOfferManager`].
    pub fn new(
        steamid: SteamID,
        key: String,
    ) -> Self {
        Self::builder(steamid, key).build()
    }
    
    /// Builder for new manager.
    pub fn builder(
        steamid: SteamID,
        key: String,
    ) -> TradeOfferManagerBuilder {
        TradeOfferManagerBuilder::new(steamid, key)
    }
    
    /// Sets the session and cookies.
    /// 
    /// **IMPORTANT:** If you passed in a client to the builder for this manager but did not also 
    /// pass in the cookies connected to the client this method will effectively do nothing.
    pub fn set_session(
        &self,
        sessionid: &str,
        cookies: &Vec<String>,
    ) -> Result<(), ParseError> {
        self.api.set_session(sessionid, cookies)?;
        self.mobile_api.set_session(sessionid, cookies)?;
        
        Ok(())
    }
    
    /// Accepts an offer. This checks if the offer can be acted on and updates the state of the 
    /// offer upon success.
    pub async fn accept_offer(
        &self,
        offer: &mut response::TradeOffer,
    ) -> Result<response::AcceptedOffer, Error> {
        if offer.is_our_offer {
            return Err(Error::Parameter("Cannot accept an offer that is ours"));
        } else if offer.trade_offer_state != TradeOfferState::Active {
            return Err(Error::Parameter("Cannot accept an offer that is not active"));
        }
        
        let accepted_offer = self.api.accept_offer(offer.tradeofferid, &offer.partner).await?;
        offer.trade_offer_state = TradeOfferState::Accepted;
        
        Ok(accepted_offer)
    }
    
    /// Accepts an offer using its tradeofferid..
    pub async fn accept_offer_id(
        &self,
        tradeofferid: TradeOfferId,
        partner: &SteamID,
    ) -> Result<response::AcceptedOffer, Error> {
        let accepted_offer = self.api.accept_offer(tradeofferid, &partner).await?;
        
        Ok(accepted_offer)
    }
    
    /// Cancels an offer. This checks if the offer was not creating by us and updates the state of 
    /// the offer upon success.
    pub async fn cancel_offer(
        &self,
        offer: &mut response::TradeOffer,
    ) -> Result<(), Error> {
        if !offer.is_our_offer {
            return Err(Error::Parameter("Cannot cancel an offer we did not create"));
        }
        
        self.api.cancel_offer(offer.tradeofferid).await?;
        offer.trade_offer_state = TradeOfferState::Canceled;
        
        Ok(())
    }
    
    /// Cancels an offer using its tradeofferid.
    pub async fn cancel_offer_id(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<(), Error> {
        self.api.cancel_offer(tradeofferid).await?;
        
        Ok(())
    }
    
    /// Declines an offer. This checks if the offer was creating by us and updates the state of 
    /// the offer upon success.
    pub async fn decline_offer(
        &self,
        offer: &mut response::TradeOffer,
    ) -> Result<(), Error> {
        if offer.is_our_offer {
            return Err(Error::Parameter("Cannot decline an offer we created"));
        }
        
        self.api.decline_offer(offer.tradeofferid).await?;
        offer.trade_offer_state = TradeOfferState::Declined;
        
        Ok(())
    }
    
    /// Declines an offer using its tradeofferid.
    pub async fn decline_offer_id(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<(), Error> {
        self.api.decline_offer(tradeofferid).await?;
        
        Ok(())
    }
    
    /// Sends an offer.
    pub async fn send_offer(
        &self,
        offer: &request::trade_offer::NewTradeOffer,
    ) -> Result<response::SentOffer, Error> {
        self.api.send_offer(offer, None).await
    }
    
    /// Counters an existing offer. This updates the state of the offer upon success.
    pub async fn counter_offer(
        &self,
        offer: &mut response::TradeOffer,
        counter_offer: &request::trade_offer::NewTradeOffer,
    ) -> Result<response::SentOffer, Error> {
        let sent_offer = self.api.send_offer(
            counter_offer,
            Some(offer.tradeofferid),
        ).await?;
        
        offer.trade_offer_state = TradeOfferState::Countered;
        
        Ok(sent_offer)
    }
    
    /// Counters an existing offer using its tradeofferid.
    pub async fn counter_offer_id(
        &self,
        tradeofferid: TradeOfferId,
        counter_offer: &request::trade_offer::NewTradeOffer,
    ) -> Result<response::SentOffer, Error> {
        let sent_offer = self.api.send_offer(
            counter_offer,
            Some(tradeofferid),
        ).await?;
        
        Ok(sent_offer)
    }

    /// Gets a user's inventory using the old endpoint.
    pub async fn get_inventory_old(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::Asset>, Error> {
        self.api.get_inventory_old(steamid, appid, contextid, tradable_only).await
    }
    
    /// Gets a user's inventory.
    pub async fn get_inventory(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::Asset>, Error> {
        self.api.get_inventory(steamid, appid, contextid, tradable_only).await
    }
    
    /// Gets a user's inventory with more detailed clasinfo data using the GetAssetClassInfo API.
    pub async fn get_inventory_with_classinfos(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::Asset>, Error> {
        self.api.get_inventory_with_classinfos(steamid, appid, contextid, tradable_only).await
    }
    
    /// Gets the user's details for trading.
    pub async fn get_user_details(
        &self,
        tradeofferid: &Option<TradeOfferId>,
        partner: &SteamID,
        token: &Option<String>,
    ) -> Result<response::UserDetails, Error> {
        self.api.get_user_details(tradeofferid, partner, token).await
    }
    
    /// Gets trade confirmations.
    pub async fn get_trade_confirmations(
        &self,
    ) -> Result<Vec<Confirmation>, Error> {
        self.mobile_api.get_trade_confirmations().await
    }
    
    /// Confirms a trade offer.
    pub async fn confirm_offer(
        &self,
        trade_offer: &response::TradeOffer,
    ) -> Result<(), Error> {
        self.confirm_offer_id(trade_offer.tradeofferid).await
    }
    
    /// Confirms an trade offer using its ID.
    pub async fn confirm_offer_id(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<(), Error> {
        let confirmations = self.get_trade_confirmations().await?;
        let confirmation = confirmations
            .into_iter()
            .find(|confirmation| confirmation.creator == tradeofferid);
        
        if let Some(confirmation) = confirmation {
            self.accept_confirmation(&confirmation).await
        } else {
            Err(Error::NoConfirmationForOffer(tradeofferid))
        }
    }
    
    /// Accepts a confirmation.
    pub async fn accept_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.mobile_api.accept_confirmation(confirmation).await
    }
    
    /// Accepts confirmations.
    pub async fn accept_confirmations(
        &self,
        confirmations: &[Confirmation],
    ) -> Result<(), Error> {
        for confirmation in confirmations {
            self.mobile_api.accept_confirmation(confirmation).await?
        }
        
        Ok(())
    }
    
    /// Declines a confirmation.
    pub async fn decline_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.mobile_api.deny_confirmation(confirmation).await
    }
    
    /// Gets the trade receipt (new items) upon completion of a trade.
    pub async fn get_receipt(&self, offer: &response::TradeOffer) -> Result<Vec<response::Asset>, Error> {
        if offer.trade_offer_state != TradeOfferState::Accepted {
            Err(Error::Parameter(r#"Offer is not in "accepted" state"#))
        } else if offer.items_to_receive.is_empty() {
            Ok(Vec::new())
        } else if let Some(tradeid) = offer.tradeid {
            self.api.get_receipt(&tradeid).await
        } else {
            Err(Error::Parameter("Missing tradeid"))
        }
    }
    
    /// Updates the offer to the most recent state against the API.
    pub async fn update_offer(&self, offer: &mut response::TradeOffer) -> Result<(), Error> {
        let updated = self.api.get_trade_offer(offer.tradeofferid).await?;
        
        offer.tradeofferid = updated.tradeofferid;
        offer.tradeid = updated.tradeid;
        offer.trade_offer_state = updated.trade_offer_state;
        offer.confirmation_method = updated.confirmation_method;
        offer.escrow_end_date = updated.escrow_end_date;
        offer.time_created = updated.time_created;
        offer.time_updated = updated.time_updated;
        offer.expiration_time = updated.expiration_time;
        
        Ok(())
    }

    /// Gets active trade offers.
    pub async fn get_active_trade_offers(
        &self
    ) -> Result<Vec<response::TradeOffer>, Error> {
        let historical_cutoff = time::timestamp_to_server_time(u32::MAX as i64);
        let offers = self.get_trade_offers(
            OfferFilter::ActiveOnly,
            Some(historical_cutoff),
        ).await?;
        
        Ok(offers)
    }
    
    /// Gets trade offers. This will trim responses based on the filter. 
    pub async fn get_trade_offers(
        &self,
        filter: OfferFilter,
        historical_cutoff: Option<ServerTime>,
    ) -> Result<Vec<response::TradeOffer>, Error> {
        let active_only = filter == OfferFilter::ActiveOnly;
        let historical_only = filter == OfferFilter::HistoricalOnly;
        let offers = self.api.get_trade_offers(
            active_only,
            historical_only,
            true,
            true,
            false,
            historical_cutoff,
        ).await?;
        
        // trim responses since these don't always return what we want
        Ok(match filter {
            OfferFilter::All => offers,
            OfferFilter::ActiveOnly => {
                offers
                    .into_iter()
                    .filter(|offer| offer.trade_offer_state == TradeOfferState::Active)
                    .collect::<_>()
            },
            OfferFilter::HistoricalOnly => {
                offers
                    .into_iter()
                    .filter(|offer| offer.trade_offer_state != TradeOfferState::Active)
                    .collect::<_>()
            },
        })
    }
    
    /// Starts polling. Listen to the receiver for events. If the receiver is dropped polling will 
    /// stop. If this method is called again polling the previous polling will also stop.
    pub fn start_polling(
        &self,
        options: PollOptions,
    ) -> mpsc::Receiver<PollResult> {
        let mut polling = self.polling.lock().unwrap();
        
        if let Some((_, handle)) = &*polling {
            // Abort the previous polling.
            handle.abort();
        }
        
        let (
            tx,
            rx,
            handle,
        ) = polling::create_poller(
            self.api.clone(),
            self.data_directory.clone(),
            options,
        );
        
        *polling = Some((tx, handle));
        
        rx
    }
    
    /// Sends a message to the poller to do a poll now. Returns an error if polling is not setup.
    /// Remember to start polling using the [`start_polling`] method before calling this method.
    /// The message will be ignored if a message with the same [`poll_type`] was sent within the 
    /// last half a second.
    pub fn do_poll(
        &self,
        poll_type: PollType,
    ) -> Result<(), Error> {
        use tokio::sync::mpsc::error::TrySendError;
        
        if let Some((sender, _)) = &*self.polling.lock().unwrap() {
            sender.try_send(PollAction::DoPoll(poll_type))
                .map_err(|error| match error {
                    TrySendError::Full(_) => Error::PollingBufferFull,
                    // Probably should happen, but if it does the handle was closed.
                    TrySendError::Closed(_) => Error::PollingNotSetup,
                })?;
            
            Ok(())
        } else {
            Err(Error::PollingNotSetup)
        }
    }
}

impl std::ops::Drop for TradeOfferManager {
    fn drop(&mut self) {
        if let Ok(polling) = self.polling.lock() {
            if let Some((_sender, handle)) = &*polling {
                // Abort polling before dropping.
                handle.abort();
            }
        }
    }
}

impl From<TradeOfferManagerBuilder> for TradeOfferManager {
    fn from(builder: TradeOfferManagerBuilder) -> Self {
        let cookies = builder.cookies.unwrap_or_else(|| Arc::new(Jar::default()));
        let client = builder.client.unwrap_or_else(|| {
            get_default_middleware(
                Arc::clone(&cookies),
                builder.user_agent,
            )
        });
        
        Self {
            steamid: builder.steamid,
            api: SteamTradeOfferAPI::new(
                client.clone(),
                Arc::clone(&cookies),
                builder.steamid,
                builder.api_key,
                builder.language.clone(),
                builder.classinfo_cache,
                builder.data_directory.clone(),
            ),
            mobile_api: MobileAPI::new(
                cookies,
                client,
                builder.steamid,
                builder.language.clone(),
                builder.identity_secret,
            ),
            data_directory: builder.data_directory,
            polling: Arc::new(Mutex::new(None)),
        }
    }
}