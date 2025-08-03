mod fetch_cards_bulk;
mod fetch_card_fuzzy;
pub mod fetch_deck;
mod fetch_tokens;
mod resolve_card_counts;

use alloc::string::String;
use core::{error::Error, fmt::Display};

use crate::api_interface::api_classes::{ApiObject, Card};

#[derive(Debug, Clone)]
pub enum CardParseError {
    ObjectNotCard(ApiObject),
    ObjectNotList(ApiObject),
    CardCountNotFound(String),
}

impl Display for CardParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ObjectNotCard(object) => write!(f, "API returned object other than a card:\n{object}"),
            Self::ObjectNotList(object) => write!(f, "API returned object other than a list:\n{object}"),
            Self::CardCountNotFound(identifier) => write!(f, "Card {identifier} could not be found in unresolved list"),
        }
    }
}

impl Error for CardParseError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedCard {
    pub count: usize,
    pub card: Card,
}

impl PartialOrd for ResolvedCard {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResolvedCard {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.card.cmp(&other.card)
    }
}

impl Display for ResolvedCard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {}", self.count, self.card.name)
    }
}
