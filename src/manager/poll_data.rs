use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{
    time::ServerTime,
    types::TradeOfferId,
    TradeOfferState,
    error::FileError,
};
use super::file;
use steamid_ng::SteamID;
use async_fs::File;
use futures_lite::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollData {
    pub offers_since: Option<ServerTime>,
    pub last_poll: Option<ServerTime>,
    pub last_poll_full_update: Option<ServerTime>,
    pub state_map: HashMap<TradeOfferId, TradeOfferState>,
}

impl PollData {
    
    pub async fn save(&self, steamid: &SteamID) -> Result<(), FileError> {
        match file::get_filepath(steamid) {
            Some(filepath) => {
                let mut file = File::create(&filepath).await?;
                let data = serde_json::to_string(&self)?;
                
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
}