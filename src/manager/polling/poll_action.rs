use super::PollType;

/// An action to send to the polling task.
#[derive(Debug, Clone)]
pub enum PollAction {
    /// Perform a poll.
    DoPoll(PollType),
    /// Stop polling.
    StopPolling,
}