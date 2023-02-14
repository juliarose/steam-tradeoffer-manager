//! Types internally used by the crate.

use crate::types::{AppId, ClassId, InstanceId};
use crate::response::ClassInfo;
use std::sync::Arc;
use std::collections::HashMap;
use reqwest_middleware::ClientWithMiddleware;

pub type Client = ClientWithMiddleware;
pub type ClassInfoClass = (AppId, ClassId, InstanceId);
pub type ClassInfoMap = HashMap<ClassInfoClass, Arc<ClassInfo>>;
pub type ClassInfoAppClass = (ClassId, InstanceId);
pub type ClassInfoAppMap = HashMap<ClassInfoAppClass, Arc<ClassInfo>>;