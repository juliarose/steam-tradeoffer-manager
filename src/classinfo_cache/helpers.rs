use crate::{
    response::ClassInfo,
    error::FileError,
    types::{
        ClassInfoClass,
        ClassInfoAppClass,
        AppId,
    },
    helpers::write_file_atomic,
};
use super::types::ClassInfoFile;
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
    collections::{HashMap, HashSet},
};
use futures::future::join_all;
use tokio::task::JoinHandle;
use serde_json;

/// Saves the classinfo.
async fn save_classinfo(
    class: ClassInfoClass,
    classinfo: String,
    data_directory: &Path, 
) -> Result<(), FileError> {
    // first validate the classinfo string
    if let Err(error) = serde_json::from_str::<ClassInfo>(&classinfo) {
        // output a warning...
        log::warn!("{}", error);
        
        return Err(FileError::Parse(error));
    }
    
    let filepath = get_classinfo_file_path(
        &class,
        false,
        data_directory,
    )?;
    
    write_file_atomic(filepath, classinfo.as_bytes()).await?;
    
    Ok(())
}

/// Loads classinfos.
pub async fn load_classinfos(
    classes: &HashSet<&ClassInfoClass>,
    data_directory: &Path, 
) -> Vec<Result<ClassInfoFile, FileError>> {
    let mut tasks: Vec<JoinHandle<Result<ClassInfoFile, FileError>>>= vec![];
    
    for class in classes {
        // must be cloned to move across threads
        let class = **class;
        let class_data_directory = data_directory.to_path_buf();
        
        tasks.push(tokio::spawn(async move {
            load_classinfo(class, &class_data_directory).await
        }));
    }
    
    let mut results: Vec<Result<ClassInfoFile, FileError>> = Vec::new();
    
    for join_result in join_all(tasks).await {
        results.push(match join_result {
            Ok(task_result) => task_result,
            Err(_err) => Err(FileError::JoinError),
        })
    }
    
    results
}

/// Saves classinfos.
pub async fn save_classinfos(
    appid: AppId,
    classinfos: &HashMap<ClassInfoAppClass, String>,
    data_directory: &Path, 
) -> Vec<Result<(), FileError>> {
    let mut tasks: Vec<JoinHandle<Result<(), FileError>>>= vec![];
    
    for ((classid, instanceid), classinfo) in classinfos {
        // must be cloned to move across threads
        let classinfo = classinfo.clone();
        let class = (appid, *classid, *instanceid);
        let class_data_directory = data_directory.to_path_buf();
        
        tasks.push(tokio::spawn(async move {
            save_classinfo(class, classinfo, &class_data_directory).await
        }));
    }
    
    let mut results: Vec<Result<(), FileError>> = Vec::new();
    
    for join_result in join_all(tasks).await {
        results.push(match join_result {
            Ok(task_result) => task_result,
            Err(_err) => Err(FileError::JoinError),
        })
    }

    results
}

async fn load_classinfo(
    class: ClassInfoClass,
    data_directory: &Path, 
) -> Result<ClassInfoFile, FileError> {
    let filepath = get_classinfo_file_path(&class, false, data_directory)?;
    let data = async_fs::read_to_string(&filepath).await?;
    
    match serde_json::from_str::<ClassInfo>(&data) {
        Ok(classinfo) => {
            Ok((class, classinfo))
        },
        Err(error) => {
            // remove the file...
            let _ = async_fs::remove_file(&filepath).await;
            
            Err(FileError::Parse(error))
        },
    }
}

fn get_classinfo_file_path(
    class: &ClassInfoClass,
    is_temp: bool,
    data_directory: &Path, 
) -> Result<PathBuf, FileError> {
    let (appid, classid, instanceid) = class;
    let instanceid = match instanceid {
        Some(instanceid) => *instanceid,
        None => 0,
    };
    let filename: String = match is_temp {
        true => {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(system_time) => {
                    let timestamp = system_time.as_millis();
                    
                    Ok(format!("{}_{}_{}.json.{}.temp", appid, classid, instanceid, timestamp))
                },
                Err(error) => {
                    Err(FileError::SystemTime(error))
                },
            }
        },
        false => {
            Ok(format!("{}_{}_{}.json", appid, classid, instanceid))
        },
    }?;
    
    Ok(data_directory.join(filename))
}