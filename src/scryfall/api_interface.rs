pub mod api_interface {
    use std::{error::Error, fmt::Display, thread::sleep, time::{Duration, Instant}};
    use reqwest::{header::ACCEPT, blocking::Client, Url};
    use serde_json::{from_str, json};
    use url_macro::url;

    use crate::scryfall::{api_classes::api_classes::ApiObject, collection_card_identifier::collection_card_identifier::CollectionCardIdentifier};

    #[derive(Debug, Clone)]
    pub struct InvalidCardIdentifierError;

    impl Display for InvalidCardIdentifierError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Oracle IDs and illustration IDs cannot be used to retrieve specific cards")
        }
    }

    impl Error for InvalidCardIdentifierError {}

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

    pub struct ApiInterface {
        http_client: Client,
        api_endpoint: Url,
        last_request_time: Option<Instant>,
    }

    impl ApiInterface {
        pub fn new() -> Result<ApiInterface, Box<dyn Error>> {
            let builder = Client::builder()
                .user_agent(APP_USER_AGENT);

            Ok(ApiInterface {
                http_client: builder.build()?,
                api_endpoint: url!("https://api.scryfall.com/"),
                last_request_time: None,
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

        pub fn get_card(&mut self, card: &CollectionCardIdentifier) -> Result<ApiObject, Box<dyn Error>> {
            self.rate_limit();

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

            return Ok(from_str(&response)?)
        }

        pub fn get_cards_from_list(&mut self, identifiers: &[CollectionCardIdentifier]) -> Result<ApiObject, Box<dyn Error>> {
            let identifiers_json = json!({
                "identifiers": identifiers
            });

            self.rate_limit();
            println!("Sending API request");

            let response = self.http_client.post(self.api_endpoint.join(CARD_COLLECTION_METHOD)?)
                .json(&identifiers_json)
                .header(ACCEPT, "application/json")
                .send()?
                .text()?;

            //println!("API response received:\n{}", response);

            return Ok(from_str(&response)?)
        }
    }
}