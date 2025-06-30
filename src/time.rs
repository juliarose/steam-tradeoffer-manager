use std::time::SystemTime;
use chrono::{DateTime, Utc, Duration};

/// The datetime format used for requests and responses.
pub type ServerTime = DateTime<Utc>;

/// Converts a unix timestamp to a [`DateTime`].
pub fn timestamp_to_server_time(timestamp: i64) -> ServerTime {
    DateTime::from_timestamp(
        timestamp,
        0,
    ).unwrap_or_default()
}

/// Gets current time.
pub fn get_server_time_now() -> ServerTime {
    ServerTime::from(SystemTime::now())
}

/// Date difference from now.
pub fn date_difference_from_now(date: &ServerTime) -> Option<Duration> {
    // I don't think this should ever fail since `date` will always be a valid date, but in the
    // off chance that it does, the program does not panic.
    Duration::try_seconds(get_server_time_now().timestamp() - date.timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn tests_date_difference_from_now() {
        let datetime = ServerTime::from_timestamp_millis(0).unwrap();
        let difference = date_difference_from_now(&datetime);
        
        assert!(difference.is_some());
    }
}
