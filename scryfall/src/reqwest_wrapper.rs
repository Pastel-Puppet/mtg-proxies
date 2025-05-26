use std::error::Error;
use reqwest::{blocking::Client, header::ACCEPT};
use serde_json::Value;
use url::Url;

use crate::api_interface::RequestClient;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

pub struct ReqwestWrapper {
    client: Client
}

impl RequestClient for ReqwestWrapper {
    fn build() -> Result<ReqwestWrapper, Box<(dyn Error)>> {
        let builder = Client::builder()
            .user_agent(APP_USER_AGENT);

        Ok(Self {
            client: builder.build()?,
        })
    }

    fn get(&self, url: Url) -> Result<String, Box<(dyn Error)>> {
        let response = self.client.get(url)
            .header(ACCEPT, "application/json")
            .send()?
            .text()?;

        Ok(response)
    }

    fn get_with_parameters(&self, url: Url, query_parameters: &[(&str, &str)]) -> Result<String, Box<(dyn Error)>> {
        let mut request = self.client.get(url);

        if !query_parameters.is_empty() {
            request = request.query(query_parameters);
        }

        let response = request
            .header(ACCEPT, "application/json")
            .send()?
            .text()?;

        Ok(response)
    }

    fn post(&self, url: Url, payload: &Value) -> Result<String, Box<(dyn Error)>> {
        let response = self.client.post(url)
            .json(payload)
            .header(ACCEPT, "application/json")
            .send()?
            .text()?;

        Ok(response)
    }
}