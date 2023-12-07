mod config;
mod steam_api_client;
mod steam_api_client_schemes;
mod utils;

use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

use env_logger::Builder;
use env_logger::Target;
use log::info;
use log::LevelFilter;
use log::{debug, error, warn};
use steam_api_client::SteamApiClient;

use clap::Parser;
use ini::Ini;

use tokio::fs::try_exists;

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

    #[arg(short, long, default_value_t = false)]
    skip_maps: bool,
}

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
    // if args.collections.len() == 0 {
    //     error!("No collections to parse - aborting! (use --collections)");
    //     exit(0);
    // }

    // info!("Got collection ids: {:?}", &args.collections);

    //INI

    let zso_conf: ZSOConfig = match try_exists(&args.config).await {
        Ok(file_exists) => match file_exists {
            true => {
                let config_data_string =
                    String::from_utf8(tokio::fs::read(&args.config).await.unwrap()).unwrap();
                serde_yaml::from_str(&config_data_string).unwrap()
            }
            false => {
                error!("Config doesn't exists. Creating new config and exiting.");

                let new_config = serde_yaml::to_string(&ZSOConfig::default()).unwrap();
                let _ = tokio::fs::write(&args.config, new_config.as_bytes()).await;
                exit(0)
            }
        },
        Err(e) => {
            error!("Failed to open config file - {}", e);
            exit(0)
        }
    };

    // Ini::load_from_file(args.ini_name) {
    //     Ok(server_conf) => server_conf,
    //     Err(e) => {
    //         error!("Failed to open ini file - {}", e);
    //         exit(0)
    //     }
    // };

    if args.ini.is_some() {
        let server_conf = match Ini::load_from_file(&args.ini.unwrap()) {
            Ok(server_conf) => server_conf,
            Err(e) => {
                error!("Failed to open ini file - {}", e);
                exit(0)
            }
        };

        let main_section = server_conf.section(None::<String>).unwrap();
        let current_workshop_items = match main_section.get("WorkshopItems") {
            Some(workshop_items_string) => {
                debug!("Current workshop items string: {}", workshop_items_string);
                workshop_items_string
                    .split(";")
                    .map(str::to_string)
                    .collect::<Vec<String>>()
            }
            None => {
                error!("WorkshopItems is missing from ini file. Aborting.");
                exit(0)
            }
        };

        let current_mods = match main_section.get("Mods") {
            Some(workshop_items_string) => {
                debug!("Current Mods string: {}", workshop_items_string);
                workshop_items_string
                    .split(";")
                    .map(str::to_string)
                    .collect::<Vec<String>>()
            }
            None => {
                error!("Mods is missing from ini file. Aborting.");
                exit(0)
            }
        };

        let current_maps = match main_section.get("Map") {
            Some(workshop_items_string) => {
                debug!("Current Map string: {}", workshop_items_string);
                workshop_items_string
                    .split(";")
                    .map(str::to_string)
                    .collect::<Vec<String>>()
            }
            None => {
                error!("Map is missing from ini file. Aborting.");
                exit(0)
            }
        };

        info!(
            "Current WorkshopItems: {}, Mods: {}, Maps: {}",
            current_workshop_items.len(),
            current_mods.len(),
            current_maps.len()
        )
    }

    //INI

    //MAIN
    let steam_api_client = SteamApiClient::new();
    debug!("Steam client is initialized");

    let full_mod_list = steam_api_client
        .get_list_of_mods_in_collections(zso_conf.collections)
        .await;

    info!("Total mods in collections: {}", &full_mod_list.len());

    if full_mod_list.len() == 0 {
        error!("No mods to parse - aborting!");
        exit(0);
    }

    let mods_data = steam_api_client.resolve_mods_data(full_mod_list).await;

    info!("Total parsed mods: {}", &mods_data.len());

    let workshop_items_string = generate_workshop_items_string(
        &mods_data,
        &zso_conf.workshop_settings.include.workshop_items,
        &zso_conf.workshop_settings.exclude.workshop_items,
    );

    let mods_string = generate_mods_string(
        &mods_data,
        &zso_conf.workshop_settings.include.mod_ids,
        &zso_conf.workshop_settings.exclude.mod_ids,
    );

    let maps_string = generate_map_string(
        &mods_data,
        &zso_conf.workshop_settings.include.maps,
        &zso_conf.workshop_settings.exclude.maps,
    );

    dbg!(workshop_items_string);
    dbg!(mods_string);
    dbg!(maps_string);


    
    //dbg!(&mods_data);
}

fn generate_workshop_items_string(
    mods_data: &Vec<steam_api_client::ModData>,
    ids_to_include: &Vec<u64>,
    ids_to_exclude: &Vec<u64>,
) -> String {
    let mut workshop_items_string = String::new();
    let mut test_vec: Vec<u64> = vec![]; //mode to impl of ModData

    for mod_data in mods_data {
        if !ids_to_exclude.contains(&mod_data.mod_id) {
            test_vec.push(mod_data.mod_id);
            workshop_items_string += &mod_data.mod_id.to_string();
            workshop_items_string += ";";
        }
    }

    for id_to_include in ids_to_include {
        //we assume user doesn't add excluded mods manually.
        if !test_vec.contains(id_to_include) {
            workshop_items_string += &id_to_include.to_string();
            workshop_items_string += ";";
        }
    }

    workshop_items_string
}

fn generate_mods_string(
    mods_data: &Vec<steam_api_client::ModData>,
    mods_to_include: &Vec<String>,
    mods_to_exclude: &Vec<String>,
) -> String {
    let mut mods_string = String::new();
    let mut test_vec: Vec<String> = vec![]; //mode to impl of ModData

    for mod_data in mods_data {
        for mod_name in mod_data.mod_name.clone() {
            if !mods_to_exclude.contains(&mod_name) {
                test_vec.push(mod_name.clone());
                mods_string += &mod_name;
                mods_string += ";";
            }
        }
    }

    for mod_to_include in mods_to_include {
        //we assume user doesn't add excluded mods manually.
        if !test_vec.contains(mod_to_include) {
            mods_string += mod_to_include;
            mods_string += ";";
        }
    }

    mods_string
}

fn generate_map_string(
    mods_data: &Vec<steam_api_client::ModData>,
    maps_to_include: &Vec<String>,
    maps_to_exclude: &Vec<String>,
) -> String {
    let mut map_string = String::new();
    let mut test_vec: Vec<String> = vec![]; //mode to impl of ModData

    for mod_data in mods_data {
        for map_name in mod_data.map_name.clone() {
            if !maps_to_exclude.contains(&map_name) {
                test_vec.push(map_name.clone());
                map_string += &map_name;
                map_string += ";";
            }
        }
    }

    for maps_to_include in maps_to_include {
        //we assume user doesn't add excluded mods manually.
        if !test_vec.contains(maps_to_include) {
            map_string += maps_to_include;
            map_string += ";";
        }
    }
    map_string += "Muldraugh, KY;";
    map_string
}

#[cfg(test)]
mod tests {
    use crate::config::ZSOConfig;

    use super::*;

    // #[tokio::test]
    // async fn config_test() {
    //     let config = ZSOConfig::default();
    //     let config_yaml_string = serde_yaml::to_string(&config).unwrap();
    //     dbg!(&config_yaml_string);
    //
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
