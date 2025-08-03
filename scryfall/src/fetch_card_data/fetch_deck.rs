use alloc::{boxed::Box, vec::Vec};
use core::error::Error;
use hashbrown::HashMap;

use crate::api_interface::{collection_card_identifier::CollectionCardIdentifier, ApiInterface, RequestClient};
use super::{fetch_cards_bulk::FetchCardsBulk, fetch_tokens::FetchRelatedTokens, resolve_card_counts::get_counts_for_cards, ResolvedCard};

pub trait FetchDeck {
    fn fetch_deck(&self, deck_list: &HashMap<CollectionCardIdentifier, usize>, fetch_related_tokens: bool) -> impl Future<Output = Result<Vec<ResolvedCard>, Box<dyn Error>>>;
}

impl<Client: RequestClient> FetchDeck for ApiInterface<Client> {
    async fn fetch_deck(&self, deck_list: &HashMap<CollectionCardIdentifier, usize>, fetch_related_tokens: bool) -> Result<Vec<ResolvedCard>, Box<dyn Error>> {
        let card_list: Vec<CollectionCardIdentifier> = deck_list.keys().cloned().collect();
        let mut cards = self.fetch_cards_bulk(&card_list).await?;

        if fetch_related_tokens {
            cards.append(&mut self.fetch_related_tokens(&cards).await?);
        }

        Ok(get_counts_for_cards(deck_list, cards))
    }
}