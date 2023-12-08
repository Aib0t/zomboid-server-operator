
use serde_derive::Deserialize;
use serde_derive::Serialize;


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZSOConfig {
    pub collections: Vec<u64>,
    pub workshop_settings: ConfigWorkshopSettings,
    pub rcon: Option<RconSettings>,
    pub server_settings: Option<ServerSettings>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerSettings {
    pub reboot_command: String,
    pub reboot_delay_sec: u64,
    pub rcon_messages: bool,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RconSettings {
    pub host: String,
    pub port: String,
    pub password: String,
    pub messages: RconMessagesSettings
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RconMessagesSettings {
    pub reboot_15m: String,
    pub reboot_5m: String,
    pub reboot_1m: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigWorkshopSettings {
    pub include: IncludeExcludeStruct,
    pub exclude: IncludeExcludeStruct
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncludeExcludeStruct {
    pub workshop_items: Vec<u64>,
    pub mods: Vec<String>,
    pub maps: Vec<String>,
}

