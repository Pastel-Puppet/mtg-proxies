use std::{collections::HashMap, error::Error};
use log::{error, Level};
use wasm_bindgen::{prelude::*, throw_str};
use web_sys::{window, Document, HtmlDivElement, HtmlImageElement, HtmlTextAreaElement};

use scryfall::{api_interface::ApiInterface, card_images_helper::{extract_images, ImageUriType}, collection_card_identifier::CollectionCardIdentifier, deck_formats::{parse_json_data, parse_txt_data}, fetch_card_list::resolve_cards, reqwest_wrapper::ReqwestWrapper};

const DECK_LIST_TEXTBOX_ID: &str = "deck_list";
const PROXIES_DIV_ID: &str = "proxies";

fn rust_error_to_js(error: Box<dyn Error>) -> JsValue {
    error!("{}", error);
    JsValue::from_str(&error.to_string())
}

async fn add_proxy_images_from_deck_list(mut deck_list: HashMap<CollectionCardIdentifier, usize>, document: &Document) -> Result<(), JsValue> {
    let mut interface = ApiInterface::<ReqwestWrapper>::new()
        .map_err(rust_error_to_js)?;

    let deck_cards = resolve_cards(&mut deck_list, true, &mut interface).await
        .map_err(rust_error_to_js)?;
    let card_images = extract_images(&Vec::from_iter(deck_cards.iter()), false, ImageUriType::Png);

    let proxies_section = document.get_element_by_id(PROXIES_DIV_ID).expect_throw("Could not retrieve proxies div element").dyn_into::<HtmlDivElement>()?;
    proxies_section.set_text_content(None);
    
    for (card_image, count) in card_images {
        for _ in 0..count {
            let image_node = document.create_element("img")?.dyn_into::<HtmlImageElement>()?;
            image_node.set_src(card_image.as_str());
            image_node.set_class_name("card");
            proxies_section.append_child(&image_node)?;
        }
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn generate_proxies_from_textbox() -> Result<(), JsValue> {
    let window = window().expect_throw("Could not retrieve global window object");
    let document = window.document().expect_throw("Could not retrieve root document object");
    let deck_list_textbox = document.get_element_by_id(DECK_LIST_TEXTBOX_ID).expect_throw("Could not find deck list textbox");

    let deck_list_text = deck_list_textbox.dyn_into::<HtmlTextAreaElement>()?.value();
    let deck_list = parse_txt_data(&deck_list_text)
        .map_err(rust_error_to_js)?;

    add_proxy_images_from_deck_list(deck_list, &document).await
}

#[wasm_bindgen]
pub async fn generate_proxies_from_file_contents(file_contents: JsValue, file_mime_type: JsValue) -> Result<(), JsValue> {
    let window = window().expect_throw("Could not retrieve global window object");
    let document = window.document().expect_throw("Could not retrieve root document object");

    let contents = file_contents.as_string().expect_throw("File contents must be a string");
    let file_type = file_mime_type.as_string().expect_throw("File MIME type must be a string");

    let deck_list = match file_type.as_str() {
        "text/plain" => parse_txt_data(&contents).map_err(rust_error_to_js)?,
        "application/json" => parse_json_data(&contents).map_err(rust_error_to_js)?,
        _ => throw_str(&format!("Unsupported MIME type {}", file_type)),
    };

    add_proxy_images_from_deck_list(deck_list, &document).await
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn initialise() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug)
        .map_err(|error| rust_error_to_js(Box::new(error)))?;

    Ok(())
}