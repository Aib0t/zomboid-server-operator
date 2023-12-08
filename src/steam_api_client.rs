use log::{debug, error, warn};
use reqwest::Client;
use crate::steam_api_client_schemes::*;
use std::time::Duration;
use regex::Regex;

#[derive(Default, Clone, Debug)]
pub struct ModData {
    pub mod_id: u64,
    pub mod_name: Vec<String>,
    pub map_name: Vec<String>,
    pub last_updated: u64 //change to DateTime?
}

pub struct SteamApiClient {
    http_client: Client,
    steam_api_url: String,
    //steam_api_key: String,
    //Is it even required?
}

impl SteamApiClient {
    //it's really not, but will do the trick
    pub fn new() -> Self {        

        let http_client = Client::builder()
            .https_only(true)
            .connect_timeout(Duration::from_secs(10))
            .user_agent("zomboid-server-operator v0.0.1")
            .build()
            .unwrap();

        let steam_api_url = "https://api.steampowered.com".to_owned();
        //let steam_api_key = "DUMMY".to_owned();
        Self {
            http_client,
            steam_api_url,
            //steam_api_key,
        }
    }

    pub async fn get_list_of_mods_in_collections(&self, collections_id: Vec<u64>) -> Vec<u64> {
        let mut list_of_mods: Vec<u64> = vec![];
        let mut request_params: Vec<(String, String)> = vec![];

        request_params.push((
            "collectioncount".to_string(),
            collections_id.len().to_string(),
        ));

        for i in 0 as usize..collections_id.len() {
            request_params.push((
                format!("publishedfileids[{}]", i.to_string()),
                collections_id[i].to_string(),
            ));
        }

        let url = format!(
            "{}/ISteamRemoteStorage/GetCollectionDetails/v1/",
            self.steam_api_url
        );
        debug!("{}", url);

        let resp = match self.http_client.post(url).form(&request_params).send().await {
            Ok(collections_data) => collections_data,
            Err(e) => {
                error!("Failed to collections data data: {e}");
                return list_of_mods
            }
        };

        let get_collection_info = match resp.json::<GetCollectionInfo>().await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to parse character data: {e}");
                return list_of_mods

            },
        };

        for collection_data in get_collection_info.response.collectiondetails {
            match collection_data.result {
                1 => {
                    match collection_data.children {
                        Some(mods_data) => {
                            for mod_data in mods_data {
                                list_of_mods.push(mod_data.publishedfileid.parse::<u64>().unwrap())
                            }
                        },
                        None => {
                            warn!("Collection {} is empty.",collection_data.publishedfileid);
                        },
                    }
                },
                9 => {
                    error!("Collection {} result is 9. Collection is probably private or unavalible.",collection_data.publishedfileid);
                    continue;
                },
                _ => {
                    error!("Unknown collection result({}) for collection {}", collection_data.result,collection_data.publishedfileid);
                    continue;
                }
            }
        }

        return list_of_mods;
    }

    pub async fn resolve_mods_data(&self, mut mod_ids: Vec<u64>) -> Vec<ModData> {
        let mut mods_data: Vec<ModData> = vec![];


        //move to lazy once_cell
        let mod_name_re = Regex::new(r"(?m)Mod\sID:\s(?P<mod_name>[ a-zA-Z0-9()\[\]_\-\(\).,]+)(\\r\\n)*").unwrap();
        let map_folder_name_re = Regex::new(r"(?m)Map\sFolder:\s(?P<map_folder>[ a-zA-Z0-9()\[\]_\-\(\).,]+)(\\r\\n)*").unwrap();

        let url = format!(
            "{}/ISteamRemoteStorage/GetPublishedFileDetails/v1/",
            self.steam_api_url
        );

        let batch_size: usize = 10; //todo: move to config

        let mut batches = mod_ids.len() / batch_size;

        if (mod_ids.len() % batch_size) > 0 {
            batches +=1;
        }

        for batch_num in 0..batches {
            debug!("Processing batch {}",batch_num);

            let mod_ids_to_parse: Vec<u64>;

            if mod_ids.len() >= batch_size {
                mod_ids_to_parse = mod_ids.drain(0..batch_size).collect::<Vec<u64>>();
            } else {
                mod_ids_to_parse = mod_ids.drain(0..mod_ids.len()).collect::<Vec<u64>>();
            }

            let mut form_payload: Vec<(String, String)> = vec![("itemcount".to_string(),mod_ids_to_parse.len().to_string())];

            
            
            for mod_pos in 0..mod_ids_to_parse.len() {
                form_payload.push((format!("publishedfileids[{}]",mod_pos),mod_ids_to_parse[mod_pos].to_string()));
            }

            debug!("Getting info for: {:?}",form_payload);

            let resp = match self.http_client.post(&url).form(&form_payload).send().await {
                Ok(collections_data) => collections_data,
                Err(e) => {
                    error!("Failed to get mods data: {e}");
                    return mods_data
                }
            };
    
            let mods_data_response = match resp.json::<GetPublishedFileDetails>().await {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to parse mods data: {e}");
                    return mods_data
    
                },
            };

            for full_mod_data in mods_data_response.response.publishedfiledetails {

                if full_mod_data.result != 1 {
                    error!("Failed to get data for mod {}, skipping mod",full_mod_data.publishedfileid);
                    continue;
                }

                let mod_name_result = mod_name_re.captures_iter(&full_mod_data.description);
                let map_folder_name_result = map_folder_name_re.captures_iter(&full_mod_data.description);

                let mut mod_data = ModData::default();  
                mod_data.mod_id = full_mod_data.publishedfileid.parse::<u64>().unwrap();
                mod_data.last_updated = full_mod_data.time_updated;

                // if &mod_name_result.count() == 0 {
                //     
                // }
                
                let mut mod_names: Vec<String> = vec![];

                for mod_name_capture in mod_name_result {
                    match mod_name_capture.name("mod_name") {
                        Some(mod_name_match) => {
                            if !mod_names.contains(&mod_name_match.as_str().trim_end().to_owned())
                            {
                                mod_names.push(mod_name_match.as_str().trim_end().to_owned())
                            }
                            
                        },
                        None => {
                            
                        },
                    }
                };

                if mod_names.len() == 0 {
                    error!("Failed to parse mod name for mod {}, skipping mod",full_mod_data.publishedfileid);
                    continue;
                }
                mod_data.mod_name = mod_names;

                //MAP NAME

                let mut map_names: Vec<String> = vec![];

                for  map_folder_name_capture in map_folder_name_result {
                    match map_folder_name_capture.name("map_folder") {
                        Some(map_folder_match) => {
                            if !map_names.contains(&map_folder_match.as_str().trim_end().to_owned())
                            {
                                map_names.push(map_folder_match.as_str().trim_end().to_owned())
                            }
                            
                        },
                        None => {
                            
                        },
                    }
                };

                mod_data.map_name = map_names;

                mods_data.push(mod_data);


            }

        }



        mods_data
    }

}
