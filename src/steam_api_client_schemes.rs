use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;
use time::OffsetDateTime;


#[derive(Serialize, Deserialize)]
pub struct GetCollectionInfo {
    #[serde(rename = "response")]
    pub response: GetCollectionInfoResponse,
}

#[derive(Serialize, Deserialize)]
pub struct GetCollectionInfoResponse {
    #[serde(rename = "result")]
    pub result: i64,

    #[serde(rename = "resultcount")]
    pub resultcount: i64,

    #[serde(rename = "collectiondetails")]
    pub collectiondetails: Vec<Collectiondetail>,
}

#[derive(Serialize, Deserialize)]
pub struct Collectiondetail {
    #[serde(rename = "publishedfileid")]
    pub publishedfileid: String,

    #[serde(rename = "result")]
    pub result: i64,

    #[serde(rename = "children")]
    pub children: Option<Vec<CollectiondetailChild>>,
}

#[derive(Serialize, Deserialize)]
pub struct CollectiondetailChild {
    #[serde(rename = "publishedfileid")]
    pub publishedfileid: String,

    #[serde(rename = "sortorder")]
    pub sortorder: i64,

    #[serde(rename = "filetype")]
    pub filetype: i64,
}


//#################################

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublishedFileDetails {
    pub response: GetPublishedFileDetailsResponse,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPublishedFileDetailsResponse {
    pub result: i64,
    pub resultcount: i64,
    pub publishedfiledetails: Vec<Publishedfiledetail>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Publishedfiledetail {
    pub publishedfileid: String,
    pub result: i64,
    pub creator: String,
    #[serde(rename = "creator_app_id")]
    pub creator_app_id: i64,
    #[serde(rename = "consumer_app_id")]
    pub consumer_app_id: i64,
    pub filename: String,
    #[serde(rename = "file_size")]
    pub file_size: i64,
    #[serde(rename = "file_url")]
    pub file_url: String,
    #[serde(rename = "hcontent_file")]
    pub hcontent_file: String,
    #[serde(rename = "preview_url")]
    pub preview_url: String,
    #[serde(rename = "hcontent_preview")]
    pub hcontent_preview: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "time_created")]
    pub time_created: i64,
    #[serde(rename = "time_updated")]
    pub time_updated: i64,
    pub visibility: i64,
    pub banned: i64,
    #[serde(rename = "ban_reason")]
    pub ban_reason: String,
    pub subscriptions: i64,
    pub favorited: i64,
    #[serde(rename = "lifetime_subscriptions")]
    pub lifetime_subscriptions: i64,
    #[serde(rename = "lifetime_favorited")]
    pub lifetime_favorited: i64,
    pub views: i64,
    pub tags: Vec<Tag>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub tag: String,
}
