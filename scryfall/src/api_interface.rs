use std::{error::Error as ErrorTrait, fmt::Display, thread::sleep, time::{Duration, Instant}};
use serde_json::{from_str, json, Value};
use url::Url;
use url_macro::url;

use crate::{api_classes::{ApiObject, Card, Error}, collection_card_identifier::CollectionCardIdentifier};

pub trait RequestClient {
    fn build() -> Result<Self, Box<dyn ErrorTrait>>
        where Self: Sized;

    fn get(&self, url: Url) -> Result<String, Box<dyn ErrorTrait>>;
    fn get_with_parameters(&self, url: Url, query_parameters: &[(&str, &str)]) -> Result<String, Box<dyn ErrorTrait>>;
    fn post(&self, url: Url, payload: &Value) -> Result<String, Box<dyn ErrorTrait>>;
}

#[derive(Debug, Clone)]
pub struct InvalidCardIdentifierError;

impl Display for InvalidCardIdentifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Oracle IDs and illustration IDs cannot be used to retrieve specific cards")
    }
}

impl ErrorTrait for InvalidCardIdentifierError {}

#[derive(Debug, Clone)]
pub struct ApiError {
    error: Error,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Received an error from an API request:\n{:?}", self.error)
    }
}

impl ErrorTrait for ApiError {}

#[derive(Debug, Clone)]
pub struct InvalidApiObjectError {
    expected: &'static str,
    received: ApiObject,
}

impl Display for InvalidApiObjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected {}, received from the API:\n{:?}", self.expected, self.received)
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
    api_endpoint: Url,
    last_request_time: Option<Instant>,
    verbose: bool,
}

impl<Client> ApiInterface<Client>
    where Client: RequestClient {
    pub fn new(verbose: bool) -> Result<Self, Box<dyn ErrorTrait>> {
        Ok(Self {
            http_client: Client::build()?,
            api_endpoint: url!("https://api.scryfall.com/"),
            last_request_time: None,
            verbose,
        })
    }

    fn rate_limit(&mut self) {
        if let Some(last_request_time) = self.last_request_time {
            if last_request_time.elapsed() < Duration::from_millis(100) {
                println!("Sleeping briefly for rate limiting");
                sleep(Duration::from_millis(100) - last_request_time.elapsed());
            }
        } else {
            self.last_request_time = Some(Instant::now());
        }
    }

    pub fn get_card(&mut self, card: &CollectionCardIdentifier) -> Result<ApiObject, Box<dyn ErrorTrait>> {
        self.rate_limit();

        if self.verbose {
            println!("Sending API request for card {:?}", card);
        }

        let response = match card {
            CollectionCardIdentifier::Id(uuid) => {
                self.http_client.get(self.api_endpoint.join(format!("{}/{}", SPECIFIED_CARD_METHOD, uuid).as_str())?)?
            },
            CollectionCardIdentifier::MtgoId(id) => {
                self.http_client.get(self.api_endpoint.join(format!("{}/{}", MTGO_CARD_METHOD, id).as_str())?)?
            },
            CollectionCardIdentifier::MultiverseId(id) => {
                self.http_client.get(self.api_endpoint.join(format!("{}/{}", MULTIVERSE_CARD_METHOD, id).as_str())?)?
            },
            CollectionCardIdentifier::OracleId(_) |
            CollectionCardIdentifier::IllustrationId(_) => return Err(Box::new(InvalidCardIdentifierError)),
            CollectionCardIdentifier::Name(name) => {
                self.http_client.get_with_parameters(self.api_endpoint.join(NAMED_CARD_METHOD)?, &[("fuzzy", name)])?
            },
            CollectionCardIdentifier::NameSet((name, set)) => {
                self.http_client.get_with_parameters(self.api_endpoint.join(NAMED_CARD_METHOD)?, &[("fuzzy", name), ("set", set)])?
            },
            CollectionCardIdentifier::CollectorNumberSet((collector_number, set)) => {
                self.http_client.get(self.api_endpoint.join(format!("{}/{}/{}", SPECIFIED_CARD_METHOD, set, collector_number).as_str())?)?
            },
        };

        let api_object = from_str(&response)?;
        if let ApiObject::Error(error) = api_object {
            Err(Box::new(ApiError { error: *error }))
        } else {
            Ok(api_object)
        }
    }

    pub fn get_cards_from_list(&mut self, identifiers: &[&CollectionCardIdentifier]) -> Result<ApiObject, Box<dyn ErrorTrait>> {
        let identifiers_json = json!({
            "identifiers": identifiers
        });

        self.rate_limit();

        if self.verbose {
            println!("Sending API request for multiple cards");
        }

        let response = self.http_client.post(self.api_endpoint.join(CARD_COLLECTION_METHOD)?, &identifiers_json)?;

        let api_object = from_str(&response)?;
        if let ApiObject::Error(error) = api_object {
            Err(Box::new(ApiError { error: *error }))
        } else {
            Ok(api_object)
        }
    }

    fn resolve_multi_page_search(&mut self, search_url: Url) -> Result<Vec<ApiObject>, Box<dyn ErrorTrait>> {
        self.rate_limit();

        if self.verbose {
            println!("Sending API request for next page of results");
        }

        let response = self.http_client.get(search_url)?;

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
            println!("Current page claims to have more data but fetch URL is absent");
            return Ok(current_page.data);
        };

        current_page.data.append(&mut self.resolve_multi_page_search(next_page_url)?);
        Ok(current_page.data)
    }

    pub fn get_all_printings(&mut self, card: Card) -> Result<Vec<Card>, Box<dyn ErrorTrait>> {
        if self.verbose {
            println!("Sending API request for all printings of {}", card.name);
        }

        let search_results = self.resolve_multi_page_search(card.prints_search_uri)?;

        let mut card_printings = Vec::new();
        for api_object in search_results {
            let ApiObject::Card(card_printing) = api_object else {
                return Err(Box::new(InvalidApiObjectError { expected: "Card", received: api_object }))
            };

            card_printings.push(*card_printing);
        }

        Ok(card_printings)
    }

    fn get_bulk_data_endpoint(&mut self) -> Result<ApiObject, Box<dyn ErrorTrait>> {
        self.rate_limit();

        if self.verbose {
            println!("Sending API request for bulk data endpoints");
        }

        let response = self.http_client.get(self.api_endpoint.join(BULK_DATA_METHOD)?)?;

        let api_object = from_str(&response)?;
        if let ApiObject::Error(error) = api_object {
            Err(Box::new(ApiError { error: *error }))
        } else {
            Ok(api_object)
        }
    }

    pub fn get_bulk_data(&mut self) -> Result<String, Box<dyn ErrorTrait>> {
        let received_object = self.get_bulk_data_endpoint()?;
        let ApiObject::BulkData(bulk_data_endpoint) = received_object else {
            return Err(Box::new(InvalidApiObjectError { expected: "BulkData",  received: received_object }));
        };

        if self.verbose {
            println!("Sending API request for bulk data");
        }
        let response = self.http_client.get(bulk_data_endpoint.download_uri)?;

        Ok(response)
    }
}