use std::{collections::HashMap, hash::Hash};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "object")]
#[serde(rename_all = "snake_case")]
pub enum ApiObject {
    Error(Box<Error>),
    List(Box<List>),
    Card(Box<Card>),
    CardFace(Box<CardFace>),
    RelatedCard(Box<RelatedCard>),
    Deck(Box<Deck>),
    DeckEntry(Box<DeckEntry>),
    CardDigest(Box<CardDigest>),
    BulkData(Box<BulkData>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardNotFound {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "list")]
pub struct List {
    pub data: Vec<ApiObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_found: Option<Vec<CardNotFound>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cards: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "error")]
pub struct Error {
    pub status: usize,
    pub code: String,
    pub details: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "card")]
pub struct Card {
    // Core card fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arena_id: Option<usize>,
    pub id: Uuid,
    pub lang: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtgo_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtgo_foil_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiverse_ids: Option<Vec<usize>>,
    pub tcgplayer_id:  Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcgplayer_etched_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardmarket_id: Option<usize>,
    pub layout: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_id: Option<Uuid>,
    pub prints_search_uri: Url,
    pub rulings_uri: Url,
    pub scryfall_uri: Url,
    pub uri: Url,

    // Gameplay fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_parts: Option<Vec<RelatedCard>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_faces: Option<Vec<CardFace>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmc: Option<f32>,
    pub color_identity: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_indicator: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edhrec_rank: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_changer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_modifier: Option<String>,
    pub keywords: Vec<String>,
    pub legalities: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub life_modifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loyalty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mana_cost: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub penny_rank: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub produced_mana: Option<Vec<String>>,
    pub reserved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toughness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_line: Option<String>,

    // Print fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_ids: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attraction_lights: Option<Vec<usize>>,
    pub booster: bool,
    pub border_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_back_id: Option<Uuid>,
    pub collector_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_warning: Option<bool>,
    pub digital: bool,
    pub finishes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flavor_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flavor_text: Option<String>,
    pub foil: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_effects: Option<Vec<String>>,
    pub frame: String,
    pub full_art: bool,
    pub games: Vec<String>,
    pub highres_image: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub illustration_id: Option<Uuid>,
    pub image_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_uris: Option<ImageUris>,
    pub nonfoil: bool,
    pub oversized: bool,
    pub prices: HashMap<String, Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printed_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printed_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printed_type_line: Option<String>,
    pub promo: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promo_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_uris: Option<HashMap<String, Url>>,
    pub rarity: String,
    pub related_uris: HashMap<String, Url>,
    pub released_at: String,
    pub reprint: bool,
    pub scryfall_set_uri: Url,
    pub set_name: String,
    pub set_search_uri: Url,
    pub set_type: String,
    pub set_uri: Url,
    pub set: String,
    pub set_id: Uuid,
    pub story_spotlight: bool,
    pub textless: bool,
    pub variation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation_of: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_stamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<HashMap<String, String>>,
}

impl Hash for Card {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Card {}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "card_face")]
pub struct CardFace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmc: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_indicator: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defense: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flavor_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub illustration_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_uris: Option<ImageUris>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loyalty: Option<String>,
    pub mana_cost: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printed_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printed_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printed_type_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toughness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "related_card")]
pub struct RelatedCard {
    pub id: Uuid,
    pub component: String,
    pub name: String,
    pub type_line: String,
    pub uri: Url,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageUris {
    pub small: Url,
    pub normal: Url,
    pub large: Url,
    pub art_crop: Url,
    pub border_crop: Url,
    pub png: Url,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "deck")]
pub struct Deck {
    pub id: Uuid,
    pub name: String,
    pub format: String,
    pub layout: String,
    pub uri: Url,
    pub scryfall_uri: Url,
    pub description: Option<String>,
    pub trashed: bool,
    pub in_compliance: bool,
    pub sections: HashMap<String, Vec<String>>,
    pub entries: HashMap<String, Vec<DeckEntry>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "deck_entry")]
pub struct DeckEntry {
    pub id: Uuid,
    pub deck_id: Uuid,
    pub section: String,
    pub cardinality: f32,
    pub count: usize,
    pub raw_text: String,
    pub found: bool,
    pub printing_specified: bool,
    pub finish: Option<Finish>,
    pub card_digest: Option<CardDigest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Finish {
    NoFinish(bool),
    Finish(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "card_digest")]
pub struct CardDigest {
    pub id: Uuid,
    pub oracle_id: Uuid,
    pub name: String,
    pub scryfall_uri: Url,
    pub mana_cost: String,
    pub type_line: String,
    pub collector_number: String,
    pub set: String,
    pub image_uris: DeckImageUris,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeckImageUris {
    pub front: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back: Option<Url>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "bulk_data")]
pub struct BulkData {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub bulk_type: String,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    pub uri: Url,
    pub name: String,
    pub description: String,
    pub size: usize,
    pub download_uri: Url,
    pub content_type: String,
    pub content_encoding: String,
}