use crate::{error::FileError, SteamID};
use async_fs::File;
use futures_lite::io::AsyncWriteExt;
use std::{path::PathBuf, fs};
use super::poll_data::PollData;

pub fn load_poll_data(
    steamid: &SteamID,
    path: &PathBuf,
) -> Result<PollData, FileError> {
    let filepath = path.join(format!("poll_data_{}.json", u64::from(*steamid)));
    let data = fs::read_to_string(&filepath)?;
    let poll_data: PollData = serde_json::from_str(&data)?;
    
    Ok(poll_data)
}

pub async fn save_poll_data(
    steamid: &SteamID,
    data: &str,
    path: &PathBuf,
) -> Result<(), FileError> {
    let filepath = path.join(format!("poll_data_{}.json", u64::from(*steamid)));
    let mut file = File::create(&filepath).await?;
    
    match file.write_all(data.as_bytes()).await {
        Ok(_) => {
            file.flush().await?;
    
            Ok(())
        },
        Err(error) => {
            // something went wrong writing to this file...
            async_fs::remove_file(&filepath).await?;
            
            Err(error.into())
        }
    }
}