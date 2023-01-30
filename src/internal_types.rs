use std::{sync::Arc, collections::HashMap};
use reqwest_middleware::ClientWithMiddleware;
use crate::{types::{AppId, ClassId, InstanceId}, response::ClassInfo};

pub type Client = ClientWithMiddleware;
pub type ClassInfoClass = (AppId, ClassId, InstanceId);
pub type ClassInfoMap = HashMap<ClassInfoClass, Arc<ClassInfo>>;
pub type ClassInfoAppClass = (ClassId, InstanceId);
pub type ClassInfoAppMap = HashMap<ClassInfoAppClass, Arc<ClassInfo>>;