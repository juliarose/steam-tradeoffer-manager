//! Models related to polling offers.

mod file;
mod poll_type;
mod poll_action;
mod poller;
mod poll_data;

pub use poll_type::PollType;
pub use poll_action::PollAction;
pub use poller::{Result, Poll};
pub use poll_data::PollData;
/// The receiver for polling events.
pub type PollReceiver = mpsc::Receiver<Result>;
/// The sender for polling events.
pub type PollSender = mpsc::Sender<PollAction>;

use poller::Poller;

use crate::SteamID;
use crate::api::SteamTradeOfferAPI;
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
    /// polling using [`steam_tradeoffer_manager::polling::PollType::Auto`]. Default is 5 minutes.
    pub poll_full_update_duration: Duration,
    /// Interval to poll at. Default is 30 seconds.
    pub poll_interval: Duration,
}

impl Default for PollOptions {
    fn default() -> Self {
        Self {
            cancel_duration: None,
            // unwrap is safe because the value is in range
            poll_full_update_duration: Duration::try_seconds(DEFAULT_FULL_UPDATE_SECONDS).unwrap(),
            // unwrap is safe because the value is in range
            poll_interval: Duration::try_seconds(DEFAULT_POLL_INTERVAL_SECONDS).unwrap(),
        }
    }
}

impl PollOptions {
    /// Uses the default values but includes a cancel duration.
    pub fn default_with_cancel_duration(duration: Duration) -> Self {
        Self {
            cancel_duration: Some(duration),
            ..Default::default()
        }
    }
}

/// Packs the sender, receiver, and [`JoinHandle`] for the poller.
pub struct Polling {
    pub sender: mpsc::Sender<PollAction>,
    pub receiver: mpsc::Receiver<Result>,
    pub handle: JoinHandle<()>,
}

impl Polling {
    pub fn new(
        steamid: SteamID,
        api: SteamTradeOfferAPI,
        options: PollOptions,
    ) -> Self {
        let poll_data = file::load_poll_data(
            steamid,
            &api.data_directory,
        ).unwrap_or_default();
        // Allows sending a message into the poller.
        let (
            sender,
            receiver,
        ) = mpsc::channel::<PollAction>(10);
        // Allows broadcasting polls outside of the poller.
        let (
            polling_sender,
            polling_receiver,
        ) = mpsc::channel::<Result>(10);
        let handle = tokio::spawn(async move {
            // The asynchronous mutex allows only one poll to be performed at a time. This not only 
            // ensures that the poller is not spammed with requests but also that the state is not 
            // modified by multiple tasks at the same time.
            let poller = Arc::new(Mutex::new(Poller {
                api,
                steamid,
                poll_data,
                cancel_duration: options.cancel_duration,
                poll_full_update_duration: options.poll_full_update_duration,
            }));
            let handle = tokio::spawn(receive_poll_action_events(
                receiver,
                polling_sender.clone(),
                poller.clone(),
            ));
            let poll_interval = options.poll_interval.to_std()
                .unwrap_or(std::time::Duration::from_secs(DEFAULT_POLL_INTERVAL_SECONDS as u64));
            
            // Performs polls.
            loop {
                let poll = poller
                    .lock().await
                    .do_poll(PollType::Auto).await;
                
                match polling_sender.send(poll).await {
                    Ok(_) => async_std::task::sleep(poll_interval).await,
                    // The connection was closed or receiver stopped listening for events.
                    Err(_error) => break,
                }
            }
            
            handle.abort();
        });
        
        Self {
            sender,
            receiver: polling_receiver,
            handle,
        }
    }
}

/// Receives poll action events.
async fn receive_poll_action_events(
    mut receiver: mpsc::Receiver<PollAction>,
    sender: mpsc::Sender<Result>,
    poller: Arc<Mutex<Poller>>,
) {
    // To prevent spam.
    let mut poll_events: HashMap<PollType, DateTime<chrono::Utc>> = HashMap::new();
    
    while let Some(message) = receiver.recv().await {
        match message {
            PollAction::DoPoll(poll_type) => {
                // This type of poll was called too recently.
                if is_called_too_recently(&mut poll_events, poll_type) {
                    // Ignore it.
                    continue;
                }
                
                let poll = poller.lock().await.do_poll(poll_type).await;
                
                if sender.send(poll).await.is_err() {
                    // They closed the connection.
                    break;
                }
            },
            // Breaks out of the loop and ends the task.
            PollAction::StopPolling => break,
        }
    }
}

/// Checks if a poll was called too recently. Mutates the `poll_events` map to update the last 
/// poll date to now.
fn is_called_too_recently(
    poll_events: &mut HashMap<PollType, DateTime<chrono::Utc>>,
    poll_type: PollType,
) -> bool {
    if let Some(last_poll_date) = poll_events.get_mut(&poll_type) {
        let now = chrono::Utc::now();
        let duration = now - *last_poll_date;
        
        *last_poll_date = now;
        
        // unwrap is safe because the value is in range
        duration < Duration::try_milliseconds(CALLED_TOO_RECENTLY_MILLISECONDS).unwrap()
    } else {
        poll_events.insert(poll_type, chrono::Utc::now());
        false
    }
}