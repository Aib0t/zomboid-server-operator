use std::{path::PathBuf, process::exit};

use ini::Ini;
use log::{error, debug, info};
use regex::Regex;

use crate::steam_api_client;




pub(crate) fn generate_workshop_items_string(
    mods_data: &Vec<steam_api_client::ModData>,
    ids_to_include: &Vec<u64>,
    ids_to_exclude: &Vec<u64>,
) -> String {
    let mut workshop_items_string = String::new();
    let mut test_vec: Vec<u64> = vec![]; //move to impl of ModData

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

pub(crate) fn generate_mods_string(
    mods_data: &Vec<steam_api_client::ModData>,
    mods_to_include: &Vec<String>,
    mods_to_exclude: &Vec<String>,
) -> String {
    let mut mods_string = String::new();
    let mut test_vec: Vec<String> = vec![]; //move to impl of ModData

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

pub(crate) fn generate_map_string(
    mods_data: &Vec<steam_api_client::ModData>,
    maps_to_include: &Vec<String>,
    maps_to_exclude: &Vec<String>,
) -> String {
    let mut map_string = String::new();
    let mut test_vec: Vec<String> = vec![]; //move to impl of ModData

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
        //we assume user doesn't add excluded maps manually.
        if !test_vec.contains(maps_to_include) {
            map_string += maps_to_include;
            map_string += ";";
        }
    }
    map_string += "Muldraugh, KY;";
    map_string
}

pub fn ini_initial_check(ini_path: &PathBuf) {
    let server_conf = match Ini::load_from_file(ini_path) {
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


pub(crate) fn _update_server_config_ini(ini_path: &PathBuf,workshop_items_string: &str,mods_string: &str,maps_string: Option<&str>) -> Result<(),()> {

    //doing so remove formatting and comments, which might not be the best solution.

    let server_conf = match Ini::load_from_file(ini_path) {
        Ok(server_conf) => server_conf,
        Err(e) => {
            error!("Failed to open ini file - {}", e);
            return Err(())
        }
    };

    let main_section = server_conf.section(None::<String>).unwrap();

    main_section.clone().insert("WorkshopItems", workshop_items_string);
    main_section.clone().insert("Mods", mods_string);

    match maps_string {
        Some(maps_string) => {
            main_section.clone().insert("Maps", maps_string);
        },
        None => {

        },
    };

    match server_conf.write_to_file(ini_path) {
        Ok(_) => {},
        Err(e) => {
            error!("Failed to write ini file - {}", e);
            return Err(())
        }
    }

    Ok(())

}

pub async fn update_server_config(ini_path: &PathBuf,workshop_items_string: &str,mods_string: &str,maps_string: Option<&str>) -> Result<(),()> {

    let mut server_conf = match tokio::fs::read_to_string(ini_path).await {
        Ok(server_conf) => server_conf,
        Err(e) => {
            error!("Failed to open ini file - {}", e);
            return Err(())
        }
    };
    let workshop_items_regex = Regex::new(r"(?m)^WorkshopItems=(?P<workshop_items>.*);{0,1}$").unwrap();
    let mods_regex = Regex::new(r"(?m)^Mods=(?P<mods>.*);{0,1}$").unwrap();
    let maps_regex = Regex::new(r"(?m)^Map=(?P<maps>.*);{0,1}$").unwrap();

    server_conf = workshop_items_regex.replace(&server_conf,format!("WorkshopItems={workshop_items_string}")).to_string();
    server_conf = mods_regex.replace(&server_conf,format!("Mods={mods_string}")).to_string();
    match maps_string {
        Some(maps_string) => {
            server_conf = maps_regex.replace(&server_conf,format!("Map={maps_string}")).to_string();
        },
        None => {

        },
    }

    match tokio::fs::write(ini_path,&server_conf.as_bytes()).await {
        Ok(server_conf) => server_conf,
        Err(e) => {
            error!("Failed to write ini file - {}", e);
            return Err(())
        }
    };

    Ok(())

}