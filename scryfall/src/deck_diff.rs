use alloc::vec::Vec;
use hashbrown::HashSet;

use crate::{api_interface::api_classes::Card, fetch_card_data::ResolvedCard};

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