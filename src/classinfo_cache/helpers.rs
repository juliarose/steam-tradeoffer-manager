use crate::{
    response::ClassInfo,
    error::FileError,
    types::AppId,
    internal_types::{ClassInfoClass, ClassInfoAppClass},
    helpers::write_file_atomic,
};
use super::types::ClassInfoFile;
use std::{path::{Path, PathBuf}, collections::{HashMap, HashSet}};
use futures::future::join_all;

/// Saves the classinfo.
async fn save_classinfo(
    class: ClassInfoClass,
    classinfo: String,
    data_directory: PathBuf, 
) -> Result<(), FileError> {
    // Before saving we want to validate if the JSON is valid
    serde_json::from_str::<ClassInfo>(&classinfo)?;
    
    let filepath = get_classinfo_file_path(
        &class,
        &data_directory,
    );
    
    write_file_atomic(filepath, classinfo.as_bytes()).await?;
    
    Ok(())
}

/// Loads classinfos.
pub async fn load_classinfos(
    classes: &HashSet<&ClassInfoClass>,
    data_directory: &Path, 
) -> Vec<Result<ClassInfoFile, FileError>> {
    let tasks = classes
        .iter()
        .map(|class|  {
            // must be cloned to move across threads
            let class = **class;
            let class_data_directory = data_directory.to_path_buf();
            
            load_classinfo(class, class_data_directory)
        })
        .collect::<Vec<_>>();
    
    join_all(tasks).await
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
            
            save_classinfo(class, classinfo, class_data_directory)
        })
        .collect::<Vec<_>>();
    
    for result in join_all(tasks).await {
        if let Err(error) = result {
            // These are allowed to fail but we want a message of the error.
            log::debug!("Error saving classinfo: {}", error);
        }
    }
}

async fn load_classinfo(
    class: ClassInfoClass,
    data_directory: PathBuf, 
) -> Result<ClassInfoFile, FileError> {
    let filepath = get_classinfo_file_path(&class, &data_directory);
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
) -> PathBuf {
    let (appid, classid, instanceid) = class;
    let filename = format!("{}_{}_{}.json", appid, classid, instanceid.unwrap_or(0));
    
    data_directory.join(filename)
}