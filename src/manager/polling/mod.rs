//! Models related to polling offers.

mod file;
mod poll_type;
mod poll_action;
mod poller;
mod poll_data;

pub use poll_action::PollAction;
pub use poll_data::PollData;
pub use poll_type::PollType;
pub use poller::{Poll, Result};
/// The receiver for polling events.
pub type PollReceiver = mpsc::Receiver<Result>;
/// The sender for polling events.
pub type PollSender = mpsc::Sender<PollAction>;

use poller::Poller;

use crate::api::SteamTradeOfferAPI;
use crate::SteamID;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration};
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

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
    
    /// Checks that the durations aren't too low so API calls are not spammed.
    fn sanity_check(&mut self) {
        let one_second = Duration::try_seconds(1).unwrap();
        
        if self.poll_full_update_duration < one_second {
            log::warn!("poll_full_update_duration is less than 1 second, setting to 1 second");
            self.poll_full_update_duration = one_second;
        }
        
        if self.poll_interval < one_second {
            log::warn!("poll_interval is less than 1 second, setting to 1 second");
            self.poll_interval = one_second;
        }
    }
}

/// Packs the sender, receiver, and [`CancellationToken`] for the poller.
pub struct Polling {
    pub sender: mpsc::Sender<PollAction>,
    pub receiver: mpsc::Receiver<Result>,
    pub cancellation_token: CancellationToken,
}

impl Polling {
    /// Creates a new polling handle.
    pub fn new(
        steamid: SteamID,
        api: SteamTradeOfferAPI,
        mut options: PollOptions,
    ) -> Self {
        // Sanity check the options.
        options.sanity_check();
        
        let cancellation_token = CancellationToken::new();
        let token = cancellation_token.clone();
        let poll_data = file::load_poll_data(
            steamid,
            &api.data_directory,
        ).unwrap_or_default();
        // Allows sending a message into the polling handle.
        let (
            sender,
            receiver,
        ) = mpsc::channel::<PollAction>(10);
        // Allows sending polls outside of the polling handle.
        let (
            polling_sender,
            polling_receiver,
        ) = mpsc::channel::<Result>(10);
        
        // This is the task that performs the polling.
        tokio::spawn(async move {
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
            let poll_interval = options.poll_interval.to_std()
                .unwrap_or(std::time::Duration::from_secs(DEFAULT_POLL_INTERVAL_SECONDS as u64));
            
            // Task that listens for poll action events.
            tokio::spawn(receive_poll_action_events(
                receiver,
                polling_sender.clone(),
                poller.clone(),
                token.clone(),
            ));
            
            // Performs polls.
            loop {
                let poll = poller
                    .lock().await
                    .do_poll(PollType::Auto)
                    .await;
                
                if let Err(_error) = polling_sender.send(poll).await {
                    // The connection was closed or receiver stopped listening for events.
                    break;
                }
                
                tokio::select! {
                    // Breaks out of the loop and ends the task.
                    _ = token.cancelled() => break,
                    // Waits until the next poll interval before continuing.
                    _ = async_std::task::sleep(poll_interval) => continue,
                }
            }
        });
        
        Self {
            sender,
            receiver: polling_receiver,
            cancellation_token,
        }
    }
}

/// Receives poll action events.
async fn receive_poll_action_events(
    mut receiver: mpsc::Receiver<PollAction>,
    sender: mpsc::Sender<Result>,
    poller: Arc<Mutex<Poller>>,
    cancellation_token: CancellationToken,
) {
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
            
            // unwrap is safe because the value for CALLED_TOO_RECENTLY_MILLISECONDS is in range
            duration < Duration::try_milliseconds(CALLED_TOO_RECENTLY_MILLISECONDS).unwrap()
        } else {
            poll_events.insert(poll_type, chrono::Utc::now());
            false
        }
    }
    
    // To prevent spam.
    let mut poll_events: HashMap<PollType, DateTime<chrono::Utc>> = HashMap::new();
    
    loop {
        tokio::select! {
            // Breaks out of the loop and ends the task.
            _ = cancellation_token.cancelled() => break,
            message = receiver.recv() => {
                if let Some(message) = message {
                    match message {
                        PollAction::DoPoll(poll_type) => {
                            // To prevent spam.
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
                } else {
                    // The sender was dropped
                    break;
                }
            }
        }
    }
}
