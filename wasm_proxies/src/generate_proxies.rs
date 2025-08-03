use alloc::{borrow::ToOwned, format, string::{String, ToString}, vec::Vec};
use scryfall::{api_interface::{wasm_fetch_wrapper::WasmFetchWrapper, ApiInterface}, card_images_helper::extract_images, deck_diff::deck_diff, deck_parsers::{parse_json_data, parse_txt_data_js}};
use wasm_bindgen::prelude::*;
use web_sys::{js_sys::{Array, Function, JsString}, window, Document, HtmlDivElement, HtmlImageElement, HtmlTextAreaElement};

use scryfall::fetch_card_data::fetch_deck::FetchDeck;
use crate::{rust_error_to_js, user_options::{get_selected_options, UserOptions}};

const DECK_LIST_TEXTBOX_ID: &str = "deck-list";
const OLD_DECK_LIST_TEXTBOX_ID: &str = "old-deck-list";
const PROXIES_DIV_ID: &str = "proxies";

#[wasm_bindgen]
pub struct CardClickedData {
    #[wasm_bindgen(getter_with_clone)]
    pub card_face_images_array: Array,
    #[wasm_bindgen(getter_with_clone)]
    pub prints_search_url: String,
    #[wasm_bindgen(getter_with_clone)]
    pub card_name: String,
    pub is_custom_card: bool,
}

async fn add_proxy_images_from_deck_list(user_options: UserOptions, document: &Document, card_click_callback: Function) -> Result<(), JsValue> {
    let interface = ApiInterface::<WasmFetchWrapper>::new()
        .map_err(rust_error_to_js)?;

    let deck_cards = interface.fetch_deck(&user_options.deck_list, user_options.include_tokens).await
        .map_err(rust_error_to_js)?;

    let mut cards_to_display = if let Some(old_deck) = user_options.old_deck {
        let old_deck_cards = interface.fetch_deck(&old_deck, user_options.include_tokens).await
            .map_err(rust_error_to_js)?;

        deck_diff(old_deck_cards, deck_cards).added
    } else {
        deck_cards.into_iter().flat_map(|card| {
            let mut cards = Vec::new();
            for _ in 0..card.count {
                cards.push(card.card.clone());
            }
            cards
        }).collect()
    };

    cards_to_display.sort();

    let card_images = extract_images(cards_to_display, user_options.exclude_basic_lands, user_options.image_type);

    let proxies_section = match document.get_element_by_id(PROXIES_DIV_ID) {
        Some(proxies_section) => proxies_section.dyn_into::<HtmlDivElement>()?,
        None => return Err("Could not find proxies div element".into()),
    };
    proxies_section.set_text_content(None);

    for extra_card in user_options.extra_cards {
        let image_node = document.create_element("img")?.dyn_into::<HtmlImageElement>()?;
        image_node.set_src(&extra_card);
        image_node.set_class_name("card-face");
        image_node.set_tab_index(0);
        image_node.set_alt("Custom card");
        image_node.set_attribute("loading", "lazy")?;

        let card_face_images_array = Array::of1(&JsString::from(extra_card));
        image_node.set_onclick(Some(&card_click_callback.bind1(&image_node, &JsValue::from(CardClickedData {
            card_face_images_array,
            prints_search_url: "".to_owned(),
            card_name: "".to_owned(),
            is_custom_card: true,
        }))));
        proxies_section.append_child(&image_node)?;
    }
    
    for (card, card_face_images) in card_images {
        for card_image in &card_face_images {
            let image_node = document.create_element("img")?.dyn_into::<HtmlImageElement>()?;
            image_node.set_src(card_image);
            image_node.set_class_name("card-face");
            image_node.set_tab_index(0);
            image_node.set_alt(&card.name);
            image_node.set_attribute("loading", "lazy")?;

            let card_face_images_array = Array::from_iter(card_face_images.iter().cloned().map(JsString::from));
            image_node.set_onclick(Some(&card_click_callback.bind1(&image_node, &JsValue::from(CardClickedData {
                card_face_images_array,
                prints_search_url: card.prints_search_uri.as_str().to_string(),
                card_name: card.name.clone(),
                is_custom_card: false,
            }))));
            
            proxies_section.append_child(&image_node)?;
        }
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn generate_proxies_from_textbox(custom_card_blob_urls: Array, old_deck_list_enabled: JsValue, card_click_callback: Function) -> Result<(), JsValue> {
    let mut custom_cards: Vec<String> = Vec::new();
    
    for card in custom_card_blob_urls.into_iter() {
        match card.as_string() {
            Some(card) => custom_cards.push(card),
            None => return Err("Custom card blob URLs must be strings".into()),
        };
    }

    let Some(window) = window() else {
        return Err("Could not find global window object".into());
    };
    let Some(document) = window.document() else {
        return Err("Could not find root document object".into());
    };
    let Some(deck_list_textbox) = document.get_element_by_id(DECK_LIST_TEXTBOX_ID) else {
        return Err("Could not find deck list textbox".into());
    };

    let deck_list_text = deck_list_textbox.dyn_into::<HtmlTextAreaElement>()?.value();
    let deck_list = parse_txt_data_js(&deck_list_text)?;

    let old_deck_list = if old_deck_list_enabled.is_truthy() {
        let Some(old_deck_list_textbox) = document.get_element_by_id(OLD_DECK_LIST_TEXTBOX_ID) else {
            return Err("Could not find old deck list textbox".into());
        };

        let old_deck_list_text = old_deck_list_textbox.dyn_into::<HtmlTextAreaElement>()?.value();
        Some(parse_txt_data_js(&old_deck_list_text)?)
    } else {
        None
    };

    add_proxy_images_from_deck_list(get_selected_options(deck_list, old_deck_list, custom_cards, &document)?, &document, card_click_callback).await
}

#[wasm_bindgen]
pub async fn generate_proxies_from_file_contents(file_contents: JsValue, file_mime_type: JsValue, old_file_contents: JsValue, old_file_mime_type: JsValue, custom_card_blob_urls: Array, card_click_callback: Function) -> Result<(), JsValue> {
    let mut custom_cards: Vec<String> = Vec::new();
    
    for card in custom_card_blob_urls.into_iter() {
        match card.as_string() {
            Some(card) => custom_cards.push(card),
            None => return Err("Custom card blob URLs must be strings".into()),
        };
    }

    let Some(window) = window() else {
        return Err("Could not find global window object".into());
    };
    let Some(document) = window.document() else {
        return Err("Could not find root document object".into());
    };

    let Some(contents) = file_contents.as_string() else {
        return Err("File contents must be a string".into());
    };
    let Some(file_type) = file_mime_type.as_string() else {
        return Err("File MIME type must be a string".into());
    };

    let deck_list = match file_type.as_str() {
        "text/plain" | "" => parse_txt_data_js(&contents)?,
        "application/json" => parse_json_data(&contents).map_err(rust_error_to_js)?,
        _ => return Err(format!("Unsupported MIME type {file_type}").into()),
    };

    let old_deck_list = if old_file_contents.is_null() {
        None
    } else {
        let Some(old_contents) = old_file_contents.as_string() else {
            return Err("File contents must be a string".into());
        };
        let Some(old_file_type) = old_file_mime_type.as_string() else {
            return Err("File MIME type must be a string".into());
        };

        Some(match old_file_type.as_str() {
            "text/plain" | "" => parse_txt_data_js(&old_contents)?,
            "application/json" => parse_json_data(&old_contents).map_err(rust_error_to_js)?,
            _ => return Err(format!("Unsupported MIME type {old_file_type}").into()),
        })
    };

    add_proxy_images_from_deck_list(get_selected_options(deck_list, old_deck_list, custom_cards, &document)?, &document, card_click_callback).await
}