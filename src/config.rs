use std::collections::HashMap;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::BTreeMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZSOConfig {
    pub collections: Vec<u64>,
    pub workshop_settings: ConfigWorkshopSettings
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigWorkshopSettings {
    pub include: IncludeExcludeStruct,
    pub exclude: IncludeExcludeStruct
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncludeExcludeStruct {
    pub workshop_items: Vec<u64>,
    pub mod_ids: Vec<String>,
    pub maps: Vec<String>,
}

