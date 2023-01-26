mod poll_data;
mod file;
mod builder;

pub use builder::TradeOfferManagerBuilder;
use poll_data::PollData;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
};
use chrono::Duration;
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
use tokio::task::JoinHandle;
use reqwest::cookie::Jar;

pub type Poll = Vec<(response::TradeOffer, Option<TradeOfferState>)>;
pub const USER_AGENT_STRING: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.36";

/// The type of poll to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollType {
    /// Let the manager decide. Unless you need to fetch offers in special cases this is what 
    /// should be used.
    Auto,
    /// Ideal and fastest method for obtaining offers when new offers have just been received.
    /// This will fetch only active offers and includes descriptions in the response rather than 
    /// relying on ISteamEconomy/GetAssetClassInfo. This will not update the timestamps in  the 
    /// poll data. For this reason, this should not be used as your only method of polling if you 
    /// care about checking the state of changed offers.
    NewOffers,
    /// Do a full update.
    FullUpdate,
}

impl PollType {
    /// The poll is a forced update.
    fn is_forced(&self) -> bool {
        match self {
            Self::NewOffers => true,
            _ => false,
        }
    }
    
    /// The poll is a full update.
    fn is_full_update(&self) -> bool {
        match self {
            Self::FullUpdate => true,
            _ => false,
        }
    }
    
    /// The poll is only active offers.
    fn is_active_only(&self) -> bool {
        match self {
            Self::NewOffers => true,
            _ => false,
        }
    }
    
    /// The poll is sent offers only.
    fn is_sent_only(&self) -> bool {
        false
    }
    
    /// The poll is received offers only.
    fn is_received_only(&self) -> bool {
        false
    }
}

/// Manager which includes functionality for interacting with trade offers, confirmations and 
/// inventories.
#[derive(Debug)]
pub struct TradeOfferManager {
    /// The account's SteamID.
    pub steamid: SteamID,
    pub cancel_duration: Option<Duration>,
    pub full_poll_update_duration: Duration,
    /// The underlying API.
    api: SteamTradeOfferAPI,
    /// The underlying API for mobile confirmations.
    mobile_api: MobileAPI,
    /// Account poll data.
    poll_data: Arc<tokio::sync::Mutex<PollData>>,
    /// The directory to store poll data and [`response::ClassInfo`] data.
    data_directory: PathBuf,
    /// The spawned task for polling offers.
    polling_handle: Option<JoinHandle<()>>,
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
    /// **IMPORTANT:** If you passed in a client but did not also pass in the cookies connected 
    /// to the client this method will effectively do nothing.
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
            &Some(historical_cutoff),
        ).await?;
        
        Ok(offers)
    }
    
    /// Gets trade offers. This will trim responses based on the filter. 
    pub async fn get_trade_offers(
        &self,
        filter: OfferFilter,
        historical_cutoff: &Option<ServerTime>,
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
    
    /// Performs a poll for changes to offers. Provides a parameter to determine what type of poll to perform.
    pub async fn do_poll(
        &self,
        poll_type: PollType,
    ) -> Result<Poll, Error> {
        // Concurrent mutex prevents poll spamming.
        let mut poll_data = self.poll_data.lock().await;
        
        poll_data.set_last_poll(time::get_server_time_now());
        
        let mut full_update = poll_type.is_full_update();
        let offers_since = if poll_type == PollType::NewOffers {
            // a very high date
            u32::MAX as i64
        } else if {
            // If we're doing a full update.
            full_update ||
            // Or the date of the last full poll is outdated.
            poll_data.last_poll_is_stale(&self.full_poll_update_duration)
        } {
            poll_data.set_last_poll_full_update(time::get_server_time_now());
            full_update = true;
            
            poll_data.oldest_active_offer
                // It looks like sometimes Steam can be dumb and backdate a modified offer.
                // We need to handle this. Let's add a 30-minute buffer.
                .map(|date| date.timestamp() - (60 * 30))
                .unwrap_or(1)
        } else {
            poll_data.offers_since
                // It looks like sometimes Steam can be dumb and backdate a modified offer.
                // We need to handle this. Let's add a 30-minute buffer.
                .map(|date| date.timestamp() - (60 * 30))
                .unwrap_or(1)
        };
        println!("poll full_update {} {:?}", full_update, poll_data.offers_since);
        let mut offers_since = time::timestamp_to_server_time(offers_since);
        let (mut offers, _descriptions) = self.api.get_raw_trade_offers(
            poll_type.is_active_only(),
            false,
            !poll_type.is_received_only(),
            !poll_type.is_sent_only(),
            false,
            &Some(offers_since.clone()),
        ).await?;
        // Vec of offers that were cancelled.
        let cancelled_offers = if let Some(cancel_duration) = self.cancel_duration {
            let cancel_time = chrono::Utc::now() - cancel_duration;
            // Cancels all offers older than cancel_time.
            let cancel_futures = offers
                .iter_mut()
                .filter(|offer| {
                    let is_active_state = {
                        offer.trade_offer_state == TradeOfferState::Active ||
                        offer.trade_offer_state == TradeOfferState::CreatedNeedsConfirmation
                    };
                    
                    is_active_state &&
                    offer.is_our_offer &&
                    offer.time_created < cancel_time
                })
                .map(|offer| self.api.cancel_offer(offer.tradeofferid))
                .collect::<Vec<_>>();
            
            futures::future::join_all(cancel_futures).await
                .into_iter()
                .filter_map(|offer| offer.ok())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        // For reducing file writes, keep track of whether the state of poll data has changed.
        let mut prev_states_map: HashMap<TradeOfferId, TradeOfferState> = HashMap::new();
        let mut poll: Vec<_> = Vec::new();
        let mut polled_oldest_active_offer = time::get_server_time_now();
        
        for mut offer in offers {
            // This offer was successfully cancelled above...
            // We need to update its state here.
            if cancelled_offers.contains(&offer.tradeofferid) {
                offer.trade_offer_state = TradeOfferState::Canceled;
            }
            
            // To optimize our full updates we detect the oldest offer whose state can be updated 
            // e.g. active, in escrow, or offers requiring mobile confirmations.
            if {
                full_update &&
                // If the state can change..
                offer.state_is_changeable() &&
                // Update if the time of the offer is older than the current oldest active offer.
                offer.time_created < polled_oldest_active_offer
            } {
                polled_oldest_active_offer = offer.time_created.clone();
            }
            
            // Just don't do anything with this offer.
            if offer.is_glitched() {
                continue;
            }
            
            // Update the offers_since to the most recent trade offer.;
            if offer.time_updated > offers_since {
                offers_since = offer.time_updated.clone();
            }
            
            match poll_data.state_map.get(&offer.tradeofferid) {
                // State has changed.
                Some(
                    poll_trade_offer_state
                ) if poll_trade_offer_state != &offer.trade_offer_state => {
                    prev_states_map.insert(offer.tradeofferid, *poll_trade_offer_state);
                    poll.push(offer);
                },
                // Nothing has changed...
                Some(_) => {},
                // This is a new offer
                None => poll.push(offer),
            }
        }
        
        if full_update {
            println!("new oldest active offer {:?}", polled_oldest_active_offer);
            poll_data.set_oldest_active_offer(polled_oldest_active_offer);
        }
        
        // Clear poll data offers otherwise this could expand infinitely.
        // Using a higher number than is removed so this process needs to run less frequently.
        // This could be better but it works.
        if poll_data.state_map.len() > 2500 {
            let mut tradeofferids = poll_data.state_map
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            
            // High to low.
            tradeofferids.sort_by(|a, b| b.cmp(a));
            
            let (
                _tradeofferids,
                tradeofferids_to_remove,
            ) = tradeofferids.split_at(2000);
            
            for tradeofferid in tradeofferids_to_remove {
                poll_data.state_map.remove(tradeofferid);
                poll_data.changed = true;
            }
        }
        
        poll_data.set_offers_since(offers_since);
        
        // Maps raw offers to offers with classinfo descriptions.
        let offers = self.api.map_raw_trade_offers(poll).await?;
        let poll = if offers.is_empty() {
            // map_raw_trade_offers may have excluded some offers - the state of the poll data
            // is not updated until all descriptions are loaded for the offer
            Vec::new()
        } else {
            poll_data.changed = true;
            offers
                .into_iter()
                // Combines changed state maps.
                .map(|offer| {
                    let prev_state = prev_states_map.remove(&offer.tradeofferid);
                    
                    // insert new state into map
                    poll_data.state_map.insert(offer.tradeofferid, offer.trade_offer_state.clone());
                    
                    (offer, prev_state)
                })
                .collect::<Vec<_>>()
        };
        
        // Only save if changes were detected.
        if poll_data.changed {
            poll_data.changed = false;
            let _ = file::save_poll_data(
                &self.steamid,
                &serde_json::to_string(&*poll_data)?,
                &self.data_directory,
            ).await;
        }
        
        Ok(poll)
    }
}

impl std::ops::Drop for TradeOfferManager {
    fn drop(&mut self) {
        if let Some(handle) = &self.polling_handle {
            // abort polling on drop
            handle.abort();
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
        let steamid = builder.steamid;
        let identity_secret = builder.identity_secret;
        let poll_data = file::load_poll_data(
            &steamid,
            &builder.data_directory,
        ).unwrap_or_else(|_| PollData::new());
        let language = builder.language;
        let mobile_api_client = client.clone();
        
        Self {
            steamid: builder.steamid,
            api: SteamTradeOfferAPI::new(
                client,
                Arc::clone(&cookies),
                steamid,
                builder.key,
                language.clone(),
                builder.classinfo_cache,
                builder.data_directory.clone(),
            ),
            mobile_api: MobileAPI::new(
                cookies,
                mobile_api_client,
                steamid,
                language,
                identity_secret,
            ),
            poll_data: Arc::new(tokio::sync::Mutex::new(poll_data)),
            cancel_duration: builder.cancel_duration,
            full_poll_update_duration: builder.full_poll_update_duration,
            data_directory: builder.data_directory,
            polling_handle: None,
        }
    }
}