use std::{collections::HashMap, error::Error, str::FromStr};
use log::{error, Level};
use url::Url;
use wasm_bindgen::prelude::*;
use web_sys::{js_sys::Array, window, Document, HtmlDivElement, HtmlImageElement, HtmlInputElement, HtmlTextAreaElement};

use scryfall::{api_interface::ApiInterface, card_images_helper::{extract_images, ImageUriType}, collection_card_identifier::CollectionCardIdentifier, deck_formats::{deck_diff, parse_json_data, parse_txt_data}, fetch_card_list::resolve_cards, reqwest_wrapper::ReqwestWrapper};

const DECK_LIST_TEXTBOX_ID: &str = "deck-list";
const OLD_DECK_LIST_TEXTBOX_ID: &str = "old-deck-list";
const PROXIES_DIV_ID: &str = "proxies";
const INCLUDE_BASIC_LANDS_CHECKBOX_ID: &str = "include-basic-lands";
const INCLUDE_TOKENS_CHECKBOX_ID: &str = "include-tokens";
const DECK_DIFF_CHECKBOX_ID: &str = "deck-diff";

const IMAGE_TYPE_SMALL_RADIO: &str = "image-type-small-radio";
const IMAGE_TYPE_NORMAL_RADIO: &str = "image-type-normal-radio";
const IMAGE_TYPE_LARGE_RADIO: &str = "image-type-large-radio";
const IMAGE_TYPE_PNG_RADIO: &str = "image-type-png-radio";
const IMAGE_TYPE_ART_CROP_RADIO: &str = "image-type-art-crop-radio";
const IMAGE_TYPE_BORDER_CROP_RADIO: &str = "image-type-border-crop-radio";

struct UserOptions {
    exclude_basic_lands: bool,
    include_tokens: bool,
    image_type: ImageUriType,
    extra_cards: Vec<String>,
    deck_list: HashMap<CollectionCardIdentifier, usize>,
    old_deck: Option<HashMap<CollectionCardIdentifier, usize>>,
}

fn rust_error_to_js<T>(error: T) -> JsValue
    where T: Into<Box<dyn Error>> {
    let boxed_error = error.into();
    error!("{}", boxed_error);
    JsValue::from_str(&boxed_error.to_string())
}

fn get_selected_image_type(document: &Document) -> Result<ImageUriType, JsValue> {
    let image_type_small_radio = match document.get_element_by_id(IMAGE_TYPE_SMALL_RADIO) {
        Some(image_type_small_radio) => image_type_small_radio.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not retrieve include image type small radio element".into()),
    };

    if image_type_small_radio.checked() {
        return Ok(ImageUriType::Small);
    }

    let image_type_normal_radio = match document.get_element_by_id(IMAGE_TYPE_NORMAL_RADIO) {
        Some(image_type_normal_radio) => image_type_normal_radio.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not retrieve include image type normal radio element".into()),
    };

    if image_type_normal_radio.checked() {
        return Ok(ImageUriType::Normal);
    }

    let image_type_large_radio = match document.get_element_by_id(IMAGE_TYPE_LARGE_RADIO) {
        Some(image_type_large_radio) => image_type_large_radio.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not retrieve include image type large radio element".into()),
    };

    if image_type_large_radio.checked() {
        return Ok(ImageUriType::Large);
    }

    let image_type_png_radio = match document.get_element_by_id(IMAGE_TYPE_PNG_RADIO) {
        Some(image_type_png_radio) => image_type_png_radio.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not retrieve include image type png radio element".into()),
    };

    if image_type_png_radio.checked() {
        return Ok(ImageUriType::Png);
    }

    let image_type_art_crop_radio = match document.get_element_by_id(IMAGE_TYPE_ART_CROP_RADIO) {
        Some(image_type_art_crop_radio) => image_type_art_crop_radio.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not retrieve include image type art crop radio element".into()),
    };

    if image_type_art_crop_radio.checked() {
        return Ok(ImageUriType::ArtCrop);
    }

    let image_type_border_crop_radio = match document.get_element_by_id(IMAGE_TYPE_BORDER_CROP_RADIO) {
        Some(image_type_border_crop_radio) => image_type_border_crop_radio.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not retrieve include image type border crop radio element".into()),
    };

    if image_type_border_crop_radio.checked() {
        return Ok(ImageUriType::BorderCrop);
    }

    Err("Could not find any checked image type radio button".into())
}

fn get_selected_options(deck_list: HashMap<CollectionCardIdentifier, usize>, old_deck_list: Option<HashMap<CollectionCardIdentifier, usize>>, custom_card_blob_urls: Vec<String>, document: &Document) -> Result<UserOptions, JsValue> {
    let include_basic_lands_checkbox = match document.get_element_by_id(INCLUDE_BASIC_LANDS_CHECKBOX_ID) {
        Some(include_basic_lands_checkbox) => include_basic_lands_checkbox.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not retrieve include basic lands checkbox element".into()),
    };

    let include_tokens_checkbox = match document.get_element_by_id(INCLUDE_TOKENS_CHECKBOX_ID) {
        Some(include_tokens_checkbox) => include_tokens_checkbox.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not retrieve include tokens checkbox element".into()),
    };

    let deck_diff_checkbox = match document.get_element_by_id(DECK_DIFF_CHECKBOX_ID) {
        Some(deck_diff_checkbox) => deck_diff_checkbox.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not retrieve deck diff checkbox element".into()),
    };

    let old_deck = if deck_diff_checkbox.checked() {
        old_deck_list
    } else {
        None
    };

    Ok(UserOptions {
        exclude_basic_lands: !include_basic_lands_checkbox.checked(),
        include_tokens: include_tokens_checkbox.checked(),
        image_type: get_selected_image_type(document)?,
        extra_cards: custom_card_blob_urls,
        deck_list,
        old_deck,
    })
}

async fn add_proxy_images_from_deck_list(mut user_options: UserOptions, document: &Document) -> Result<(), JsValue> {
    let mut interface = ApiInterface::<ReqwestWrapper>::new()
        .map_err(rust_error_to_js)?;

    let deck_cards = resolve_cards(&mut user_options.deck_list, user_options.include_tokens, &mut interface).await
        .map_err(rust_error_to_js)?;

    let mut cards_to_display = if let Some(mut old_deck) = user_options.old_deck {
        let old_deck_cards = resolve_cards(&mut old_deck, user_options.include_tokens, &mut interface).await
            .map_err(rust_error_to_js)?;

        deck_diff(old_deck_cards, deck_cards).added
    } else {
        deck_cards.into_iter().map(|card| card.card).collect()
    };

    cards_to_display.sort();

    let mut card_images = extract_images(&cards_to_display, user_options.exclude_basic_lands, user_options.image_type);

    let proxies_section = match document.get_element_by_id(PROXIES_DIV_ID) {
        Some(proxies_section) => proxies_section.dyn_into::<HtmlDivElement>()?,
        None => return Err("Could not retrieve proxies div element".into()),
    };
    proxies_section.set_text_content(None);

    for extra_card in user_options.extra_cards {
        card_images.push(Url::from_str(&extra_card).map_err(rust_error_to_js)?);
    }
    
    for card_image in card_images {
        let image_node = document.create_element("img")?.dyn_into::<HtmlImageElement>()?;
        image_node.set_src(card_image.as_str());
        image_node.set_class_name("card");
        proxies_section.append_child(&image_node)?;
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn generate_proxies_from_textbox(custom_card_blob_urls: Array, old_deck_list_enabled: JsValue) -> Result<(), JsValue> {
    let mut custom_cards: Vec<String> = Vec::new();
    
    for card in custom_card_blob_urls.into_iter() {
        match card.as_string() {
            Some(card) => custom_cards.push(card),
            None => return Err("Custom card blob URLs must be strings".into()),
        };
    }

    let Some(window) = window() else {
        return Err("Could not retrieve global window object".into());
    };
    let Some(document) = window.document() else {
        return Err("Could not retrieve root document object".into());
    };
    let Some(deck_list_textbox) = document.get_element_by_id(DECK_LIST_TEXTBOX_ID) else {
        return Err("Could not find deck list textbox".into());
    };

    let deck_list_text = deck_list_textbox.dyn_into::<HtmlTextAreaElement>()?.value();
    let deck_list = parse_txt_data(&deck_list_text)
        .map_err(rust_error_to_js)?;

    let old_deck_list = if old_deck_list_enabled.is_truthy() {
        let Some(old_deck_list_textbox) = document.get_element_by_id(OLD_DECK_LIST_TEXTBOX_ID) else {
            return Err("Could not find old deck list textbox".into());
        };

        let old_deck_list_text = old_deck_list_textbox.dyn_into::<HtmlTextAreaElement>()?.value();
        Some(parse_txt_data(&old_deck_list_text)
            .map_err(rust_error_to_js)?)
    } else {
        None
    };

    add_proxy_images_from_deck_list(get_selected_options(deck_list, old_deck_list, custom_cards, &document)?, &document).await
}

#[wasm_bindgen]
pub async fn generate_proxies_from_file_contents(file_contents: JsValue, file_mime_type: JsValue, old_file_contents: JsValue, old_file_mime_type: JsValue, custom_card_blob_urls: Array) -> Result<(), JsValue> {
    let mut custom_cards: Vec<String> = Vec::new();
    
    for card in custom_card_blob_urls.into_iter() {
        match card.as_string() {
            Some(card) => custom_cards.push(card),
            None => return Err("Custom card blob URLs must be strings".into()),
        };
    }

    let Some(window) = window() else {
        return Err("Could not retrieve global window object".into());
    };
    let Some(document) = window.document() else {
        return Err("Could not retrieve root document object".into());
    };

    let Some(contents) = file_contents.as_string() else {
        return Err("File contents must be a string".into());
    };
    let Some(file_type) = file_mime_type.as_string() else {
        return Err("File MIME type must be a string".into());
    };

    let deck_list = match file_type.as_str() {
        "text/plain" => parse_txt_data(&contents).map_err(rust_error_to_js)?,
        "application/json" => parse_json_data(&contents).map_err(rust_error_to_js)?,
        _ => return Err(format!("Unsupported MIME type {}", file_type).into()),
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
            "text/plain" => parse_txt_data(&old_contents).map_err(rust_error_to_js)?,
            "application/json" => parse_json_data(&old_contents).map_err(rust_error_to_js)?,
            _ => return Err(format!("Unsupported MIME type {}", old_file_type).into()),
        })
    };

    add_proxy_images_from_deck_list(get_selected_options(deck_list, old_deck_list, custom_cards, &document)?, &document).await
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn initialise() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug)
        .map_err(rust_error_to_js)?;

    Ok(())
}