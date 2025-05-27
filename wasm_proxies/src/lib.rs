use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::{js_sys::Promise, window};

use scryfall::{api_interface::ApiInterface, collection_card_identifier::CollectionCardIdentifier, reqwest_wrapper::ReqwestWrapper};

async fn test() -> Result<JsValue, JsValue> {
    let mut interface = ApiInterface::<ReqwestWrapper>::new(false)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    let card = interface.get_card(&CollectionCardIdentifier::Name("Birds of Paradise".to_string())).await
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let val = document.create_element("p")?;
    val.set_inner_html(&format!("{:?}", card));

    body.append_child(&val)?;

    Ok(JsValue::null())
}

#[wasm_bindgen]
pub fn test_get() -> Promise {
    future_to_promise(test())
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    Ok(())
}