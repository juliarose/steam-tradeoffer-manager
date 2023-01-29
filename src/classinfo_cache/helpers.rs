use crate::{
    response::ClassInfo,
    error::FileError,
    types::{ClassInfoClass, ClassInfoAppClass, AppId},
    helpers::write_file_atomic,
};
use super::types::ClassInfoFile;
use std::{path::{Path, PathBuf}, collections::{HashMap, HashSet}};
use futures::future::join_all;
use tokio::task::JoinHandle;
use serde_json;

/// Saves the classinfo.
async fn save_classinfo(
    class: ClassInfoClass,
    classinfo: String,
    data_directory: &Path, 
) -> Result<(), FileError> {
    // Before saving we want to validate if the JSON is valid
    serde_json::from_str::<ClassInfo>(&classinfo)?;
    
    let filepath = get_classinfo_file_path(
        &class,
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
) {
    let tasks = classinfos
        .iter()
        .map(|((classid, instanceid), classinfo)|  {
            // must be cloned to move across threads
            let classinfo = classinfo.to_owned();
            let class = (appid, *classid, *instanceid);
            let class_data_directory = data_directory.to_path_buf();
            
            tokio::spawn(async move {
                save_classinfo(class, classinfo, &class_data_directory).await
            })
        })
        .collect::<Vec<_>>();
    
    for join_result in join_all(tasks).await {
        match join_result {
            Ok(result) => if let Err(error) = result {
                // These are allowed to fail but we want a message of the error.
                log::debug!("Error saving classinfo: {}", error);
            },
            Err(_error) => {},
        }
    }
}

async fn load_classinfo(
    class: ClassInfoClass,
    data_directory: &Path, 
) -> Result<ClassInfoFile, FileError> {
    let filepath = get_classinfo_file_path(&class, data_directory)?;
    let data = async_fs::read_to_string(&filepath).await?;
    
    match serde_json::from_str::<ClassInfo>(&data) {
        Ok(classinfo) => Ok((class, classinfo)),
        Err(error) => {
            // Remove the file...
            let _ = async_fs::remove_file(&filepath).await;
            
            Err(FileError::Parse(error))
        },
    }
}

fn get_classinfo_file_path(
    class: &ClassInfoClass,
    data_directory: &Path, 
) -> Result<PathBuf, FileError> {
    let (appid, classid, instanceid) = class;
    let filename = format!("{}_{}_{}.json", appid, classid, instanceid.unwrap_or(0));
    
    Ok(data_directory.join(filename))
}