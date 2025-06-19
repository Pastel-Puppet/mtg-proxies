use std::error::Error;
use reqwest::{Client, header::ACCEPT};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use serde_json::Value;

use crate::api_interface::RequestClient;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

pub struct ReqwestWrapper {
    client: Client,
    rate_limiter: DefaultDirectRateLimiter,
}

impl RequestClient for ReqwestWrapper {
    fn build() -> Result<ReqwestWrapper, Box<(dyn Error)>> {
        let mut builder = Client::builder();
        builder = builder.user_agent(APP_USER_AGENT);

        Ok(Self {
            client: builder.build()?,
            rate_limiter: RateLimiter::direct(Quota::per_second(nonzero!(10_u32))),
        })
    }

    async fn get(&self, url: String) -> Result<String, Box<(dyn Error)>> {
        self.rate_limiter.until_ready().await;

        let response = self.client.get(url)
            .header(ACCEPT, "application/json")
            .send().await?
            .text().await?;

        Ok(response)
    }

    async fn get_with_parameters(&self, url: String, query_parameters: &[(&str, &str)]) -> Result<String, Box<(dyn Error)>> {
        self.rate_limiter.until_ready().await;

        let mut request = self.client.get(url);

        if !query_parameters.is_empty() {
            request = request.query(query_parameters);
        }

        let response = request
            .header(ACCEPT, "application/json")
            .send().await?
            .text().await?;

        Ok(response)
    }

    async fn post(&self, url: String, payload: &Value) -> Result<String, Box<(dyn Error)>> {
        self.rate_limiter.until_ready().await;

        let response = self.client.post(url)
            .json(payload)
            .header(ACCEPT, "application/json")
            .send().await?
            .text().await?;

        Ok(response)
    }
}