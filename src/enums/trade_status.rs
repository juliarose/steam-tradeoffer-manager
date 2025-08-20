use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, EnumString};

/// Status of a trade.
#[derive(Debug, Serialize_repr, Deserialize_repr, Display, EnumString, PartialEq, TryFromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum TradeStatus {
    /// Trade has just been accepted/confirmed, but no work has been done yet.
    Init = 0,
    /// Steam is about to start committing the trade.
    PreCommitted = 1,
    /// The items have been exchanged.
    Committed = 2,
    /// All work is finished.
    Complete = 3,
    /// Something went wrong after Init, but before Committed, and the trade has been rolled back.
    Failed = 4,
    /// A support person rolled back the trade for one side.
    PartialSupportRollback = 5,
    /// A support person rolled back the trade for both sides.
    FullSupportRollback = 6,
    /// A support person rolled back the trade for some set of items.
    SupportRollbackSelective = 7,
    /// We tried to roll back the trade when it failed, but haven't managed to do that for all
    /// items yet.
    RollbackFailed = 8,
    /// We tried to roll back the trade, but some failure didn't go away and we gave up.
    RollbackAbandoned = 9,
    /// Trade is in escrow.
    InEscrow = 10,
    /// A trade in escrow was rolled back.
    EscrowRollback = 11,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    
    #[derive(Serialize, Deserialize)]
    struct Body {
        status: TradeStatus,
    }
    
    #[test]
    fn serializes() {
        
        assert_eq!(serde_json::to_string(&Body { status: TradeStatus::Init }).unwrap(), r#"{"status":0}"#);
    }
    
    #[test]
    fn deserializes() {
        let json = r#"{"status":0}"#;
        let body: Body = serde_json::from_str(json).unwrap();
        
        assert_eq!(body.status, TradeStatus::Init);
    }
}
