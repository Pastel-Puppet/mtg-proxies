use core::{error::Error, fmt::Display};
use alloc::{boxed::Box, format, string::String, vec::Vec};
use futures::{stream::FuturesUnordered, StreamExt};
use hashbrown::HashMap;
use log::warn;

use crate::{api_classes::{ApiObject, Card}, api_interface::{ApiInterface, RequestClient}, collection_card_identifier::CollectionCardIdentifier, token_handling::Token};

#[derive(Debug, Clone)]
enum CardParseErrorCause {
    ObjectNotCard(ApiObject),
    ObjectNotList(ApiObject),
    CardCountNotFound(String),
}

#[derive(Debug, Clone)]
pub struct CardParseError {
    cause: CardParseErrorCause,
}

impl Display for CardParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.cause {
            CardParseErrorCause::ObjectNotCard(object) => write!(f, "API returned object other than a card:\n{object}"),
            CardParseErrorCause::ObjectNotList(object) => write!(f, "API returned object other than a list:\n{object}"),
            CardParseErrorCause::CardCountNotFound(identifier) => write!(f, "Card {identifier} could not be found in unresolved list"),
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

fn get_count_for_card(card_map: &HashMap<CollectionCardIdentifier, usize>, card: &Card) -> Option<usize> {
    if let Some(count) = get_count_for_card_identifier(card_map, &CollectionCardIdentifier::Id { id: card.id }, false) {
        return Some(count)
    };
    if let Some(count) = get_count_for_card_identifier(card_map, &CollectionCardIdentifier::CollectorNumberSet { collector_number: card.collector_number.clone(), set: card.set.clone() }, false) {
        return Some(count)
    };
    if let Some(mtgo_id) = &card.mtgo_id && let Some(count) = get_count_for_card_identifier(card_map, &CollectionCardIdentifier::MtgoId { mtgo_id: *mtgo_id }, false) {
        return Some(count)
    };
    if let Some(multiverse_ids) = &card.multiverse_ids {
        for multiverse_id in multiverse_ids {
            if let Some(count) = get_count_for_card_identifier(card_map, &CollectionCardIdentifier::MultiverseId { multiverse_id: *multiverse_id }, false) {
                return Some(count)
            };
        }
    }
    if let Some(oracle_id) = &card.oracle_id && let Some(count) = get_count_for_card_identifier(card_map, &CollectionCardIdentifier::OracleId { oracle_id: *oracle_id }, false) {
        return Some(count)
    };
    if let Some(illustration_id) = &card.illustration_id && let Some(count) = get_count_for_card_identifier(card_map, &CollectionCardIdentifier::IllustrationId { illustration_id: *illustration_id }, false) {
        return Some(count)
    };
    if let Some(count) = get_count_for_card_identifier(card_map, &CollectionCardIdentifier::NameSet { name: card.name.clone(), set: card.set.clone() }, false) {
        return Some(count)
    };
    if let Some(count) = get_count_for_card_identifier(card_map, &CollectionCardIdentifier::Name { name: card.name.clone() }, true) {
        return Some(count)
    };
    None
}

fn get_count_for_card_identifier(card_map: &HashMap<CollectionCardIdentifier, usize>, card_identifier: &CollectionCardIdentifier, use_default: bool) -> Option<usize> {
    match card_map.get(card_identifier) {
        Some(count) => Some(*count),
        None => {
            if use_default {
                warn!("Could not find card {card_identifier} on the deck list, assuming it has one copy");
                Some(1)
            } else {
                None
            }
        },
    }
}

async fn fuzzy_resolve<Client: RequestClient>(card_map: &HashMap<CollectionCardIdentifier, usize>, api_interface: &ApiInterface<Client>, identifier: &CollectionCardIdentifier) -> Result<ResolvedCard, Box<dyn Error>> {
    let Some(count) = get_count_for_card_identifier(card_map, identifier, true) else {
        return Err(Box::new(CardParseError { cause: CardParseErrorCause::CardCountNotFound(format!("{identifier}")) }));
    };

    let object = api_interface.get_card(identifier).await?;
    if let ApiObject::Card(card) = object {
        Ok(ResolvedCard { count, card: *card })
    } else {
        Err(Box::new(CardParseError { cause: CardParseErrorCause::ObjectNotCard(object) }))
    }
}

pub async fn resolve_cards<Client: RequestClient>(card_map: &HashMap<CollectionCardIdentifier, usize>, fetch_related_tokens: bool, api_interface: &ApiInterface<Client>) -> Result<Vec<ResolvedCard>, Box<dyn Error>> {
    if card_map.is_empty() {
        return Ok(Vec::new());
    }

    let mut not_found_cards_list: Vec<CollectionCardIdentifier> = Vec::new();
    let mut resolved_cards: Vec<ResolvedCard> = Vec::new();
    let unresolved_cards: Vec<&CollectionCardIdentifier> = card_map.keys().collect();
    let mut related_tokens: HashMap<CollectionCardIdentifier, usize> = HashMap::new();

    let num_of_chunks = unresolved_cards.len().div_ceil(75);
    let length_of_chunks = unresolved_cards.len().div_ceil(num_of_chunks);

    let mut resolved_cards_futures: FuturesUnordered<_> =
        unresolved_cards
            .chunks(length_of_chunks)
            .map(|unresolved_cards_chunk| 
                api_interface.get_cards_from_list(unresolved_cards_chunk)
            )
            .collect();

    while let Some(resolved_cards_chunk) = resolved_cards_futures.next().await {
        let list = match resolved_cards_chunk? {
            ApiObject::List(list) => list,
            other => return Err(Box::new(CardParseError { cause: CardParseErrorCause::ObjectNotList(other) })),
        };

        if let Some(not_found_cards) = list.not_found {
            not_found_cards_list.append(&mut not_found_cards.clone());
        }

        for object in list.data {
            match object {
                ApiObject::Card(card) => {
                    let Some(count) = get_count_for_card(card_map, &card) else {
                        return Err(Box::new(CardParseError { cause: CardParseErrorCause::CardCountNotFound(card.name) }));
                    };

                    if fetch_related_tokens && let Some(related_cards) = &card.all_parts {
                        for related_card in related_cards {
                            if related_card.is_token() {
                                related_tokens.insert(CollectionCardIdentifier::Id { id: related_card.id }, 1);
                            }
                        }
                    }

                    resolved_cards.push(ResolvedCard { count, card: *card });
                },
                other => return Err(Box::new(CardParseError { cause: CardParseErrorCause::ObjectNotCard(other) })),
            }
        }
    }

    drop(resolved_cards_futures);

    for not_found_card in not_found_cards_list {
        let resolved_card = fuzzy_resolve(card_map, api_interface, &not_found_card).await?;
        warn!("{} did not match any card, using closest match: {}", not_found_card, resolved_card.card.name);

        if fetch_related_tokens && let Some(related_cards) = &resolved_card.card.all_parts {
            for related_card in related_cards {
                if related_card.is_token() {
                    related_tokens.insert(CollectionCardIdentifier::Id { id: related_card.id }, 1);
                }
            }
        }

        resolved_cards.push(resolved_card);
    }

    if fetch_related_tokens {
        let tokens = Box::pin(resolve_cards(&related_tokens, false, api_interface)).await?;
        let token_oracle_ids: HashMap<uuid::Uuid, ResolvedCard> = HashMap::from_iter(tokens.into_iter().filter_map(|card| {
            if let Some(oracle_id) = card.card.oracle_id {
                Some((oracle_id, card))
            } else {
                warn!("Dropping token {} as it has no oracle ID (Scryfall URL: {})", card.card.name, card.card.scryfall_uri);
                None
            }
        }));

        resolved_cards.append(&mut token_oracle_ids.into_values().collect());
    }

    Ok(resolved_cards)
}