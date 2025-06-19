use core::{error::Error, fmt::Display};
use alloc::{string::{String, ToString}, boxed::Box, borrow::ToOwned};
use serde_json::Value;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console::error_1, js_sys::JsString, window, Request, RequestInit, RequestMode, Response, Window};

use crate::api_interface::RequestClient;

#[derive(Debug, Clone)]
pub struct JsErrorWrapper {
    error: JsValue,
}

impl Display for JsErrorWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        error_1(&self.error);
        write!(f, "Error in Javascript while performing HTTP request, check console for details")
    }
}

impl Error for JsErrorWrapper {}

impl From<JsValue> for JsErrorWrapper {
    fn from(error: JsValue) -> Self {
        Self {
            error
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoWindowError {}

impl Display for NoWindowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Could not access global window")
    }
}

impl Error for NoWindowError {}

#[derive(Debug, Clone)]
pub struct NotStringError {
    not_string_value: JsValue,
}

impl Display for NotStringError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        error_1(&self.not_string_value);
        write!(f, "Javascript value must be a string")
    }
}

impl Error for NotStringError {}

pub struct WasmFetchWrapper {
    window: Window,
}

impl WasmFetchWrapper {
    async fn _get(&self, url: Url, opts: RequestInit) -> Result<JsValue, JsValue> {
        let request = Request::new_with_str_and_init(&url.as_str(), &opts)?;
        request.headers().set("Accept", "application/json")?;

        let response: Response = JsFuture::from(self.window.fetch_with_request(&request)).await?.dyn_into()?;
        let json = JsFuture::from(response.text()?).await?;

        Ok(json)
    }

    async fn _post(&self, url: Url, opts: RequestInit) -> Result<JsValue, JsValue> {
        let request = Request::new_with_str_and_init(&url.as_str(), &opts)?;
        request.headers().set("Accept", "application/json")?;
        request.headers().set("Content-Type", "application/json")?;

        let response: Response = JsFuture::from(self.window.fetch_with_request(&request)).await?.dyn_into()?;
        let json = JsFuture::from(response.text()?).await?;

        Ok(json)
    }
}

impl RequestClient for WasmFetchWrapper {
    fn build() -> Result<WasmFetchWrapper, Box<(dyn Error)>> {
        let Some(window) = window() else {
            return Err(NoWindowError {}.into());
        };

        Ok(Self {
            window
        })
    }

    async fn get(&self, url: Url) -> Result<String, Box<(dyn Error)>> {
        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let json_value = match self._get(url, opts).await {
            Ok(json) => json.dyn_into::<JsString>().map_err(|not_string_value: JsValue| NotStringError { not_string_value })?,
            Err(js_error) => return Err(Box::<JsErrorWrapper>::new(js_error.into())),
        };

        let Some(json) = json_value.as_string() else {
            return Err(NotStringError { not_string_value: json_value.into() }.into());
        };

        Ok(json.as_str().to_owned())
    }

    async fn get_with_parameters(&self, mut url: Url, query_parameters: &[(&str, &str)]) -> Result<String, Box<(dyn Error)>> {
        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let mut query_pairs = url.query_pairs_mut();
        for (parameter_name, parameter_value) in query_parameters {
            query_pairs.append_pair(parameter_name, parameter_value);
        }
        query_pairs.finish();
        drop(query_pairs);

        let json_value = match self._get(url, opts).await {
            Ok(json) => json.dyn_into::<JsString>().map_err(|not_string_value: JsValue| NotStringError { not_string_value })?,
            Err(js_error) => return Err(Box::<JsErrorWrapper>::new(js_error.into())),
        };

        let Some(json) = json_value.as_string() else {
            return Err(NotStringError { not_string_value: json_value.into() }.into());
        };

        Ok(json.as_str().to_owned())
    }

    async fn post(&self, url: Url, payload: &Value) -> Result<String, Box<(dyn Error)>> {
        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_mode(RequestMode::Cors);
        opts.set_body(&JsValue::from(payload.to_string()));

        let json_value = match self._post(url, opts).await {
            Ok(json) => json.dyn_into::<JsString>().map_err(|not_string_value: JsValue| NotStringError { not_string_value })?,
            Err(js_error) => return Err(Box::<JsErrorWrapper>::new(js_error.into())),
        };

        let Some(json) = json_value.as_string() else {
            return Err(NotStringError { not_string_value: json_value.into() }.into());
        };

        Ok(json.as_str().to_owned())
    }
}