use alloc::vec::Vec;
use hashbrown::HashMap;
use log::warn;

use crate::{api_interface::{api_classes::Card, collection_card_identifier::CollectionCardIdentifier}, token_handling::Token};
use super::ResolvedCard;

fn get_count_for_card(deck_list: &HashMap<CollectionCardIdentifier, usize>, card: &Card) -> usize {
    if let Some(count) = deck_list.get(&CollectionCardIdentifier::Id { id: card.id }) {
        return *count;
    }
    if let Some(count) = deck_list.get(&CollectionCardIdentifier::CollectorNumberSet { collector_number: card.collector_number.clone(), set: card.set.clone() }) {
        return *count;
    }
    if let Some(mtgo_id) = &card.mtgo_id && let Some(count) = deck_list.get(&CollectionCardIdentifier::MtgoId { mtgo_id: *mtgo_id }) {
        return *count;
    }
    if let Some(multiverse_ids) = &card.multiverse_ids {
        for multiverse_id in multiverse_ids {
            if let Some(count) = deck_list.get(&CollectionCardIdentifier::MultiverseId { multiverse_id: *multiverse_id }) {
                return *count;
            };
        }
    }
    if let Some(oracle_id) = &card.oracle_id && let Some(count) = deck_list.get(&CollectionCardIdentifier::OracleId { oracle_id: *oracle_id }) {
        return *count;
    }
    if let Some(illustration_id) = &card.illustration_id && let Some(count) = deck_list.get(&CollectionCardIdentifier::IllustrationId { illustration_id: *illustration_id }) {
        return *count;
    }
    if let Some(count) = deck_list.get(&CollectionCardIdentifier::NameSet { name: card.name.clone(), set: card.set.clone() }) {
        return *count;
    }
    if let Some(count) = deck_list.get(&CollectionCardIdentifier::Name { name: card.name.clone() }) {
        return *count;
    }

    if !card.is_token() {
        warn!("Could not find card {card} on the deck list, assuming it has one copy");
    }

    1
}

pub fn get_counts_for_cards(deck_list: &HashMap<CollectionCardIdentifier, usize>, cards: Vec<Card>) -> Vec<ResolvedCard> {
    cards.into_iter().map(|card| ResolvedCard { count: get_count_for_card(deck_list, &card), card }).collect()
}