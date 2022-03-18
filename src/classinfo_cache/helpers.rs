use crate::{
    response::classinfo::ClassInfo,
    error::FileError,
    types::{
        ClassInfoClass,
        ClassInfoAppClass
    }
};
use super::{
    types::ClassInfoFile,
};
use futures::future::join_all;
use tokio::task::JoinHandle;
use std::collections::HashMap;
use async_fs::File;
use futures_lite::io::AsyncWriteExt;
use std::path::Path;

async fn load_classinfo(class: ClassInfoClass) -> Result<ClassInfoFile, FileError> {
    match get_classinfo_file_path(&class) {
        Some(filepath) => {
            let data = async_fs::read_to_string(filepath).await?;
            let classinfo = serde_json::from_str::<ClassInfo>(&data)?;
                    
            Ok((
                class,
                classinfo,
            ))
        },
        None => Err(FileError::PathError),
    }
}

fn get_classinfo_file_path(class: &ClassInfoClass) -> Option<String> {
    fn get_classinfo_file_name(class: &ClassInfoClass) -> String {
        let (appid, classid, instanceid) = class;
        let instanceid = match instanceid {
            Some(instanceid) => *instanceid,
            None => 0,
        };
        
        format!("assets/{}_{}_{}.json", appid, classid, instanceid)
    }
    
    let rootdir = env!("CARGO_MANIFEST_DIR");
    let filename = get_classinfo_file_name(&class);
    
    match Path::new(rootdir).join(filename).to_str() {
        Some(filepath) => Some(String::from(filepath)),
        None => None,
    }
}

async fn save_classinfo(class: ClassInfoClass, classinfo: String) -> Result<(), FileError> {
    match get_classinfo_file_path(&class) {
        Some(filepath) => {
            let mut file = File::create(&filepath).await?;
            // let data = serde_json::to_string(&classinfo)?;
            
            match file.write_all(classinfo.as_bytes()).await {
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

pub async fn load_classinfos(classes: &Vec<ClassInfoClass>) -> Vec<Result<ClassInfoFile, FileError>> {
    let mut tasks: Vec<JoinHandle<Result<ClassInfoFile, FileError>>>= vec![];
    
    for class in classes {
        // must be cloned to move across threads
        let class = *class;
        
        tasks.push(tokio::spawn(async move {
            load_classinfo(class).await
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

pub async fn save_classinfos(appid: u32, classinfos: &HashMap<ClassInfoAppClass, String>) -> Vec<Result<(), FileError>> {
    let mut tasks: Vec<JoinHandle<Result<(), FileError>>>= vec![];
    
    for ((classid, instanceid), classinfo) in classinfos {
        // must be cloned to move across threads
        let classinfo = classinfo.clone();
        let class = (appid, *classid, *instanceid);
        
        tasks.push(tokio::spawn(async move {
            save_classinfo(class, classinfo).await
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