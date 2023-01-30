mod builder;
mod polling;
mod helpers;

use helpers::generate_sessionid;
use lazy_regex::regex_captures;
pub use polling::{PollAction, Poll, PollResult, PollType, PollOptions, PollData};
pub use builder::TradeOfferManagerBuilder;

use std::{sync::Mutex, path::PathBuf, sync::Arc};
use crate::{
    time,
    error::{ParameterError, Error},
    ServerTime,
    api::SteamTradeOfferAPI,
    helpers::get_default_middleware,
    request::{NewTradeOffer, GetTradeHistoryOptions},
    enums::TradeOfferState,
    mobile_api::MobileAPI,
    types::{AppId, ContextId, TradeOfferId},
    response::{UserDetails, Asset, SentOffer, TradeOffer, AcceptedOffer, Confirmation, Trades},
};
use steamid_ng::SteamID;
use tokio::{sync::mpsc, task::JoinHandle};
use reqwest::cookie::Jar;

pub const USER_AGENT_STRING: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.36";

type Polling = (mpsc::Sender<PollAction>, JoinHandle<()>);

/// Manager which includes functionality for interacting with trade offers, confirmations and 
/// inventories.
#[derive(Debug, Clone)]
pub struct TradeOfferManager {
    /// The account's SteamID.
    pub steamid: SteamID,
    /// The underlying API. Use this if you need more direct control over API calls.
    pub api: SteamTradeOfferAPI,
    /// The underlying API for mobile confirmations.
    mobile_api: MobileAPI,
    /// The directory to store poll data and [`crate::response::ClassInfo`] data.
    data_directory: PathBuf,
    /// The sender for sending messages to polling
    polling: Arc<Mutex<Option<Polling>>>,
}

impl TradeOfferManager {
    /// Creates a new [`TradeOfferManager`].
    pub fn new(
        steamid: SteamID,
        api_key: String,
        data_directory: PathBuf,
    ) -> Self {
        Self::builder(
            steamid,
            api_key,
            data_directory,
        ).build()
    }
    
    /// Builder for new manager.
    pub fn builder(
        steamid: SteamID,
        api_key: String,
        data_directory: PathBuf,
    ) -> TradeOfferManagerBuilder {
        TradeOfferManagerBuilder::new(
            steamid,
            api_key,
            data_directory,
        )
    }
    
    /// Sets cookies.
    /// 
    /// **IMPORTANT:** If you passed in a client to the builder for this manager but did not also 
    /// pass in the cookies connected to the client this method will effectively do nothing.
    pub fn set_cookies(
        &self,
        cookies: &[String],
    ) {
        let mut cookies = cookies.to_owned();
        let mut sessionid = None;
        
        for cookie in &cookies {
            if let Some((_, key, value)) = regex_captures!(r#"([^=]+)=(.+)"#, cookie) {
                if key == "sessionid" {
                    sessionid = Some(value.to_string());
                }
            }
        }
        
        let sessionid = if let Some(sessionid) = sessionid {
            sessionid
        } else {
            // the cookies don't contain a sessionid
            let sessionid = generate_sessionid();
            
            cookies.push(format!("sessionid={sessionid}"));
            sessionid
        };
        
        self.api.set_session(&sessionid, &cookies);
        self.mobile_api.set_session(&sessionid, &cookies);
    }
    
    /// Accepts an offer. This checks if the offer can be acted on and updates the state of the 
    /// offer upon success.
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
        
        let accepted_offer = self.api.accept_offer(offer.tradeofferid, &offer.partner).await?;
        offer.trade_offer_state = TradeOfferState::Accepted;
        
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
    
    /// Gets our nventory. This method **does not** to include untradable items.
    pub async fn get_my_inventory(
        &self,
        appid: AppId,
        contextid: ContextId,
    ) -> Result<Vec<Asset>, Error> {
        self.api.get_inventory(&self.steamid, appid, contextid, true).await
    }
    
    /// Gets a user's inventory. This method **does not** to include untradable items.
    pub async fn get_inventory(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
    ) -> Result<Vec<Asset>, Error> {
        self.api.get_inventory(steamid, appid, contextid, true).await
    }
    
    /// Gets a user's inventory including untradable items.
    pub async fn get_inventory_with_untradables(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
    ) -> Result<Vec<Asset>, Error> {
        self.api.get_inventory(steamid, appid, contextid, false).await
    }
    
    /// Gets escrow details for user.
    pub async fn get_user_details_with_tradeofferid(
        &self,
        partner: &SteamID,
        tradeofferid: TradeOfferId,
    ) -> Result<UserDetails, Error> {
        self.api.get_user_details(partner, Some(tradeofferid), &None).await
    }
    
    /// Gets escrow details for user.
    pub async fn get_user_details_with_access_token(
        &self,
        partner: &SteamID,
        token: &str,
    ) -> Result<UserDetails, Error> {
        self.api.get_user_details(partner, None, &Some(token.into())).await
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
            true,
            false,
            Some(historical_cutoff),
        ).await?;
        
        Ok(offers)
    }
    
    /// Gets trade offers. This will trim responses based on the filter. 
    pub async fn get_trade_offers(
        &self,
        active_only: bool,
        historical_only: bool,
        historical_cutoff: Option<ServerTime>,
    ) -> Result<Vec<TradeOffer>, Error> {
        let offers = self.api.get_trade_offers(
            active_only,
            historical_only,
            true,
            true,
            false,
            historical_cutoff,
        ).await?;
        
        // trim responses since these don't always return what we want
        Ok(if active_only {
            offers
                .into_iter()
                .filter(|offer| offer.trade_offer_state == TradeOfferState::Active)
                .collect::<_>()
        } else if historical_only {
            offers
                .into_iter()
                .filter(|offer| offer.trade_offer_state != TradeOfferState::Active)
                .collect::<_>()
        } else {
            offers
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
    
    /// Starts polling offers. Listen to the returned receiver for events. To stop polling simply 
    /// drop the receiver. If this method is called again the previous polling task will be 
    /// aborted.
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