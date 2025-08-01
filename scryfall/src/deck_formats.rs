#[cfg(feature = "std")]
use std::{fs::File, io::{BufReader, Read}};
use core::error::Error;
use alloc::{boxed::Box, string::ToString, vec::Vec};
use hashbrown::{HashMap, HashSet};
use log::error;
use regex::Regex;
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

#[cfg(feature = "std")]
pub fn parse_txt_file(file: &File) -> Result<HashMap<CollectionCardIdentifier, usize>, Box<dyn Error>> {
    let mut deck_file = alloc::string::String::new();
    let mut buffered_reader = BufReader::new(file);
    buffered_reader.read_to_string(&mut deck_file)?;

    parse_txt_data(&deck_file)
}

pub fn parse_txt_data(txt_data: &str) -> Result<HashMap<CollectionCardIdentifier, usize>, Box<dyn Error>> {
    let mut cards = HashMap::new();
    let regex = Regex::new(r"(?Rm)^(?<count>\d+) (?:\[(?<set>\S+?)(?:#(?<collector_number>\d+))?\] )?(?<name>.+?)(?:\((?<arena_set>.+)\) (?<arena_collector_number>.+))?(?: <.*>)?(?: #.*)?$")?;

    for card_details in regex.captures_iter(txt_data) {
        let count: usize = if let Some(digits) = card_details.name("count") {
            digits.as_str().parse()?
        } else {
            error!("RegEx matched but no capturing group called 'count' present");
            continue;
        };

        let Some(name) = card_details.name("name").map(|matched_str| matched_str.as_str().to_string()) else {
            error!("RegEx matched but no capturing group called 'name' present");
            continue;
        };

        let set = card_details.name("set").or_else(|| card_details.name("arena_set")).or_else(|| None).map(|matched_str| matched_str.as_str().to_string());
        let collector_number = card_details.name("collector_number").or_else(|| card_details.name("arena_collector_number")).or_else(|| None).map(|matched_str| matched_str.as_str().to_string());

        if let Some(set) = set {
            if let Some(collector_number) = collector_number {
                cards.insert(CollectionCardIdentifier::CollectorNumberSet { collector_number, set }, count);
            } else {
                cards.insert(CollectionCardIdentifier::NameSet { name, set }, count);
            }
        } else {
            cards.insert(CollectionCardIdentifier::Name { name }, count);
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

#[cfg(test)]
mod tests {
    use log::LevelFilter;
    use super::*;

    extern crate test;
    use test::Bencher;

    #[test]
    fn test_parsing() {
        let mut logging_builder = colog::default_builder();
        logging_builder.filter(None, LevelFilter::Info);
        logging_builder.init();

        let test_cards = "\
Main
1 [LCI] Anim Pakal, Thousandth Moon
12 Needleverge Pathway // Pillarverge Pathway
//Main
1 [LCI#223] Anim Pakal, Thousandth Moon <cost={WU}{U}> #test comment
1 Lae'zel, Vlaakith's Champion (CLB) 29
1 Wake the Reflections (PLST) MM3-28
1 Toby, Beastie Befriender (PDSK) 35p
";

        let ground_truth: HashMap<CollectionCardIdentifier, usize> = HashMap::from_iter([
            (CollectionCardIdentifier::NameSet { name: "Anim Pakal, Thousandth Moon".to_string(), set: "LCI".to_string() }, 1),
            (CollectionCardIdentifier::Name { name: "Needleverge Pathway // Pillarverge Pathway".to_string() }, 12),
            (CollectionCardIdentifier::CollectorNumberSet { collector_number: "223".to_string(), set: "LCI".to_string() }, 1),
            (CollectionCardIdentifier::CollectorNumberSet { collector_number: "29".to_string(), set: "CLB".to_string() }, 1),
            (CollectionCardIdentifier::CollectorNumberSet { collector_number: "MM3-28".to_string(), set: "PLST".to_string() }, 1),
            (CollectionCardIdentifier::CollectorNumberSet { collector_number: "35p".to_string(), set: "PDSK".to_string() }, 1),
        ].into_iter());

        let test_card_map = parse_txt_data(test_cards).expect("Parsing of test card data failed");

        assert_eq!(test_card_map.len(), ground_truth.len());

        for (card, count) in ground_truth {
            assert_eq!(*test_card_map.get(&card).unwrap_or_else(|| panic!("Parsed card data should contain {}\nParsed card data: {:?}", card, test_card_map)), count);
        }
    }

    #[bench]
    fn benchmark_parsing_regex(b: &mut Bencher) {
        let test_cards = "\
Main
1 [LCI] Anim Pakal, Thousandth Moon
12 Needleverge Pathway // Pillarverge Pathway
//Main
1 [LCI#223] Anim Pakal, Thousandth Moon <cost={WU}{U}> #test comment
1 Lae'zel, Vlaakith's Champion (CLB) 29
1 Wake the Reflections (PLST) MM3-28
1 Toby, Beastie Befriender (PDSK) 35p
";

        b.iter(|| parse_txt_data(test_cards).expect("Parsing of test card data failed"));
    }
}
