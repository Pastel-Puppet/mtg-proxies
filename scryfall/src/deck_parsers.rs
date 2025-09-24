use core::error::Error;
use alloc::boxed::Box;
use hashbrown::HashMap;
use log::error;
use serde_json::from_str;
#[cfg(feature = "std")]
use regex::Regex;
#[cfg(feature = "wasm")]
use js_sys::{RegExp, Array, JsString};
#[cfg(feature = "wasm")]
use wasm_bindgen::{JsValue, JsCast};
#[cfg(feature = "wasm")]
use alloc::string::ToString;

use crate::api_interface::{api_classes::Deck, collection_card_identifier::CollectionCardIdentifier};

#[cfg(feature = "std")]
pub fn parse_txt_data(txt_data: &str) -> Result<HashMap<CollectionCardIdentifier, usize>, Box<dyn Error>> {
    let mut cards = HashMap::new();
    let regex = Regex::new(r"(?Rm)^(?<count>\d+) (?:\[(?<set>\S+?)(?:#(?<collector_number>\d+))?\] )?(?<name>.+?)(?:\((?<arena_set>.+)\) (?<arena_collector_number>\S+))?(?: \*F\*)?(?: <.*>)?(?: #.*)?$")?;

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

        let set = card_details.name("set").or_else(|| card_details.name("arena_set")).or(None).map(|matched_str| matched_str.as_str().to_string());
        let collector_number = card_details.name("collector_number").or_else(|| card_details.name("arena_collector_number")).or(None).map(|matched_str| matched_str.as_str().to_string());

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

#[cfg(feature = "wasm")]
pub fn parse_txt_data_js(txt_data: &str) -> Result<HashMap<CollectionCardIdentifier, usize>, JsValue> {
    let mut cards = HashMap::new();
    let regex = RegExp::new(r"^(?<count>\d+) (?:\[(?<set>\S+?)(?:#(?<collector_number>\d+))?\] )?(?<name>.+?)(?:\((?<arena_set>.+)\) (?<arena_collector_number>\S+))?(?: \*F\*)?(?: <.*>)?(?: #.*)?$", "gum");

    for matched_line in JsString::from(txt_data).match_all(&regex) {
        let Ok(Ok(matches_array)) = matched_line.map(|array| array.dyn_into::<Array>()) else {
            continue;
        };

        let count: usize = if let Some(digits) = matches_array.get(1).as_string() {
            digits.parse().map_err(|error: core::num::ParseIntError| error.to_string())?
        } else {
            error!("RegEx matched but no capturing group called 'count' present");
            continue;
        };

        let Some(name) = matches_array.get(4).as_string() else {
            error!("RegEx matched but no capturing group called 'name' present");
            continue;
        };

        let set = matches_array.get(2).as_string().or_else(|| matches_array.get(5).as_string()).or(None);
        let collector_number = matches_array.get(3).as_string().or_else(|| matches_array.get(6).as_string()).or(None);

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
