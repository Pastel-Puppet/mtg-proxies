#[cfg(feature = "std")]
use std::{fs::File, io::{BufReader, Read}};
use core::error::Error;
use alloc::{boxed::Box, string::{String, ToString}, vec::Vec};
use hashbrown::{HashMap, HashSet};
use log::error;
use serde_json::from_str;

use crate::{api_classes::{Card, Deck}, collection_card_identifier::CollectionCardIdentifier, fetch_card_list::ResolvedCard};

pub struct DeckDiff {
    pub unchanged: Vec<Card>,
    pub added: Vec<Card>,
    pub removed: Vec<Card>,
}

pub fn deck_diff(old_deck: Vec<ResolvedCard>, new_deck: Vec<ResolvedCard>) -> DeckDiff {
    let new_cards_set: HashSet<ResolvedCard> = HashSet::from_iter(new_deck.iter().flat_map(|card| {
        let mut cards = Vec::new();
        for i in 0..card.count {
            cards.push(ResolvedCard { count: i, card: card.card.clone() });
        }
        cards
    }));
    let old_cards_set: HashSet<ResolvedCard> = HashSet::from_iter(old_deck.iter().flat_map(|card| {
        let mut cards = Vec::new();
        for i in 0..card.count {
            cards.push(ResolvedCard { count: i, card: card.card.clone() });
        }
        cards
    }));

    let unchanged: Vec<Card> = new_cards_set.union(&old_cards_set).map(|card| card.card.clone()).collect();
    let added: Vec<Card> = new_cards_set.difference(&old_cards_set).map(|card| card.card.clone()).collect();
    let removed: Vec<Card> = old_cards_set.difference(&new_cards_set).map(|card| card.card.clone()).collect();

    DeckDiff { unchanged, added, removed }
}

fn parse_card_name(name: &str) -> CollectionCardIdentifier {
    let name = name.trim();

    // Check for and parse cards with [SET] or [SET#NUM] formatting.
    match name.strip_prefix("[") {
        Some(name_strip) => match name_strip.split_once("] ") {
                None => CollectionCardIdentifier::Name { name: name.to_string() },
                Some((set_identifier, name_clean)) => {
                    match set_identifier.split_once("#") {
                        None => CollectionCardIdentifier::NameSet { name: name_clean.to_string(), set: set_identifier.to_string() },
                        Some((set, collector_number)) => CollectionCardIdentifier::CollectorNumberSet { collector_number: collector_number.to_string(), set: set.to_string() },
                    }
                },
            },
        None => CollectionCardIdentifier::Name { name: name.to_string() },
    }
}

fn try_parse_line_as_mtg_arena(line: &str) -> Option<(CollectionCardIdentifier, usize)> {
    let line = match line.strip_suffix(" *F*") {
        Some(stripped_line) => stripped_line,
        None => line,
    };

    let mut line_split = line.rsplit(" ");

    let collector_number_string = line_split.next()?;
    let set_identifier = line_split.next()?.strip_prefix("(")?.strip_suffix(")")?;
    let Ok(card_count) = line_split.last()?.parse::<usize>() else {
        return Some((CollectionCardIdentifier::CollectorNumberSet { collector_number: collector_number_string.to_string(), set: set_identifier.to_ascii_lowercase() }, 1));
    };

    Some((CollectionCardIdentifier::CollectorNumberSet { collector_number: collector_number_string.to_string(), set: set_identifier.to_ascii_lowercase() }, card_count))
}

fn parse_txt_line(line: String) -> Option<(CollectionCardIdentifier, usize)> {
    let (count, mut text) = match line.trim().split_once(" ") {
        None => return None,
        Some((digits, text)) => {
            match digits.parse() {
                Err(_) => return None,
                Ok(digits) => (digits, text),
            }
        },
    };

    text = match text.split_once(" #") {
        None => text,
        Some((main_text, _comment)) => main_text,
    };

    text = match text.split_once(" <") {
        None => text,
        Some((main_text, _overrides)) => main_text,
    };

    if let Some(identifier) = try_parse_line_as_mtg_arena(text) {
        return Some(identifier);
    }

    Some((parse_card_name(text), count))
}

#[cfg(feature = "std")]
pub fn parse_txt_file(file: &File) -> Result<HashMap<CollectionCardIdentifier, usize>, Box<dyn Error>> {
    let mut deck_file = String::new();
    let mut buffered_reader = BufReader::new(file);
    buffered_reader.read_to_string(&mut deck_file)?;

    parse_txt_data(&deck_file)
}

pub fn parse_txt_data(txt_data: &str) -> Result<HashMap<CollectionCardIdentifier, usize>, Box<dyn Error>> {
    let mut cards = HashMap::new();

    for deck_line in txt_data.lines() {
        if let Some((card, count)) = parse_txt_line(deck_line.to_string()) {
            cards.insert(card, count);
        }
    }

    Ok(cards)
}

#[cfg(feature = "std")]
pub fn parse_json_file(file: &File) -> Result<HashMap<CollectionCardIdentifier, usize>, Box<dyn Error>> {
    let mut deck_file = String::new();
    let mut buffered_reader = BufReader::new(file);
    buffered_reader.read_to_string(&mut deck_file)?;

    parse_json_data(&deck_file)
}

pub fn parse_json_data(json_data: &str) -> Result<HashMap<CollectionCardIdentifier, usize>, Box<dyn Error>> {
    let mut card_map = HashMap::new();
    let deck: Deck = from_str(json_data)?;

    for (section_name, deck_section) in deck.entries.iter() {
        if section_name == "maybeboard" {
            continue;
        }

        for card in deck_section {
            if let Some(card_digest) = &card.card_digest {
                card_map.insert(CollectionCardIdentifier::Id { id: card_digest.id }, card.count);
                continue;
            }

            if card.found {
                error!("Could not find ID for card {}", &card.raw_text);
            }
        }
    }

    Ok(card_map)
}

#[cfg(feature = "std")]
pub fn images_from_json_file(file: &File, exclude_basic_lands: bool) -> Result<Vec<(String, usize)>, Box<dyn Error>> {
    let mut image_list = Vec::new();

    let buffered_reader = BufReader::new(file);
    let deck: Deck = serde_json::from_reader(buffered_reader)?;

    for (section_name, deck_section) in deck.entries.iter() {
        if section_name == "maybeboard" {
            continue;
        }

        for card in deck_section {
            if let Some(card_digest) = &card.card_digest {
                if exclude_basic_lands && card_digest.type_line.starts_with("Basic Land") {
                    continue;
                }

                image_list.push((card_digest.image_uris.front.clone(), card.count));
                if let Some(back_image) = card_digest.image_uris.back.clone() {
                    image_list.push((back_image, card.count));
                }
                continue;
            }

            if card.found {
                error!("Could not find image for card {}", &card.raw_text);
            }
        }
    }

    Ok(image_list)
}
