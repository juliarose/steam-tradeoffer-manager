use crate::{
    error::FileError,
    SteamID,
};
use async_fs::File;
use futures_lite::io::AsyncWriteExt;
use std::path::Path;
use std::fs;
use super::poll_data::PollData;

pub fn load_poll_data(steamid: &SteamID) -> Result<PollData, FileError> {
    match get_filepath(steamid) {
        Some(filepath) => {
            let data = fs::read_to_string(filepath)?;
            let poll_data: PollData = serde_json::from_str(&data)?;
            
            Ok(poll_data)
        },
        None => Err(FileError::PathError),
    }
}

pub async fn save_poll_data(steamid: &SteamID, data: &str) -> Result<(), FileError> {
    match get_filepath(steamid) {
        Some(filepath) => {
            let mut file = File::create(&filepath).await?;
            // let data = serde_json::to_string(&poll_data)?;
            
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
        },
        None => Err(FileError::PathError),
    }
}

pub fn get_filepath(steamid: &SteamID) -> Option<String> {
    let rootdir = env!("CARGO_MANIFEST_DIR");
    let filename = format!("assets/poll_data_{}.json", u64::from(*steamid));
    
    match Path::new(rootdir).join(filename).to_str() {
        Some(filepath) => Some(String::from(filepath)),
        None => None,
    }
}