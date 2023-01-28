use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{NaiveDateTime, DateTime, Utc, Duration};

/// The datetime formated used for requests and responses.
pub type ServerTime = DateTime<Utc>;

/// Converts a unix timestamp to a [DateTime].
pub fn timestamp_to_server_time(timestamp: i64) -> ServerTime {
    // I'm not sure when this would ever fail, so hopefully it never fails
    let naive_data_time = NaiveDateTime::from_timestamp_opt(
        timestamp,
        0,
    ).unwrap_or_default();
    let time: ServerTime = DateTime::from_utc(naive_data_time, Utc);

    time
}

/// Gets current time.
pub fn get_server_time_now() -> ServerTime {
    ServerTime::from(SystemTime::now())
}

/// Gets current timestamp.
pub fn get_system_time() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        // should never occur
        Err(_) => 0,
    }
}

/// Date difference from now.
pub fn date_difference_from_now(date: &ServerTime) -> Duration {
    Duration::seconds(get_server_time_now().timestamp() - date.timestamp())
}
