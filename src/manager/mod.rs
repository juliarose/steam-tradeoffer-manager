mod builder;
pub(crate) mod polling;

pub use builder::TradeOfferManagerBuilder;
use polling::{Polling, PollOptions, PollReceiver, PollSender};

use crate::api::request::GetTradeOffersOptions;
use crate::time;
use crate::types::ServerTime;
use crate::api::SteamTradeOfferAPI;
use crate::mobile_api::MobileAPI;
use crate::static_functions::get_api_key;
use crate::helpers::get_default_client;
use crate::error::{Error, ParameterError, SetCookiesError};
use crate::request::{NewTradeOffer, GetTradeHistoryOptions};
use crate::enums::{TradeOfferState, OfferFilter, GetUserDetailsMethod};
use crate::types::{AppId, ContextId, TradeOfferId};
use crate::response::{UserDetails, Asset, SentOffer, TradeOffer, AcceptedOffer, Confirmation, Trades};
use std::sync::{Arc, Mutex, RwLock};
use steamid_ng::SteamID;
use tokio::task::JoinHandle;

/// Manager which includes functionality for interacting with trade offers, confirmations and
/// inventories.
#[derive(Debug, Clone)]
pub struct TradeOfferManager {
    /// The underlying API.
    api: SteamTradeOfferAPI,
    /// The underlying API for mobile confirmations.
    mobile_api: MobileAPI,
    /// The task handle for polling offers.
    polling: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl TradeOfferManager {
    /// Builder for constructing a [`TradeOfferManager`].
    pub fn builder() -> TradeOfferManagerBuilder {
        TradeOfferManagerBuilder::new()
    }
    
    /// Gets your Steam Web API key.
    /// 
    /// This method requires your cookies. If your account does not have an API key set, one will
    /// be created using `localhost` as the domain. By calling this method you are agreeing to the
    /// [Steam Web API Terms of Use](https://steamcommunity.com/dev/apiterms). 
    /// 
    /// # Examples
    /// ```no_run
    /// use steam_tradeoffer_manager::TradeOfferManager;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     // You'll need to use your own cookies here.
    ///     let cookies = vec![
    ///         "sessionid=blahblahblah".to_string(),
    ///         "steamLoginSecure=blahblahblah".to_string(),
    ///     ];
    ///     let api_key = TradeOfferManager::get_api_key(&cookies).await.unwrap();
    ///     
    ///     println!("Your API key is: {api_key}");
    /// }
    /// ```
    pub async fn get_api_key(
        cookies: &[String],
    ) -> Result<String, Error> {
        get_api_key(cookies).await
    }
    
    /// Sets cookies.
    /// 
    /// Some features will only work if cookies are set, such as sending or responding to trade
    /// offers. Make sure your cookies are set before calling these methods.
    /// 
    /// # Errors
    /// If the cookies do not contain a `steamLoginSecure` cookie that includes an access token.
    /// 
    /// # Examples
    /// ```no_run
    /// use steam_tradeoffer_manager::TradeOfferManager;
    /// 
    /// let manager = TradeOfferManager::builder().build();
    /// let cookies = vec![
    ///     "sessionid=blahblahblah".to_string(),
    ///     "steamLoginSecure=blahblahblah".to_string(),
    /// ]; 
    /// 
    /// if let Err(error) = manager.set_cookies(cookies) {
    ///     println!("Error setting cookies: {error}");
    /// }
    /// ```
    pub fn set_cookies(
        &self,
        cookies: Vec<String>,
    ) -> Result<(), SetCookiesError> {
        self.api.set_cookies(cookies.clone())?;
        self.mobile_api.set_cookies(cookies)?;
        Ok(())
    }
    
    /// Gets the logged-in user's [`SteamID`].
    /// 
    /// # Errors
    /// If the cookies are not set. (See [`TradeOfferManager::set_cookies`])
    pub fn get_steamid(
        &self,
    ) -> Result<SteamID, Error> {
        self.mobile_api.get_steamid()
    }
    
    /// Starts polling offers. Listen to the returned receiver for events. Use the returned sender
    /// to send an action to the poller using [`steam_tradeoffer_manager::polling::PollAction`].
    /// 
    /// Call `stop_polling` to stop polling offers. Polling will also stop if either the receiver
    /// or this [`TradeOfferManager`] are dropped. If this method is called again, the previous
    /// polling task will be aborted.
    /// 
    /// # Examples
    /// ```no_run
    /// use steam_tradeoffer_manager::TradeOfferManager;
    /// use steam_tradeoffer_manager::enums::TradeOfferState;
    /// use steam_tradeoffer_manager::polling::{PollOptions, PollReceiver, Poll};
    /// 
    /// // Polls offers.
    /// async fn poll_offers(
    ///     manager: TradeOfferManager,
    ///     mut receiver: PollReceiver,
    /// ) {
    ///     while let Some(result) = receiver.recv().await {
    ///         match result {
    ///             Ok(offers) => on_poll(&manager, offers).await,
    ///             Err(error) => println!("Error encountered polling offers: {error}"),
    ///         }
    ///     }
    ///     
    ///     println!("Polling stopped");
    /// }
    /// 
    /// // Do something with offers.
    /// async fn on_poll(
    ///     manager: &TradeOfferManager,
    ///     offers: Poll, // Poll is an alias for Vec<(TradeOffer, Option<TradeOfferState>)>
    /// ) {
    ///     for (mut offer, _old_state) in offers {
    ///         let is_free_items = {
    ///             // Offer must be active.
    ///             offer.trade_offer_state == TradeOfferState::Active &&
    ///             // Offer must not be created by us.
    ///             !offer.is_our_offer && 
    ///             // Offer must not be giving items.
    ///             offer.items_to_give.is_empty()
    ///         };
    ///         
    ///         if is_free_items {
    ///             println!("{offer} is giving us free items - accepting");
    ///             
    ///             match manager.accept_offer(&mut offer).await {
    ///                 Ok(accepted_offer) => println!("{} Accepted", offer),
    ///                 Err(error) => println!("Error accepting {offer}: {error}"),
    ///             }
    ///         }
    ///     }
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let manager = TradeOfferManager::builder()
    ///         .api_key("00000000000000000000000000000000".to_string())
    ///         .cookies(vec![
    ///             "sessionid=blahblahblah".to_string(),
    ///             "steamLoginSecure=blahblahblah".to_string(),
    ///         ])
    ///         .build();
    ///     let (_sender, receiver) = manager.start_polling(PollOptions::default()).unwrap();
    ///     
    ///     // Cloning isn't necessary here, but if you need to use the manager elsewhere, you can
    ///     // clone it for each task. The state for each clone is shared.
    ///     tokio::spawn(poll_offers(manager.clone(), receiver));
    /// }
    /// ```
    /// 
    /// # Errors
    /// - If the API key is not set. (See [`TradeOfferManagerBuilder::get_api_key`])
    /// - If the cookies are not set. (See [`TradeOfferManager::set_cookies`])
    pub fn start_polling(
        &self,
        options: PollOptions,
    ) -> Result<(PollSender, PollReceiver), Error> {
        if self.api.api_key.is_none() && self.api.session.read().unwrap().is_none() {
            return Err(ParameterError::MissingApiKeyOrAccessToken.into());
        }
        
        let steamid = self.get_steamid()?;
        let mut polling = self.polling.lock().unwrap();
        
        if let Some(handle) = &*polling {
            // Abort the previous polling.
            handle.abort();
        }
        
        let Polling {
            sender,
            receiver,
            handle,
        } = Polling::new(
            steamid,
            self.api.clone(),
            options,
        );
        
        *polling = Some(handle);
        
        Ok((sender, receiver))
    }
    
    /// Stops polling.
    pub fn stop_polling(
        &self,
    ) {
        if let Ok(polling) = self.polling.lock() {
            if let Some(handle) = &*polling {
                handle.abort();
            }
        }
    }
    
    /// Accepts an offer. Updates the state of the offer upon success as long as it does not
    /// require mobile confirmation.
    /// 
    /// # Errors
    /// - If the offer is ours.
    /// - If the offer is not active.
    /// - Any other error encountered while performing requests.
    pub async fn accept_offer(
        &self,
        offer: &mut TradeOffer,
    ) -> Result<AcceptedOffer, Error> {
        // Offer must not be created by us.
        if offer.is_our_offer {
            return Err(ParameterError::CannotAcceptOfferWeCreated.into());
        }
        
        // Offer must be active to be accepted.
        if offer.trade_offer_state != TradeOfferState::Active {
            return Err(ParameterError::CannotAcceptOfferThatIsNotActive(offer.trade_offer_state).into());
        }
        
        let accepted_offer = self.api.accept_offer(offer.tradeofferid, offer.partner).await?;
        
        // This offer doesn't need confirmation, so we can update its state here. If the
        // accepted_offer returns without error and does not need confirmation, then we can
        // assume it was accepted.
        if !accepted_offer.needs_confirimation() {
            offer.trade_offer_state = TradeOfferState::Accepted;
        }
        
        Ok(accepted_offer)
    }
    
    /// Cancels an offer. Updates the state of the offer upon success.
    /// 
    /// # Errors
    /// - If the offer is not ours.
    /// - Any other error encountered while performing requests.
    pub async fn cancel_offer(
        &self,
        offer: &mut TradeOffer,
    ) -> Result<(), Error> {
        if !offer.is_our_offer {
            return Err(ParameterError::CannotCancelOfferWeDidNotCreate.into());
        }
        
        self.api.cancel_offer(offer.tradeofferid).await?;
        offer.trade_offer_state = TradeOfferState::Canceled;
        
        Ok(())
    }
    
    /// Declines an offer. Updates the state of the offer upon success.
    /// 
    /// # Errors
    /// - If the offer is ours.
    /// - Any other error encountered while performing requests.
    pub async fn decline_offer(
        &self,
        offer: &mut TradeOffer,
    ) -> Result<(), Error> {
        if offer.is_our_offer {
            return Err(ParameterError::CannotDeclineOfferWeCreated.into());
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
    
    /// Gets our inventory. This method **does not** include untradable items.
    /// 
    /// # Errors
    /// If the cookies are not set. (See [`TradeOfferManager::set_cookies`])
    pub async fn get_my_inventory(
        &self,
        appid: AppId,
        contextid: ContextId,
    ) -> Result<Vec<Asset>, Error> {
        let steamid = self.get_steamid()?;
        
        self.api.get_inventory(steamid, appid, contextid, true).await
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
    /// 
    /// This will load up the trade confirmations, find the confirmation for the trade offer, and
    /// confirm it.
    /// 
    /// # Errors
    /// - If no confirmation is found for the trade offer.
    /// - Any other error encountered while performing requests.
    pub async fn confirm_offer(
        &self,
        trade_offer: &TradeOffer,
    ) -> Result<(), Error> {
        self.confirm_offer_id(trade_offer.tradeofferid).await
    }
    
    /// Confirms a trade offer using its ID.
    /// 
    /// This will load up the trade confirmations, find the confirmation for the trade offer, and
    /// confirm it.
    /// 
    /// # Errors
    /// - If no confirmation is found for the trade offer.
    /// - Any other error encountered while performing requests.
    pub async fn confirm_offer_id(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<(), Error> {
        let confirmation = self.get_trade_confirmations().await?
            .into_iter()
            .find(|confirmation| confirmation.creator_id == tradeofferid);
        
        if let Some(confirmation) = confirmation {
            return self.accept_confirmation(&confirmation).await;
        }
        
        Err(Error::NoConfirmationForOffer(tradeofferid))
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
    /// 
    /// # Errors
    /// - If the offer is not in the accepted state.
    /// - If the offer does not have a trade ID.
    /// - Any other error encountered while performing requests.
    pub async fn get_receipt(
        &self,
        offer: &TradeOffer,
    ) -> Result<Vec<Asset>, Error> {
        if offer.trade_offer_state != TradeOfferState::Accepted {
            Err(ParameterError::NotInAcceptedState(offer.trade_offer_state).into())
        } else if offer.items_to_receive.is_empty() {
            Ok(Vec::new())
        } else if let Some(tradeid) = offer.tradeid {
            self.api.get_receipt(&tradeid).await
        } else {
            Err(ParameterError::MissingTradeId.into())
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
        let offers = self.api.get_trade_offers(&GetTradeOffersOptions {
            active_only: filter == OfferFilter::ActiveOnly,
            historical_only: filter == OfferFilter::HistoricalOnly,
            get_sent_offers: true,
            get_received_offers: true,
            get_descriptions: false,
            historical_cutoff,
        }).await?;
        
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
    
    /// Gets trade history.
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
            if let Some(handle) = &*polling {
                // Abort polling before dropping.
                handle.abort();
            }
        }
    }
}

impl From<TradeOfferManagerBuilder> for TradeOfferManager {
    fn from(builder: TradeOfferManagerBuilder) -> Self {
        let cookies = builder.cookie_jar
            .unwrap_or_default();
        let client = builder.client
            .unwrap_or_else(|| get_default_client(
                Arc::clone(&cookies),
                builder.user_agent,
            ));
        let classinfo_cache = builder.classinfo_cache.unwrap_or_default();
        let mut api_builder = SteamTradeOfferAPI::builder()
            .data_directory(builder.data_directory)
            .client(client.clone(), Arc::clone(&cookies))
            .language(builder.language)
            .get_inventory_page_size(builder.get_inventory_page_size)
            .classinfo_cache(classinfo_cache);
        let session = Arc::new(RwLock::new(None));
        
        if let Some(api_key) = builder.api_key {
            api_builder = api_builder.api_key(api_key);   
        }
        
        if let Some(access_token) = builder.access_token {
            api_builder = api_builder.access_token(access_token);
        }
        
        api_builder = api_builder.session(Arc::clone(&session));
        
        let mut mobile_api_builder = MobileAPI::builder()
            .client(client, cookies)
            .time_offset(builder.time_offset)
            .session(session);
        
        if let Some(identity_secret) = builder.identity_secret {
            mobile_api_builder = mobile_api_builder.identity_secret(identity_secret);
        }
        
        let manager = Self {
            api: api_builder.build(),
            mobile_api: mobile_api_builder.build(),
            polling: Arc::new(Mutex::new(None)),
        };
        
        if let Some(cookies) = builder.cookies {
            // We don't care if this fails.
            manager.set_cookies(cookies).ok();
        }
        
        manager
    }
}
