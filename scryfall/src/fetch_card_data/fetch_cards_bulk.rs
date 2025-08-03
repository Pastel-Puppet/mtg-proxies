use core::error::Error;
use alloc::{boxed::Box, vec::Vec};
use futures::{stream::FuturesUnordered, StreamExt};
use log::warn;

use crate::{api_interface::{api_classes::{ApiObject, Card}, collection_card_identifier::CollectionCardIdentifier, ApiInterface, RequestClient}};
use super::{CardParseError, fetch_card_fuzzy::FetchCardFuzzy};

pub trait FetchCardsBulk {
    fn fetch_cards_bulk(&self, card_list: &[CollectionCardIdentifier]) -> impl Future<Output = Result<Vec<Card>, Box<dyn Error>>>;
}

impl<Client: RequestClient> FetchCardsBulk for ApiInterface<Client> {
    async fn fetch_cards_bulk(&self, card_list: &[CollectionCardIdentifier]) -> Result<Vec<Card>, Box<dyn Error>> {
        if card_list.is_empty() {
            return Ok(Vec::new());
        }

        let mut not_found_cards_list: Vec<CollectionCardIdentifier> = Vec::new();
        let mut resolved_cards: Vec<Card> = Vec::new();

        // API accepts at most 75 cards in one request.
        let num_of_chunks = card_list.len().div_ceil(75);
        let length_of_chunks = card_list.len().div_ceil(num_of_chunks);

        let mut resolved_cards_futures: FuturesUnordered<_> =
            card_list
                .chunks(length_of_chunks)
                .map(|unresolved_cards_chunk| 
                    self.get_cards_from_list(unresolved_cards_chunk)
                )
                .collect();

        while let Some(resolved_cards_chunk) = resolved_cards_futures.next().await {
            let list = match resolved_cards_chunk? {
                ApiObject::List(list) => list,
                other => return Err(Box::new(CardParseError::ObjectNotList(other))),
            };

            if let Some(not_found_cards) = list.not_found {
                not_found_cards_list.append(&mut not_found_cards.clone());
            }

            for object in list.data {
                match object {
                    ApiObject::Card(card) => {
                        resolved_cards.push(*card);
                    },
                    other => return Err(Box::new(CardParseError::ObjectNotCard(other))),
                }
            }
        }

        for not_found_card in not_found_cards_list {
            let resolved_card = self.fetch_card_fuzzy(&not_found_card).await?;
            warn!("{} did not match any card, using closest match: {}", not_found_card, resolved_card.name);

            resolved_cards.push(resolved_card);
        }

        Ok(resolved_cards)
    }
}
