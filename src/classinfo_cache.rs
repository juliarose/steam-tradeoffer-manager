use crate::response::{
    ClassInfo,
    ClassInfoMap,
    ClassInfoAppClass,
    ClassInfoClass
};
use std::{
    env,
    sync::Arc,
    collections::HashMap
};
use async_fs::File;
use futures_lite::io::AsyncWriteExt;
use std::path::Path;
use futures::future::join_all;
use std::fmt;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct ClassInfoCache {
    map: HashMap<ClassInfoClass, Arc<ClassInfo>>,
}

impl ClassInfoCache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }
    
    pub fn get_classinfo(&self, class: &ClassInfoClass) -> Option<Arc<ClassInfo>> {
        match self.map.get(&class) {
            Some(classinfo) => Some(Arc::clone(classinfo)),
            None => None,
        }
    }
    
    pub async fn load_classes(&mut self, classes: &Vec<ClassInfoClass>) {
        for result in load_classinfos(classes).await {
            // we don't care if any errors occurred...
            if let Ok(file) = result {
                self.map.insert(file.class, Arc::new(file.classinfo));
            }
        }
    }
    
    pub fn insert_classinfos(&mut self, appid: u32, classinfos: &HashMap<ClassInfoAppClass, String>) -> Result<ClassInfoMap, serde_json::Error> {
        let mut map = HashMap::new();
        
        for ((classid, instanceid), classinfo_string) in classinfos {
            let classinfo = serde_json::from_str(classinfo_string)?;
            let classinfo = Arc::new(classinfo);
            let class = (appid.clone(), classid.clone(), instanceid.clone());
            
            self.map.insert(class, Arc::clone(&classinfo));
            map.insert(class, Arc::clone(&classinfo));
        }

        Ok(map)
    }
}

struct ClassInfoFile {
    class: ClassInfoClass,
    classinfo: ClassInfo,
}

pub enum FileError {
    FileSystem(std::io::Error),
    ParseError(serde_json::Error),
    JoinError,
    PathError,
}

impl From<serde_json::Error> for FileError {
    fn from(error: serde_json::Error) -> FileError {
        FileError::ParseError(error)
    }
}

impl From<std::io::Error> for FileError {
    fn from(error: std::io::Error) -> FileError {
        FileError::FileSystem(error)
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileError::FileSystem(s) => write!(f, "{}", s),
            FileError::ParseError(s) => write!(f, "{}", s),
            FileError::PathError => write!(f, "Path conversion to string failed"),
            FileError::JoinError => write!(f, "Join error"),
        }
    }
}

async fn load_classinfos(classes: &Vec<ClassInfoClass>) -> Vec<Result<ClassInfoFile, FileError>> {
    let mut tasks: Vec<JoinHandle<Result<ClassInfoFile, FileError>>>= vec![];
    
    for class in classes {
        // must be cloned to move across threads
        let class = class.clone();
        
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

async fn load_classinfo(class: ClassInfoClass) -> Result<ClassInfoFile, FileError> {
    match get_classinfo_file_path(&class) {
        Some(filepath) => {
            let data = async_fs::read_to_string(filepath).await?;
            let classinfo = serde_json::from_str::<ClassInfo>(&data)?;
                    
            Ok(ClassInfoFile {
                class,
                classinfo,
            })
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

pub async fn save_classinfos(appid: u32, classinfos: &HashMap<ClassInfoAppClass, String>) -> Vec<Result<(), FileError>> {
    let mut tasks: Vec<JoinHandle<Result<(), FileError>>>= vec![];
    
    for ((classid, instanceid), classinfo) in classinfos {
        // must be cloned to move across threads
        let classinfo = classinfo.clone();
        let class = (appid.clone(), *classid, *instanceid);
        
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