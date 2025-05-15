pub mod scryfall_interface {
    use std::error::Error;

    use reqwest::{header::ACCEPT, Client, Url};
    use url_macro::url;

    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
    );

    pub struct CardDataInterface {
        http_client: Client,
        api_endpoint: Url,
        cards_method: &'static str,
    }

    impl CardDataInterface {
        pub fn new() -> Result<CardDataInterface, Box<dyn Error>> {
            let builder = Client::builder()
                .user_agent(APP_USER_AGENT);

            Ok(CardDataInterface {
                http_client: builder.build()?,
                api_endpoint: url!("https://api.scryfall.com/"),
                cards_method: "/cards/named",
            })
        }

        pub async fn get_named_card(&self, card_name: &str) -> Result<String, Box<dyn Error>> {
            let response = self.http_client.get(self.api_endpoint.join(self.cards_method)?)
                .query(&[("fuzzy", card_name)])
                .header(ACCEPT, "application/json")
                .send()
                .await?
                .text()
                .await?;

            return Ok(response)
        }
    }
}