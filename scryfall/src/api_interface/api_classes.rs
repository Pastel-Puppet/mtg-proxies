use core::{cmp::Ordering, fmt::Display, hash::{Hash, Hasher}};
use alloc::{borrow::ToOwned, boxed::Box, string::{String, ToString}, vec::Vec};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::collection_card_identifier::CollectionCardIdentifier;
use crate::token_handling::Token;

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
}

impl Display for ApiObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = match self {
            ApiObject::Error(error) => "Error(".to_owned() + &error.to_string() + ")",
            ApiObject::List(list) => "List(".to_owned() + &list.to_string() + ")",
            ApiObject::Card(card) => "Card(".to_owned() + &card.to_string() + ")",
            ApiObject::CardFace(card_face) => "CardFace(".to_owned() + &card_face.to_string() + ")",
            ApiObject::RelatedCard(related_card) => "RelatedCard(".to_owned() + &related_card.to_string() + ")",
            ApiObject::Deck(deck) => "Deck(".to_owned() + &deck.to_string() + ")",
            ApiObject::DeckEntry(deck_entry) => "DeckEntry(".to_owned() + &deck_entry.to_string() + ")",
            ApiObject::CardDigest(card_digest) => "CardDigest(".to_owned() + &card_digest.to_string() + ")",
        };

        write!(f, "{text}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "list")]
pub struct List {
    pub data: Vec<ApiObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_found: Option<Vec<CollectionCardIdentifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cards: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl Display for List {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = "[".to_owned() + &self.data.iter().map(|data: &ApiObject| data.to_string()).collect::<Vec<String>>().join(", ") + "]";
        write!(f, "{text}")
    }
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

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = if let Some(warnings) = &self.warnings {
            if let Some(error_type) = &self.error_type {
                "type: ".to_owned() + error_type + ", warnings: " + warnings + ", details: " + &self.details
            } else {
                "warnings: ".to_owned() + warnings + ", details: " + &self.details
            }
        } else if let Some(error_type) = &self.error_type {
            "type: ".to_owned() + error_type + ", details: " + &self.details
        } else {
            "details: ".to_owned() + &self.details
        };

        write!(f, "{text}")
    }
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
    pub prints_search_uri: String,
    pub rulings_uri: String,
    pub scryfall_uri: String,
    pub uri: String,

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
    pub purchase_uris: Option<HashMap<String, String>>,
    pub rarity: String,
    pub related_uris: HashMap<String, String>,
    pub released_at: String,
    pub reprint: bool,
    pub scryfall_set_uri: String,
    pub set_name: String,
    pub set_search_uri: String,
    pub set_type: String,
    pub set_uri: String,
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

impl Display for Card {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = self.name.clone() + " (" + &self.set + ") " + &self.collector_number;
        write!(f, "{text}")
    }
}

impl Hash for Card {
    fn hash<H: Hasher>(&self, state: &mut H) {
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        let is_self_token = self.is_token();
        let is_other_token = other.is_token();

        if is_self_token && !is_other_token {
            return Ordering::Greater;
        } else if !is_self_token && is_other_token {
            return Ordering::Less;
        }

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

impl Display for CardFace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = &self.name;
        write!(f, "{text}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "related_card")]
pub struct RelatedCard {
    pub id: Uuid,
    pub component: String,
    pub name: String,
    pub type_line: String,
    pub uri: String,
}

impl Display for RelatedCard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = &self.name;
        write!(f, "{text}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageUris {
    pub small: String,
    pub normal: String,
    pub large: String,
    pub art_crop: String,
    pub border_crop: String,
    pub png: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "deck")]
pub struct Deck {
    pub id: Uuid,
    pub name: String,
    pub format: String,
    pub layout: String,
    pub uri: String,
    pub scryfall_uri: String,
    pub description: Option<String>,
    pub trashed: bool,
    pub in_compliance: bool,
    pub sections: HashMap<String, Vec<String>>,
    pub entries: HashMap<String, Vec<DeckEntry>>,
}

impl Display for Deck {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = &self.name;
        write!(f, "{text}")
    }
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

impl Display for DeckEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = &self.raw_text;
        write!(f, "{text}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Finish {
    NoFinish(bool),
    Finish(String),
}

impl Display for Finish {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Finish::NoFinish(_) => write!(f, "no finish"),
            Finish::Finish(finish) => write!(f, "{finish}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "card_digest")]
pub struct CardDigest {
    pub id: Uuid,
    pub oracle_id: Uuid,
    pub name: String,
    pub scryfall_uri: String,
    pub mana_cost: String,
    pub type_line: String,
    pub collector_number: String,
    pub set: String,
    pub image_uris: DeckImageUris,
}

impl Display for CardDigest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = self.name.clone() + " (" + &self.set + ") " + &self.collector_number;
        write!(f, "{text}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeckImageUris {
    pub front: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back: Option<String>,
}
