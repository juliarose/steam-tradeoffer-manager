mod file;
mod poll_data;

pub use poll_data::PollData;

use crate::{
    time,
    enums::TradeOfferState,
    types::TradeOfferId,
    response::TradeOffer,
    api::SteamTradeOfferAPI,
    error::Error,
};
use std::{
    path::PathBuf,
    collections::HashMap,
    sync::{atomic::{Ordering, AtomicBool}, Arc},
};
use chrono::{Duration, DateTime};
use steamid_ng::SteamID;
use tokio::{sync::{Mutex, mpsc}, task::JoinHandle};

/// Options for polling.
#[derive(Debug, Clone, Copy)]
pub struct PollOptions {
    /// The duration after a sent offer has been active to cancel during a poll. Offers will 
    /// not be cancelled if this is not set.
    pub cancel_duration: Option<Duration>,
    /// The duration after the last poll becomes stale and a new one must be obtained when 
    /// polling using [`crate::PollType::Auto`]. Default is 5 minutes.
    pub full_poll_update_duration: Duration,
    /// Interval to poll at. Default is 30 seconds.
    pub poll_interval: Duration,
}

impl Default for PollOptions {
    fn default() -> Self {
        Self {
            cancel_duration: None,
            full_poll_update_duration: Duration::minutes(5),
            poll_interval: Duration::seconds(30),
        }
    }
}

/// The type of poll to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PollType {
    /// Let the manager decide. Unless you need to fetch offers in special cases this is what 
    /// should be used.
    Auto,
    /// Fastest method for obtaining new offers when new offers. This will fetch only active 
    /// offers and includes descriptions in the response rather than relying on 
    /// ISteamEconomy/GetAssetClassInfo. For this reason, items in the response will also not 
    /// contain app_data. This will not update the timestamps in  the poll data. For this reason, 
    /// this should not be used as your only method of polling if you care about checking the 
    /// state of changed offers.
    NewOffers,
    /// Do a full update.
    FullUpdate,
}

impl PollType {
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

pub enum PollAction {
    DoPoll(PollType),
}

pub type Poll = Vec<(TradeOffer, Option<TradeOfferState>)>;
pub type PollResult = Result<Poll, Error>;

struct Poller {
    steamid: SteamID,
    api: SteamTradeOfferAPI,
    data_directory: PathBuf,
    cancel_duration: Option<Duration>,
    full_poll_update_duration: Duration,
    poll_data: PollData,
}

impl Poller {
    /// Performs a poll for changes to offers. Provides a parameter to determine what type of poll to perform.
    async fn do_poll(
        &mut self,
        poll_type: PollType,
    ) -> PollResult {
        self.poll_data.set_last_poll(time::get_server_time_now());
        
        let mut full_update = poll_type.is_full_update();
        let offers_since = if poll_type == PollType::NewOffers {
            // a very high date
            u32::MAX as i64
        } else if {
            // If we're doing a full update.
            full_update ||
            // Or the date of the last full poll is outdated.
            self.poll_data.last_poll_is_stale(&self.full_poll_update_duration)
        } {
            self.poll_data.set_last_poll_full_update(time::get_server_time_now());
            full_update = true;
            
            self.poll_data.oldest_active_offer
                // It looks like sometimes Steam can be dumb and backdate a modified offer.
                // We need to handle this. Let's add a 30-minute buffer.
                .map(|date| date.timestamp() - (60 * 30))
                .unwrap_or(1)
        } else {
            self.poll_data.offers_since
                // It looks like sometimes Steam can be dumb and backdate a modified offer.
                // We need to handle this. Let's add a 30-minute buffer.
                .map(|date| date.timestamp() - (60 * 30))
                .unwrap_or(1)
        };
        let mut offers_since = time::timestamp_to_server_time(offers_since);
        let (mut offers, descriptions) = self.api.get_raw_trade_offers(
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
            
            match self.poll_data.state_map.get(&offer.tradeofferid) {
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
            self.poll_data.set_oldest_active_offer(polled_oldest_active_offer);
        }
        
        // Clear poll data offers otherwise this could expand infinitely.
        // Using a higher number than is removed so this process needs to run less frequently.
        // This could be better but it works.
        if self.poll_data.state_map.len() > 2500 {
            let mut tradeofferids = self.poll_data.state_map
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            
            // High to low.
            tradeofferids.sort_by(|a, b| b.cmp(a));
            
            let (
                _tradeofferids,
                tradeofferids_to_remove,
            ) = tradeofferids.split_at(2000);
            
            self.poll_data.clear_offers(tradeofferids_to_remove);
        }
        
        self.poll_data.set_offers_since(offers_since);
        
        // Maps raw offers to offers with classinfo descriptions.
        let offers = if let Some(descriptions) = descriptions {
            self.api.map_raw_trade_offers_with_descriptions(poll, descriptions)
        } else {
            self.api.map_raw_trade_offers(poll).await?
        };
        let poll = if offers.is_empty() {
            // map_raw_trade_offers may have excluded some offers - the state of the poll data
            // is not updated until all descriptions are loaded for the offer
            Vec::new()
        } else {
            self.poll_data.changed = true;
            offers
                .into_iter()
                // Combines changed state maps.
                .map(|offer| {
                    let prev_state = prev_states_map.remove(&offer.tradeofferid);
                    
                    // insert new state into map
                    self.poll_data.state_map.insert(offer.tradeofferid, offer.trade_offer_state.clone());
                    
                    (offer, prev_state)
                })
                .collect::<Vec<_>>()
        };
        
        // Only save if changes were detected.
        if self.poll_data.changed {
            self.poll_data.changed = false;
            let _ = file::save_poll_data(
                &self.steamid,
                &serde_json::to_string(&self.poll_data)?,
                &self.data_directory,
            ).await;
        }
        
        Ok(poll)
    }
}

pub fn create_poller(
    api: SteamTradeOfferAPI,
    data_directory: PathBuf,
    options: PollOptions,
) -> (
    mpsc::Sender<PollAction>,
    mpsc::Receiver<PollResult>,
    JoinHandle<()>,
) {
    let steamid = api.steamid;
    let poll_data = file::load_poll_data(
        &steamid,
        &data_directory,
    ).unwrap_or_else(|_| PollData::new());
    // allows sending a message into the poller
    let (
        tx,
        mut rx,
    ) = mpsc::channel::<PollAction>(10);
    // allows broadcasting polls outside of the poller
    let (
        polling_tx,
        polling_rx,
    ) = mpsc::channel::<PollResult>(10);
    let handle = tokio::spawn(async move {
        // Since the mutex is concurrent only one poll can be performed at a time.
        let poller = Arc::new(Mutex::new(Poller {
            api,
            steamid,
            data_directory,
            poll_data,
            cancel_duration: options.cancel_duration,
            full_poll_update_duration: options.full_poll_update_duration,
        }));
        let receiver_poller = Arc::clone(&poller);
        let receiver_polling_tx = polling_tx.clone();
        let is_listening = Arc::new(AtomicBool::new(true));
        let receiver_is_listing = Arc::clone(&is_listening);
        let handle = tokio::spawn(async move {
            let mut poll_events: HashMap<PollType, DateTime<chrono::Utc>> = HashMap::new();
            
            while let Some(message) = rx.recv().await {
                match message {
                    PollAction::DoPoll(poll_type) => {
                        let called_too_recently = if let Some(last_poll_date) = poll_events.get_mut(&poll_type) {
                            let now = chrono::Utc::now();
                            let duration = now - *last_poll_date;
                            
                            *last_poll_date = now;
                            
                            // Last called with the last half a second.
                            duration < Duration::milliseconds(500)
                        } else {
                            poll_events.insert(poll_type, chrono::Utc::now());
                            false
                        };
                        
                        // The last time this type of poll was called too recently.
                        if called_too_recently {
                            // Ignore it.
                            continue;
                        }
                        
                        let poll = receiver_poller.lock().await.do_poll(poll_type).await;
                        
                        if receiver_polling_tx.send(poll).await.is_err() {
                            // They closed the connection.
                            break;
                        }
                    },
                }
            }
            
            // The client is no longer reading new messages
            receiver_is_listing.store(false, Ordering::Relaxed);
        });
        
        let poll_interval = options.poll_interval.to_std()
            .unwrap_or_else(|_| std::time::Duration::from_secs(60 * 5));
        
        loop {
            if !is_listening.load(Ordering::Relaxed) {
                break;
            }
            
            let poll = poller.lock().await.do_poll(PollType::Auto).await;
            
            match polling_tx.send(poll).await {
                Ok(_) => async_std::task::sleep(poll_interval).await,
                // They closed the connection.
                Err(_error) => break,
            }
        }
        
        handle.abort();
    });
    
    (tx, polling_rx, handle)
}