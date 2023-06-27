use super::PollType;

#[derive(Debug, Clone)]
pub enum PollAction {
    DoPoll(PollType),
}