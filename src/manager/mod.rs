mod builder;
mod polling;

pub use polling::{PollingMpsc, PollAction, Poll, PollResult, PollType, PollOptions, PollData};
pub use builder::TradeOfferManagerBuilder;

use crate::time;
use crate::ServerTime;
use crate::api::SteamTradeOfferAPI;
use crate::mobile_api::MobileAPI;
use crate::static_functions::get_api_key;
use crate::helpers::{generate_sessionid, get_default_middleware, get_sessionid_and_steamid_from_cookies};
use crate::error::{ParameterError, Error};
use crate::request::{NewTradeOffer, GetTradeHistoryOptions};
use crate::enums::{TradeOfferState, OfferFilter, GetUserDetailsMethod};
use crate::types::{AppId, ContextId, TradeOfferId};
use crate::response::{UserDetails, Asset, SentOffer, TradeOffer, AcceptedOffer, Confirmation, Trades};
use std::sync::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicU64};
use steamid_ng::SteamID;
use tokio::{sync::mpsc, task::JoinHandle};
use reqwest::cookie::Jar;

type Polling = (mpsc::Sender<PollAction>, JoinHandle<()>);

/// Manager which includes functionality for interacting with trade offers, confirmations and 
/// inventories.
#[derive(Debug, Clone)]
pub struct TradeOfferManager {
    /// The underlying API. The methods on [`TradeOfferManager`] only include more conventional 
    /// ease-of-use methods. Use this API if you have a more specific use-case.
    pub api: SteamTradeOfferAPI,
    /// The underlying API for mobile confirmations.
    mobile_api: MobileAPI,
    /// The account's SteamID.
    steamid: Arc<AtomicU64>,
    /// The directory to store poll data and classinfo data.
    data_directory: PathBuf,
    /// The sender for sending messages to polling
    polling: Arc<Mutex<Option<Polling>>>,
}

impl TradeOfferManager {
    /// Creates a new [`TradeOfferManager`]. Requires an `api_key` for making API calls and a 
    /// `data_directory` for storing poll data and classinfo caches.
    pub fn new<T>(
        api_key: String,
        data_directory: T,
    ) -> Self
    where
        T: Into<PathBuf>,
    {
        Self::builder(
            api_key,
            data_directory,
        ).build()
    }
    
    /// Builder for constructing a [`TradeOfferManager`]. Requires an `api_key` for making API 
    /// calls and a `data_directory` for storing poll data and classinfo caches.
    pub fn builder<T>(
        api_key: String,
        data_directory: T,
    ) -> TradeOfferManagerBuilder
    where
        T: Into<PathBuf>,
    {
        TradeOfferManagerBuilder::new(
            api_key,
            data_directory,
        )
    }
    
    /// Gets your Steam Web API key. This method requires your cookies. If your account does not have
    /// an API key set, one will be created using `localhost` as the domain. By calling this method you
    /// are agreeing to the [Steam Web API Terms of Use](https://steamcommunity.com/dev/apiterms). 
    pub async fn get_api_key(
        cookies: &[String],
    ) -> Result<String, Error> {
        get_api_key(cookies).await
    }
    
    /// Sets cookies.
    pub fn set_cookies(
        &self,
        cookies: &[String],
    ) {
        let (
            sessionid,
            steamid,
        ) = get_sessionid_and_steamid_from_cookies(cookies);
        let mut cookies = cookies.to_owned();
        
        if sessionid.is_none() {
            // the cookies don't contain a sessionid
            let sessionid = generate_sessionid();
            
            cookies.push(format!("sessionid={sessionid}"));
        }
        
        if let Some(steamid) = steamid {
            self.steamid.store(steamid, Ordering::Relaxed);
        }
        
        self.api.set_cookies(&cookies);
        self.mobile_api.set_cookies(&cookies);
    }
    
    /// Gets the logged-in user's [`SteamID`].
    /// 
    /// Fails if no login is detected (cookies must be set first).
    pub fn get_steamid(
        &self,
    ) -> Result<SteamID, Error> {
        let steamid_64 = self.steamid.load(Ordering::Relaxed);
        
        if steamid_64 == 0 {
            return Err(Error::NotLoggedIn);
        }
        
        Ok(SteamID::from(steamid_64))
    }
    
    /// Starts polling offers. Listen to the returned receiver for events. To stop polling simply 
    /// drop the receiver. If this method is called again the previous polling task will be 
    /// aborted.
    /// 
    /// Fails if you are not logged in. Make sure to set your cookies before using this method.
    pub fn start_polling(
        &self,
        options: PollOptions,
    ) -> Result<mpsc::Receiver<PollResult>, Error> {
        let steamid = self.get_steamid()?;
        let mut polling = self.polling.lock().unwrap();
        
        if let Some((_, handle)) = &*polling {
            // Abort the previous polling.
            handle.abort();
        }
        
        let PollingMpsc {
            sender,
            receiver,
            handle,
        } = polling::create_poller(
            steamid,
            self.api.clone(),
            self.data_directory.clone(),
            options,
        );
        
        *polling = Some((sender, handle));
        
        Ok(receiver)
    }
    
    /// Sends a message to the poller to do a poll now. Returns an error if polling is not setup.
    /// Remember to start polling using the `start_polling` method before calling this method.
    /// The message will be ignored if a message with the same [`PollType`] was sent within the 
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
    
    /// Accepts an offer. This checks if the offer can be acted on and updates the state of the 
    /// offer upon success as long as it does not require mobile confirmation.
    pub async fn accept_offer(
        &self,
        offer: &mut TradeOffer,
    ) -> Result<AcceptedOffer, Error> {
        if offer.is_our_offer {
            return Err(Error::Parameter(
                ParameterError::CannotAcceptOfferThatIsOurs
            ));
        } else if offer.trade_offer_state != TradeOfferState::Active {
            return Err(Error::Parameter(
                ParameterError::CannotAcceptOfferThatIsNotActive(offer.trade_offer_state)
            ));
        }
        
        let accepted_offer = self.api.accept_offer(offer.tradeofferid, offer.partner).await?;
        
        // This offer doesn't need confirmation, so we can update its state here.
        if !accepted_offer.needs_confirimation() {
            offer.trade_offer_state = TradeOfferState::Accepted;
        }
        
        Ok(accepted_offer)
    }
    
    /// Cancels an offer. This checks if the offer was not creating by us and updates the state of 
    /// the offer upon success.
    pub async fn cancel_offer(
        &self,
        offer: &mut TradeOffer,
    ) -> Result<(), Error> {
        if !offer.is_our_offer {
            return Err(Error::Parameter(
                ParameterError::CannotCancelOfferWeDidNotCreate
            ));
        }
        
        self.api.cancel_offer(offer.tradeofferid).await?;
        offer.trade_offer_state = TradeOfferState::Canceled;
        
        Ok(())
    }
    
    /// Declines an offer. This checks if the offer was creating by us and updates the state of 
    /// the offer upon success.
    pub async fn decline_offer(
        &self,
        offer: &mut TradeOffer,
    ) -> Result<(), Error> {
        if offer.is_our_offer {
            return Err(Error::Parameter(
                ParameterError::CannotDeclineOfferWeCreated
            ));
        }
        
        self.api.decline_offer(offer.tradeofferid).await?;
        offer.trade_offer_state = TradeOfferState::Declined;
        
        Ok(())
    }
    
    /// Sends an offer.
    pub async fn send_offer(
        &self,
        offer: &NewTradeOffer,
    ) -> Result<SentOffer, Error> {
        self.api.send_offer(offer, None).await
    }
    
    /// Counters an existing offer. This updates the state of the offer upon success.
    pub async fn counter_offer(
        &self,
        offer: &mut TradeOffer,
        counter_offer: &NewTradeOffer,
    ) -> Result<SentOffer, Error> {
        let sent_offer = self.api.send_offer(
            counter_offer,
            Some(offer.tradeofferid),
        ).await?;
        
        offer.trade_offer_state = TradeOfferState::Countered;
        
        Ok(sent_offer)
    }
    
    /// Gets our nventory. This method **does not** include untradable items.
    pub async fn get_my_inventory(
        &self,
        appid: AppId,
        contextid: ContextId,
    ) -> Result<Vec<Asset>, Error> {
        let steamid_64 = self.steamid.load(Ordering::Relaxed);
        
        if steamid_64 == 0 {
            return Err(Error::NotLoggedIn);
        }
        
        self.api.get_inventory(SteamID::from(steamid_64), appid, contextid, true).await
    }
    
    /// Gets a user's inventory. This method **does not** include untradable items.
    pub async fn get_inventory(
        &self,
        steamid: SteamID,
        appid: AppId,
        contextid: ContextId,
    ) -> Result<Vec<Asset>, Error> {
        self.api.get_inventory(steamid, appid, contextid, true).await
    }
    
    /// Gets a user's inventory including untradable items.
    pub async fn get_inventory_with_untradables(
        &self,
        steamid: SteamID,
        appid: AppId,
        contextid: ContextId,
    ) -> Result<Vec<Asset>, Error> {
        self.api.get_inventory(steamid, appid, contextid, false).await
    }
    
    /// Gets escrow details for a user. The `method` for obtaining details can be a `tradeofferid` 
    /// or `access_token` or neither.
    pub async fn get_user_details<T>(
        &self,
        partner: SteamID,
        method: T,
    ) -> Result<UserDetails, Error> 
        where T: Into<GetUserDetailsMethod>,
    {
        self.api.get_user_details(partner, method).await
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
        trade_offer: &TradeOffer,
    ) -> Result<(), Error> {
        self.confirm_offer_id(trade_offer.tradeofferid).await
    }
    
    /// Confirms a trade offer using its ID.
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
    
    /// Cancels a confirmation.
    pub async fn cancel_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.mobile_api.cancel_confirmation(confirmation).await
    }
    
    /// Gets the trade receipt (new items) upon completion of a trade.
    pub async fn get_receipt(
        &self,
        offer: &TradeOffer,
    ) -> Result<Vec<Asset>, Error> {
        if offer.trade_offer_state != TradeOfferState::Accepted {
            Err(Error::Parameter(
                ParameterError::NotInAcceptedState(offer.trade_offer_state)
            ))
        } else if offer.items_to_receive.is_empty() {
            Ok(Vec::new())
        } else if let Some(tradeid) = offer.tradeid {
            self.api.get_receipt(&tradeid).await
        } else {
            Err(Error::Parameter(
                ParameterError::MissingTradeId
            ))
        }
    }
    
    /// Updates the offer to the most recent state against the API.
    pub async fn update_offer(
        &self,
        offer: &mut TradeOffer,
    ) -> Result<(), Error> {
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
    ) -> Result<Vec<TradeOffer>, Error> {
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
    ) -> Result<Vec<TradeOffer>, Error> {
        let offers = self.api.get_trade_offers(
            filter == OfferFilter::ActiveOnly,
            filter == OfferFilter::HistoricalOnly,
            true,
            true,
            false,
            historical_cutoff,
        ).await?;
        
        // trim responses since these don't always return what we want
        Ok(match filter {
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
            OfferFilter::All => {
                offers
            },
        })
    }
    
    /// Gets trade history. The second part of the returned tuple is whether more trades can be 
    /// fetched.
    pub async fn get_trade_history(
        &self,
        options: &GetTradeHistoryOptions,
    ) -> Result<Trades, Error> {
        self.api.get_trade_history(options).await
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
        let cookies = builder.cookies
            .unwrap_or_else(|| Arc::new(Jar::default()));
        let client = builder.client
            .unwrap_or_else(|| get_default_middleware(
                Arc::clone(&cookies),
                builder.user_agent,
            ));
        let steamid = Arc::new(AtomicU64::new(0));
        let api = SteamTradeOfferAPI::builder(
            builder.api_key,
            builder.data_directory.clone()
        )
            .language(builder.language)
            .classinfo_cache(builder.classinfo_cache)
            .client(client.clone(), Arc::clone(&cookies))
            .build();
        let mut mobile_api_builder = MobileAPI::builder()
            .client(client, cookies)
            .time_offset(builder.time_offset);
        
        if let Some(identity_secret) = builder.identity_secret {
            mobile_api_builder = mobile_api_builder.identity_secret(identity_secret);
        }
        
        let mobile_api = mobile_api_builder.build();
        
        Self {
            steamid: Arc::clone(&steamid),
            api,
            mobile_api,
            data_directory: builder.data_directory,
            polling: Arc::new(Mutex::new(None)),
        }
    }
}