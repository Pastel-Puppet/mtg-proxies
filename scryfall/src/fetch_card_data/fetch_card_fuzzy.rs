use alloc::boxed::Box;
use core::{error::Error, future::Future};

use crate::api_interface::{api_classes::{ApiObject, Card}, collection_card_identifier::CollectionCardIdentifier, ApiInterface, RequestClient};
use super::CardParseError;

pub trait FetchCardFuzzy {
    fn fetch_card_fuzzy(&self, card: &CollectionCardIdentifier) -> impl Future<Output = Result<Card, Box<dyn Error>>>;
}

impl<Client: RequestClient> FetchCardFuzzy for ApiInterface<Client> {
    async fn fetch_card_fuzzy(&self, card: &CollectionCardIdentifier) -> Result<Card, Box<dyn Error>> {
        let object = self.get_card(card).await?;

        if let ApiObject::Card(resolved_card) = object {
            Ok(*resolved_card)
        } else {
            Err(Box::new(CardParseError::ObjectNotCard(object)))
        }
    }
}
