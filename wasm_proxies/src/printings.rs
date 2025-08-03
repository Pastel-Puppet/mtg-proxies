use alloc::{string::{String, ToString}, vec::Vec};
use scryfall::{api_interface::{api_classes::Card, wasm_fetch_wrapper::WasmFetchWrapper, ApiInterface}, card_images_helper::extract_images};
use wasm_bindgen::prelude::*;
use web_sys::{js_sys::{Array, JsString}, window};

use crate::{rust_error_to_js, user_options::get_selected_image_type};

#[wasm_bindgen]
pub struct Printings {
    #[wasm_bindgen(getter_with_clone)]
    pub printings: Array,
    pub current_index: usize,
}

#[wasm_bindgen]
pub struct Printing {
    #[wasm_bindgen(getter_with_clone)]
    pub faces: Array,
    #[wasm_bindgen(getter_with_clone)]
    pub set: String,
    #[wasm_bindgen(getter_with_clone)]
    pub collector_number: String,
    #[wasm_bindgen(getter_with_clone)]
    pub scryfall_url: String,
}

fn printings_vec_to_array(card_printing_images: Vec<(Card, Vec<String>)>) -> Array {
    Array::from_iter(
        card_printing_images.into_iter().map(
            |(card, printing_images)| JsValue::from(Printing {
                faces: Array::from_iter(
                    printing_images.into_iter().map(JsString::from)
                ),
                set: card.set_name,
                collector_number: card.collector_number,
                scryfall_url: card.scryfall_uri.to_string(),
            })
        )
    )
}

fn get_printings_and_current_index(card_printing_images: Vec<(Card, Vec<String>)>, current_printing_image: String) -> Result<Printings, JsValue> {
    for (index, (_, printing_images)) in card_printing_images.iter().enumerate() {
        for printing_image in printing_images {
            if *printing_image == current_printing_image {
                return Ok(Printings {
                    printings: printings_vec_to_array(card_printing_images),
                    current_index: index,
                });
            }
        }
    }

    Err("Could not find current printing in list of card printings".into())
}

#[wasm_bindgen]
pub async fn get_printings_for_card(search_url: String, current_printing_image: String, card_name: String) -> Result<Printings, JsValue> {
    let Some(window) = window() else {
        return Err("Could not find global window object".into());
    };
    let Some(document) = window.document() else {
        return Err("Could not find root document object".into());
    };
    
    let interface = ApiInterface::<WasmFetchWrapper>::new()
        .map_err(rust_error_to_js)?;

    let card_printings = interface.get_all_printings(search_url, card_name).await
        .map_err(rust_error_to_js)?;

    let card_printing_images = extract_images(card_printings, false, get_selected_image_type(&document)?);
    get_printings_and_current_index(card_printing_images, current_printing_image)
}
