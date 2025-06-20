use core::{error::Error as ErrorTrait, fmt::Display};
use alloc::{borrow::ToOwned, boxed::Box, format, string::String, vec::Vec};
use log::{info, warn};
use serde_json::{from_str, json, Value};

use crate::{api_classes::{ApiObject, Card, Error}, collection_card_identifier::CollectionCardIdentifier};

pub trait RequestClient {
    fn build() -> Result<Self, Box<dyn ErrorTrait>>
        where Self: Sized;

    fn get(&self, url: String) -> impl core::future::Future<Output = Result<String, Box<dyn ErrorTrait>>>;
    fn get_with_parameters(&self, url: String, query_parameters: &[(&str, &str)]) -> impl core::future::Future<Output = Result<String, Box<dyn ErrorTrait>>>;
    fn post(&self, url: String, payload: &Value) -> impl core::future::Future<Output = Result<String, Box<dyn ErrorTrait>>>;
}

#[derive(Debug, Clone)]
pub struct InvalidCardIdentifierError;

impl Display for InvalidCardIdentifierError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Oracle IDs and illustration IDs cannot be used to retrieve specific cards")
    }
}

impl ErrorTrait for InvalidCardIdentifierError {}

#[derive(Debug, Clone)]
pub struct ApiError {
    error: Error,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Received an error from an API request:\n{}", self.error)
    }
}

impl ErrorTrait for ApiError {}

#[derive(Debug, Clone)]
pub struct InvalidApiObjectError {
    expected: &'static str,
    received: ApiObject,
}

impl Display for InvalidApiObjectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Expected {}, received from the API:\n{}", self.expected, self.received)
    }
}

impl ErrorTrait for InvalidApiObjectError {}

static NAMED_CARD_METHOD: &str = "/cards/named";
static SPECIFIED_CARD_METHOD: &str = "/cards";
static MULTIVERSE_CARD_METHOD: &str = "/cards/multiverse";
static MTGO_CARD_METHOD: &str = "/cards/mtgo";
static CARD_COLLECTION_METHOD: &str = "/cards/collection";
static BULK_DATA_METHOD: &str = "/bulk-data/all-cards";

pub struct ApiInterface<Client>
    where Client: RequestClient {
    http_client: Client,
    api_endpoint: String,
}

impl<Client> ApiInterface<Client>
    where Client: RequestClient {
    pub fn new() -> Result<Self, Box<dyn ErrorTrait>> {
        Ok(Self {
            http_client: Client::build()?,
            api_endpoint: "https://api.scryfall.com/".to_owned(),
        })
    }

    pub async fn get_card(&mut self, card: &CollectionCardIdentifier) -> Result<ApiObject, Box<dyn ErrorTrait>> {
        info!("Sending API request for card {}", card);

        let response = match card {
            CollectionCardIdentifier::Id(uuid) => {
                self.http_client.get(format!("{}/{}/{}", self.api_endpoint, SPECIFIED_CARD_METHOD, uuid)).await?
            },
            CollectionCardIdentifier::MtgoId(id) => {
                self.http_client.get(format!("{}/{}/{}", self.api_endpoint, MTGO_CARD_METHOD, id)).await?
            },
            CollectionCardIdentifier::MultiverseId(id) => {
                self.http_client.get(format!("{}/{}/{}", self.api_endpoint, MULTIVERSE_CARD_METHOD, id)).await?
            },
            CollectionCardIdentifier::OracleId(_) |
            CollectionCardIdentifier::IllustrationId(_) => return Err(Box::new(InvalidCardIdentifierError)),
            CollectionCardIdentifier::Name(name) => {
                self.http_client.get_with_parameters(format!("{}/{}", self.api_endpoint, NAMED_CARD_METHOD), &[("fuzzy", name)]).await?
            },
            CollectionCardIdentifier::NameSet((name, set)) => {
                self.http_client.get_with_parameters(format!("{}/{}", self.api_endpoint, NAMED_CARD_METHOD), &[("fuzzy", name), ("set", set)]).await?
            },
            CollectionCardIdentifier::CollectorNumberSet((collector_number, set)) => {
                self.http_client.get(format!("{}/{}/{}/{}", self.api_endpoint, SPECIFIED_CARD_METHOD, set, collector_number)).await?
            },
        };

        let api_object = from_str(&response)?;
        if let ApiObject::Error(error) = api_object {
            Err(Box::new(ApiError { error: *error }))
        } else {
            Ok(api_object)
        }
    }

    pub async fn get_cards_from_list(&mut self, identifiers: &[&CollectionCardIdentifier]) -> Result<ApiObject, Box<dyn ErrorTrait>> {
        let identifiers_json = json!({
            "identifiers": identifiers
        });

        info!("Sending API request for {} cards", identifiers.len());

        let response = self.http_client.post(format!("{}/{}", self.api_endpoint, CARD_COLLECTION_METHOD), &identifiers_json).await?;

        let api_object = from_str(&response)?;
        if let ApiObject::Error(error) = api_object {
            Err(Box::new(ApiError { error: *error }))
        } else {
            Ok(api_object)
        }
    }

    async fn resolve_multi_page_search(&mut self, search_url: String) -> Result<Vec<ApiObject>, Box<dyn ErrorTrait>> {
        info!("Sending API request for next page of results");

        let response = self.http_client.get(search_url).await?;

        let api_object = from_str(&response)?;
        if let ApiObject::Error(error) = api_object {
            return Err(Box::new(ApiError { error: *error }));
        }
        
        let ApiObject::List(mut current_page) = api_object else {
            return Err(Box::new(InvalidApiObjectError { expected: "List", received: api_object }));
        };
        
        let Some(has_more) = current_page.has_more else {
            return Ok(current_page.data);
        };

        if !has_more {
            return Ok(current_page.data);
        }

        let Some(next_page_url) = current_page.next_page else {
            warn!("Current page claims to have more data but fetch URL is absent");
            return Ok(current_page.data);
        };

        current_page.data.append(&mut Box::pin(self.resolve_multi_page_search(next_page_url)).await?);
        Ok(current_page.data)
    }

    pub async fn get_all_printings(&mut self, prints_search_uri: String, card_name: String) -> Result<Vec<Card>, Box<dyn ErrorTrait>> {
        info!("Sending API request for all printings of {}", card_name);

        let search_results = self.resolve_multi_page_search(prints_search_uri).await?;

        let mut card_printings = Vec::new();
        for api_object in search_results {
            let ApiObject::Card(card_printing) = api_object else {
                return Err(Box::new(InvalidApiObjectError { expected: "Card", received: api_object }))
            };

            card_printings.push(*card_printing);
        }

        Ok(card_printings)
    }

    async fn get_bulk_data_endpoint(&mut self) -> Result<ApiObject, Box<dyn ErrorTrait>> {
        info!("Sending API request for bulk data endpoints");

        let response = self.http_client.get(format!("{}/{}", self.api_endpoint, BULK_DATA_METHOD)).await?;

        let api_object = from_str(&response)?;
        if let ApiObject::Error(error) = api_object {
            Err(Box::new(ApiError { error: *error }))
        } else {
            Ok(api_object)
        }
    }

    pub async fn get_bulk_data(&mut self) -> Result<String, Box<dyn ErrorTrait>> {
        let received_object = self.get_bulk_data_endpoint().await?;
        let ApiObject::BulkData(bulk_data_endpoint) = received_object else {
            return Err(Box::new(InvalidApiObjectError { expected: "BulkData",  received: received_object }));
        };

        info!("Sending API request for bulk data");
        let response = self.http_client.get(bulk_data_endpoint.download_uri).await?;

        Ok(response)
    }
}