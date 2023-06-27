use std::fmt;

/// Operation to act on confirmations.
#[derive(Debug, Clone, Copy)]
pub enum Operation {
    /// Allow confirmation.
    Allow,
    /// Decline or cancel confirmation.
    Cancel,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Cancel => write!(f, "cancel"),
        }
    }
}