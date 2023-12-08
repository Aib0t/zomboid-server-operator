#![feature(lazy_cell)]
#![feature(fs_try_exists)]

mod config;
mod rcon;
mod steam_api_client;
mod steam_api_client_schemes;
mod zomboid_utils;

use std::path::PathBuf;
use std::process::exit;

use env_logger::Builder;
use env_logger::Target;
use log::info;

use log::LevelFilter;
use log::{debug, error};
use steam_api_client::SteamApiClient;

use clap::Parser;
use std::sync::LazyLock;

use crate::config::ZSOConfig;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: PathBuf,

    // #[arg(short, long)]
    // collections: Vec<u64>,
    #[arg(short, long)]
    ini: Option<PathBuf>,

    #[arg(short, long)]
    maps: bool,
}

static ZSO_CONFIG: LazyLock<ZSOConfig> = LazyLock::new(|| {
    println!("initializing");
    let args = Args::parse();

    match std::fs::try_exists(&args.config) {
        Ok(file_exists) => match file_exists {
            true => {
                let config_data_string =
                    String::from_utf8(std::fs::read(&args.config).unwrap()).unwrap();
                return serde_yaml::from_str(&config_data_string).unwrap();
            }
            false => {
                error!("Config doesn't exists. Creating new config and exiting.");

                let new_config = serde_yaml::to_string(&ZSOConfig::default()).unwrap();
                let _ = std::fs::write(&args.config, new_config.as_bytes());
                exit(0)
            }
        },
        Err(e) => {
            error!("Failed to open config file - {}", e);
            exit(0)
        }
    };
});

#[tokio::main]
async fn main() {
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info);
    builder.filter_module("tracing", LevelFilter::Warn);
    builder.filter_module("hyper", LevelFilter::Info);
    builder.filter_module("rustls", LevelFilter::Info);
    builder.target(Target::Stdout);
    builder.init();

    let args = Args::parse();

    match &args.ini {
        Some(args_ini_path) => zomboid_utils::ini_initial_check(args_ini_path),
        None => {}
    }

    //MAIN

    if ZSO_CONFIG.collections.len() == 0 {
        error!("No collections to parse - aborting! Update the config.");
        exit(0);
    }

    info!("Got collection ids: {:?}", &ZSO_CONFIG.collections);

    let steam_api_client = SteamApiClient::new();
    debug!("Steam client is initialized");

    let full_mod_list = steam_api_client
        .get_list_of_mods_in_collections(ZSO_CONFIG.collections.clone())
        .await;

    info!("Total mods in collections: {}", &full_mod_list.len());

    if full_mod_list.len() == 0 {
        error!("No mods to parse - aborting!");
        exit(0);
    }

    let mods_data = steam_api_client.resolve_mods_data(full_mod_list).await;

    info!("Total parsed mods: {}", &mods_data.len());

    let workshop_items_string = zomboid_utils::generate_workshop_items_string(
        &mods_data,
        &ZSO_CONFIG.workshop_settings.include.workshop_items,
        &ZSO_CONFIG.workshop_settings.exclude.workshop_items,
    );

    let mods_string = zomboid_utils::generate_mods_string(
        &mods_data,
        &ZSO_CONFIG.workshop_settings.include.mods,
        &ZSO_CONFIG.workshop_settings.exclude.mods,
    );

    let mut maps_string = String::new();

    if args.maps {
        maps_string = zomboid_utils::generate_map_string(
            &mods_data,
            &ZSO_CONFIG.workshop_settings.include.maps,
            &ZSO_CONFIG.workshop_settings.exclude.maps,
        );
    }

    match &args.ini {
        Some(ini_path) => {
            info!("Updating server ini");
            if !args.maps {

                match zomboid_utils::update_server_config(
                    ini_path,
                    &workshop_items_string,
                    &mods_string,
                    None,
                ).await {
                    Ok(_) => {
                        info!("Server config was updated. Exiting.");
                    },
                    Err(_) => {
                        error!("Failed to update server config");
                        exit(1)
                    },
                }
            } else {
                match zomboid_utils::update_server_config(
                    ini_path,
                    &workshop_items_string,
                    &mods_string,
                    Some(&maps_string),
                ).await {
                    Ok(_) => {
                        info!("Server config was updated. Exiting.");
                    },
                    Err(_) => {
                        error!("Failed to update server config");
                        exit(1)
                    },
                }
            }
        }
        None => {
            info!("Generated strings for server config:\n");
            println!("WorkshopItems={}", workshop_items_string);
            println!("Mods={}", mods_string);

            if args.maps {
                println!("Map={}", maps_string);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ZSOConfig;

    use super::*;

    #[tokio::test]
    async fn config_test() {
        let config = ZSOConfig::default();
        let config_yaml_string = serde_yaml::to_string(&config).unwrap();
        dbg!(&config_yaml_string);
    }

    // #[tokio::test]
    // async fn config_test() {
    //     let config = ZSOConfig::default();
    //     let config_yaml_string = serde_yaml::to_string(&config).unwrap();
    //     dbg!(&config_yaml_string);

    // }

    #[tokio::test]
    async fn collection_parsing_test() {
        let steam_api = SteamApiClient::new();
        let collections: Vec<u64> = vec![3105210406];
        let result = steam_api.get_list_of_mods_in_collections(collections).await;
        assert_eq!(15, result.len());
    }

    #[tokio::test]
    async fn mods_parsing_test() {
        let steam_api = SteamApiClient::new();
        let collections: Vec<u64> = vec![3105210406];
        let result = steam_api.get_list_of_mods_in_collections(collections).await;

        let result2 = steam_api.resolve_mods_data(result.clone()).await;
        assert_eq!(result.len(), result2.len());
    }
}
