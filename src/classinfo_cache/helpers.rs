use crate::response::ClassInfo;
use crate::error::FileError;
use crate::types::{AppId, ClassInfoClass, ClassInfoAppClass};
use crate::helpers::write_file_atomic;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use serde_json::value::RawValue;
use futures::future::join_all;

type ClassInfoFile = (ClassInfoClass, ClassInfo);

/// Saves classinfos.
pub fn save_classinfos<P>(
    appid: AppId,
    classinfos: Vec<(ClassInfoAppClass, Box<RawValue>)>,
    data_directory: P, 
) -> tokio::task::JoinHandle<()>
where
    P: Into<PathBuf>,
{
    // We can accept anything that can be converted into a PathBuf but we need an owned value for
    // our task.
    let data_directory: PathBuf = data_directory.into();
    
    tokio::spawn(async move {
        let tasks = classinfos
            .into_iter()
            .map(|((classid, instanceid), classinfo)|  {
                let filepath = get_classinfo_file_path(
                    (appid, classid, instanceid),
                    &data_directory,
                );
                
                save_classinfo(classinfo, filepath)
            })
            .collect::<Vec<_>>();
        
        for result in join_all(tasks).await {
            if let Err(error) = result {
                // These are allowed to fail but we want a message of the error.
                log::debug!("Error saving classinfo: {error}");
            }
        }
    })
}

/// Loads classinfos.
pub async fn load_classinfos(
    classes: &HashSet<&ClassInfoClass>,
    data_directory: &Path, 
) -> Vec<Result<ClassInfoFile, FileError>> {
    let tasks = classes
        .iter()
        .map(|class| load_classinfo(**class, data_directory))
        .collect::<Vec<_>>();
    
    join_all(tasks).await
}

/// Saves the classinfo. The classinfo value should be checked for validatity before calling this
/// function.
async fn save_classinfo(
    classinfo: Box<RawValue>,
    filepath: PathBuf,
) -> std::io::Result<()> {
    write_file_atomic(filepath, classinfo.get().as_bytes()).await
}

async fn load_classinfo(
    class: ClassInfoClass,
    data_directory: &Path,
) -> Result<ClassInfoFile, FileError> {
    let filepath = get_classinfo_file_path(class, data_directory);
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
    class: ClassInfoClass,
    data_directory: &Path, 
) -> PathBuf {
    let (appid, classid, instanceid) = class;
    let filename = format!("{}_{}_{}.json", appid, classid, instanceid.unwrap_or(0));
    
    data_directory.join(filename)
}
