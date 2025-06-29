use super::poll_data::PollData;
use crate::error::FileError;
use crate::SteamID;
use crate::helpers::write_file_atomic;
use std::path::Path;
use std::fs;

// This method is intentionally synchronous to allow TradeOfferManager to be instantiated outside of an async context.
// With most use-cases this will rarely be a problem and simplifies instantiation.
pub fn load_poll_data(
    steamid: SteamID,
    path: &Path,
) -> Result<PollData, FileError> {
    let filepath = path.join(format!("poll_data_{}.json", u64::from(steamid)));
    let data = fs::read_to_string(filepath)?;
    let poll_data: PollData = serde_json::from_str(&data)?;
    
    Ok(poll_data)
}

pub async fn save_poll_data(
    steamid: SteamID,
    data: &str,
    path: &Path,
) -> Result<(), FileError> {
    let filepath = path.join(format!("poll_data_{}.json", u64::from(steamid)));
    
    write_file_atomic(filepath, data.as_bytes()).await?;
    
    Ok(())
}