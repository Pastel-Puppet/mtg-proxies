use alloc::{boxed::Box, vec::Vec};
use core::error::Error;
use hashbrown::HashMap;
use log::warn;
use uuid::Uuid;

use crate::{api_interface::{api_classes::Card, collection_card_identifier::CollectionCardIdentifier, ApiInterface, RequestClient}, token_handling::Token};
use super::fetch_cards_bulk::FetchCardsBulk;

pub trait FetchRelatedTokens {
    fn fetch_related_tokens(&self, cards: &[Card]) -> impl Future<Output = Result<Vec<Card>, Box<dyn Error>>>;
}

impl<Client: RequestClient> FetchRelatedTokens for ApiInterface<Client> {
    async fn fetch_related_tokens(&self, cards: &[Card]) -> Result<Vec<Card>, Box<dyn Error>> {
        let mut related_tokens: Vec<CollectionCardIdentifier> = Vec::new();
        
        for card in cards {
            if let Some(related_cards) = &card.all_parts {
                for related_card in related_cards {
                    if related_card.is_token() {
                        related_tokens.push(CollectionCardIdentifier::Id { id: related_card.id });
                    }
                }
            }
        }

        let tokens = self.fetch_cards_bulk(&related_tokens).await?;
        let token_oracle_ids: HashMap<Uuid, Card> = HashMap::from_iter(tokens.into_iter().filter_map(|card| {
            if let Some(oracle_id) = card.oracle_id {
                Some((oracle_id, card))
            } else {
                warn!("Dropping token {} as it has no oracle ID (Scryfall URL: {})", card.name, card.scryfall_uri);
                None
            }
        }));

        Ok(token_oracle_ids.into_values().collect())
    }
}
