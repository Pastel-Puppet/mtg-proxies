use std::{error::Error as ErrorTrait, fmt::Display, fs::File, io::Write, path::PathBuf, thread::sleep, time::{Duration, Instant}};
use reqwest::{header::ACCEPT, blocking::Client, Url};
use serde_json::{from_str, json};
use url_macro::url;

use crate::{api_classes::{ApiObject, Error}, collection_card_identifier::CollectionCardIdentifier};

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

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

static NAMED_CARD_METHOD: &str = "/cards/named";
static SPECIFIED_CARD_METHOD: &str = "/cards";
static MULTIVERSE_CARD_METHOD: &str = "/cards/multiverse";
static MTGO_CARD_METHOD: &str = "/cards/mtgo";
static CARD_COLLECTION_METHOD: &str = "/cards/collection";
static BULK_DATA_METHOD: &str = "/bulk-data/all-cards";

pub struct ApiInterface {
    http_client: Client,
    api_endpoint: Url,
    last_request_time: Option<Instant>,
    verbose: bool,
}

impl ApiInterface {
    pub fn new(verbose: bool) -> Result<ApiInterface, Box<dyn ErrorTrait>> {
        let builder = Client::builder()
            .user_agent(APP_USER_AGENT);

        Ok(ApiInterface {
            http_client: builder.build()?,
            api_endpoint: url!("https://api.scryfall.com/"),
            last_request_time: None,
            verbose
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
                self.http_client.get(self.api_endpoint.join(format!("{}/{}", SPECIFIED_CARD_METHOD, uuid).as_str())?)
                    .header(ACCEPT, "application/json")
                    .send()?
                    .text()?
            },
            CollectionCardIdentifier::MtgoId(id) => {
                self.http_client.get(self.api_endpoint.join(format!("{}/{}", MTGO_CARD_METHOD, id).as_str())?)
                    .header(ACCEPT, "application/json")
                    .send()?
                    .text()?
            },
            CollectionCardIdentifier::MultiverseId(id) => {
                self.http_client.get(self.api_endpoint.join(format!("{}/{}", MULTIVERSE_CARD_METHOD, id).as_str())?)
                    .header(ACCEPT, "application/json")
                    .send()?
                    .text()?
            },
            CollectionCardIdentifier::OracleId(_) |
            CollectionCardIdentifier::IllustrationId(_) => return Err(Box::new(InvalidCardIdentifierError)),
            CollectionCardIdentifier::Name(name) => {
                self.http_client.get(self.api_endpoint.join(NAMED_CARD_METHOD)?)
                    .query(&[("fuzzy", name)])
                    .header(ACCEPT, "application/json")
                    .send()?
                    .text()?
            },
            CollectionCardIdentifier::NameSet((name, set)) => {
                self.http_client.get(self.api_endpoint.join(NAMED_CARD_METHOD)?)
                    .query(&[("fuzzy", name), ("set", set)])
                    .header(ACCEPT, "application/json")
                    .send()?
                    .text()?
            },
            CollectionCardIdentifier::CollectorNumberSet((collector_number, set)) => {
                self.http_client.get(self.api_endpoint.join(format!("{}/{}/{}", SPECIFIED_CARD_METHOD, set, collector_number).as_str())?)
                    .header(ACCEPT, "application/json")
                    .send()?
                    .text()?
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

        let response = self.http_client.post(self.api_endpoint.join(CARD_COLLECTION_METHOD)?)
            .json(&identifiers_json)
            .header(ACCEPT, "application/json")
            .send()?
            .text()?;

        let api_object = from_str(&response)?;
        if let ApiObject::Error(error) = api_object {
            Err(Box::new(ApiError { error: *error }))
        } else {
            Ok(api_object)
        }
    }

    fn get_bulk_data_endpoint(&mut self) -> Result<ApiObject, Box<dyn ErrorTrait>> {
        self.rate_limit();

        if self.verbose {
            println!("Sending API request for bulk data endpoints");
        }

        let response = self.http_client.get(self.api_endpoint.join(BULK_DATA_METHOD)?)
            .header(ACCEPT, "application/json")
            .send()?
            .text()?;

        let api_object = from_str(&response)?;
        if let ApiObject::Error(error) = api_object {
            Err(Box::new(ApiError { error: *error }))
        } else {
            Ok(api_object)
        }
    }

    pub fn get_bulk_data(&mut self, output_file: PathBuf) -> Result<(), Box<dyn ErrorTrait>> {
        let received_object = self.get_bulk_data_endpoint()?;
        let ApiObject::BulkData(bulk_data_endpoint) = received_object else {
            return Err(Box::new(InvalidApiObjectError { expected: "BulkData",  received: received_object }));
        };

        if self.verbose {
            println!("Sending API request for bulk data");
        }

        let mut output_file = File::create(output_file)?;
        let response = self.http_client.get(bulk_data_endpoint.download_uri)
            .header(ACCEPT, "application/json")
            .timeout(Duration::from_secs(300))
            .send()?
            .bytes()?;

        output_file.write_all(&response)?;

        Ok(())
    }
}