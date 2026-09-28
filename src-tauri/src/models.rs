use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RuneItem {
    pub id: u32,
    pub name: String,
    pub icon_url: String,
    pub slot: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RuneTree {
    pub primary_style_id: u32,
    pub primary_style_name: String,
    pub primary_style_icon: String,
    pub keystone_id: u32,
    pub keystone_name: String,
    pub keystone_icon: String,
    pub primary_runes: Vec<RuneItem>,
    pub secondary_style_id: u32,
    pub secondary_style_name: String,
    pub secondary_style_icon: String,
    pub secondary_runes: Vec<RuneItem>,
    pub shards: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemInfo {
    pub id: u32,
    pub name: String,
    pub icon_url: String,
    pub cost: u32,
    #[serde(default)]
    pub win_rate: Option<f64>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SummonerSpell {
    pub id: String,
    pub name: String,
    pub icon_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BuildRecommendation {
    pub starting_items: Vec<ItemInfo>,
    pub boots: ItemInfo,
    pub core_items: Vec<ItemInfo>,
    #[serde(default)]
    pub fourth_items: Vec<ItemInfo>,
    #[serde(default)]
    pub fifth_items: Vec<ItemInfo>,
    #[serde(default)]
    pub sixth_items: Vec<ItemInfo>,
    pub situational_items: Vec<ItemInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChampionRoleData {
    pub id: String,
    pub champion_id: String,
    pub key: String,
    pub name: String,
    pub title: String,
    pub role: String, // "TOP", "JUNGLE", "MID", "CARRY", "SUPPORT"
    pub tier: String, // "S+", "S", "A", "B", "C", "D"
    pub win_rate: f64,
    pub pick_rate: f64,
    pub ban_rate: f64,
    pub score: f64,
    pub meta_score: f64,
    pub icon_url: String,
    pub splash_url: String,
    pub runes: RuneTree,
    pub build: BuildRecommendation,
    pub skill_order: Vec<String>,
    pub summoner_spells: Vec<SummonerSpell>,
    #[serde(default)]
    pub patch: Option<String>,
    #[serde(default)]
    pub is_cached: Option<bool>,
}

pub type ChampionStat = ChampionRoleData;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoLMetaData {
    pub patch: String,
    pub server: String,
    pub tier: String,
    pub last_updated: String,
    pub champions: Vec<ChampionRoleData>,
    pub is_cached: bool,
}
