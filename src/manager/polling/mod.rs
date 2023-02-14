mod file;
mod poll_data;
mod poller;
mod poll_type;

pub use poll_type::PollType;
pub use poller::{PollResult, Poll};
pub use poll_data::PollData;

use poller::Poller;

use crate::SteamID;
use crate::api::SteamTradeOfferAPI;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{Duration, DateTime};
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

const DEFAULT_POLL_INTERVAL_SECONDS: i64 = 30;
const DEFAULT_FULL_UPDATE_SECONDS: i64 = 5 * 60;
// Duration in milliseconds for when a poll was called too recently.
const CALLED_TOO_RECENTLY_MILLISECONDS: i64 = 400;

/// Options for polling.
#[derive(Debug, Clone, Copy)]
pub struct PollOptions {
    /// The duration after a sent offer has been active to cancel during a poll. Offers will 
    /// not be cancelled if this is not set.
    pub cancel_duration: Option<Duration>,
    /// The duration after the last poll becomes stale and a new one must be obtained when 
    /// polling using [`crate::polling::PollType::Auto`]. Default is 4 minutes.
    pub poll_full_update_duration: Duration,
    /// Interval to poll at. Default is 30 seconds.
    pub poll_interval: Duration,
}

impl Default for PollOptions {
    fn default() -> Self {
        Self {
            cancel_duration: None,
            poll_full_update_duration: Duration::seconds(DEFAULT_FULL_UPDATE_SECONDS),
            poll_interval: Duration::seconds(DEFAULT_POLL_INTERVAL_SECONDS),
        }
    }
}

impl PollOptions {
    /// Uses the default values but includes a cancel duration.
    pub fn default_with_cancel_duration(duration: Duration) -> Self {
        Self {
            cancel_duration: Some(duration),
            poll_full_update_duration: Duration::seconds(DEFAULT_FULL_UPDATE_SECONDS),
            poll_interval: Duration::seconds(DEFAULT_POLL_INTERVAL_SECONDS),
        }
    }
}

pub enum PollAction {
    DoPoll(PollType),
}

pub fn create_poller(
    steamid: SteamID,
    api: SteamTradeOfferAPI,
    data_directory: PathBuf,
    options: PollOptions,
) -> (
    mpsc::Sender<PollAction>,
    mpsc::Receiver<PollResult>,
    JoinHandle<()>,
) {
    let poll_data = file::load_poll_data(
        steamid,
        &data_directory,
    ).unwrap_or_else(|_| PollData::new());
    // Allows sending a message into the poller.
    let (
        tx,
        mut rx,
    ) = mpsc::channel::<PollAction>(10);
    // Allows broadcasting polls outside of the poller.
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
            poll_full_update_duration: options.poll_full_update_duration,
        }));
        let receiver_poller = Arc::clone(&poller);
        let receiver_polling_tx = polling_tx.clone();
        let poll_interval = options.poll_interval.to_std()
            .unwrap_or_else(|_| std::time::Duration::from_secs(DEFAULT_POLL_INTERVAL_SECONDS as u64));
        let handle = tokio::spawn(async move {
            // To prevent spam.
            let mut poll_events: HashMap<PollType, DateTime<chrono::Utc>> = HashMap::new();
            
            while let Some(message) = rx.recv().await {
                match message {
                    PollAction::DoPoll(poll_type) => {
                        let called_too_recently = if let Some(last_poll_date) = poll_events.get_mut(&poll_type) {
                            let now = chrono::Utc::now();
                            let duration = now - *last_poll_date;
                            
                            *last_poll_date = now;
                            
                            duration < Duration::milliseconds(CALLED_TOO_RECENTLY_MILLISECONDS)
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
        });
        
        loop {
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