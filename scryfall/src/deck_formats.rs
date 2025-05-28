use std::{collections::HashMap, error::Error, fs::File, io::{BufReader, Read}};

use log::error;
use serde_json::{from_reader, from_str};
use url::Url;

use crate::{api_classes::Deck, collection_card_identifier::CollectionCardIdentifier};

fn parse_card_name(name: &str) -> CollectionCardIdentifier {
    let name = name.trim();
    if name.starts_with("[") && name.ends_with("]") {
        let name = name.trim_start_matches("[").trim_end_matches("]");
        if let Some((set, collector_number)) = name.split_once("/") {
            CollectionCardIdentifier::CollectorNumberSet((collector_number.to_string(), set.to_string()))
        } else {
            CollectionCardIdentifier::Name(name.to_string())
        }
    } else {
        CollectionCardIdentifier::Name(name.to_string())
    }
}

fn parse_txt_line(line: String) -> Option<(CollectionCardIdentifier, usize)> {
    let text = line.trim();
    if text.is_empty() || text.starts_with("//") {
        return None
    }

    match text.split_once(" ") {
        None => Some((parse_card_name(text), 1)),
        Some((digits, card_name)) => {
            match digits.parse() {
                Err(_) => Some((parse_card_name(text), 1)),
                Ok(digits) => Some((parse_card_name(card_name), digits)),
            }
        },
    }
}

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

pub fn parse_json_file(file: &File) -> Result<HashMap<CollectionCardIdentifier, usize>, Box<dyn Error>> {
    let mut deck_file = String::new();
    let mut buffered_reader = BufReader::new(file);
    buffered_reader.read_to_string(&mut deck_file)?;

    parse_json_data(&deck_file)
}

pub fn parse_json_data(json_data: &str) -> Result<HashMap<CollectionCardIdentifier, usize>, Box<dyn Error>> {
    let mut card_map = HashMap::new();
    let deck: Deck = from_str(&json_data)?;

    for (section_name, deck_section) in deck.entries.iter() {
        if section_name == "maybeboard" {
            continue;
        }

        for card in deck_section {
            if let Some(card_digest) = &card.card_digest {
                card_map.insert(CollectionCardIdentifier::Id(card_digest.id), card.count);
                continue;
            }

            if card.found {
                error!("Could not find ID for card {}", &card.raw_text);
            }
        }
    }

    Ok(card_map)
}

pub fn images_from_json_file(file: &File, exclude_basic_lands: bool) -> Result<Vec<(Url, usize)>, Box<dyn Error>> {
    let mut image_list = Vec::new();

    let buffered_reader = BufReader::new(file);
    let deck: Deck = from_reader(buffered_reader)?;

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