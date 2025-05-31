use std::{collections::HashMap, error::Error, str::FromStr};
use log::{error, Level};
use url::Url;
use wasm_bindgen::{prelude::*, throw_str};
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
    let image_type_small_radio = document
        .get_element_by_id(IMAGE_TYPE_SMALL_RADIO)
        .expect_throw("Could not retrieve include image type small radio element")
        .dyn_into::<HtmlInputElement>()?;

    if image_type_small_radio.checked() {
        return Ok(ImageUriType::Small);
    }

    let image_type_normal_radio = document
        .get_element_by_id(IMAGE_TYPE_NORMAL_RADIO)
        .expect_throw("Could not retrieve include image type normal radio element")
        .dyn_into::<HtmlInputElement>()?;

    if image_type_normal_radio.checked() {
        return Ok(ImageUriType::Normal);
    }

    let image_type_large_radio = document
        .get_element_by_id(IMAGE_TYPE_LARGE_RADIO)
        .expect_throw("Could not retrieve include image type large radio element")
        .dyn_into::<HtmlInputElement>()?;

    if image_type_large_radio.checked() {
        return Ok(ImageUriType::Large);
    }

    let image_type_png_radio = document
        .get_element_by_id(IMAGE_TYPE_PNG_RADIO)
        .expect_throw("Could not retrieve include image type png radio element")
        .dyn_into::<HtmlInputElement>()?;

    if image_type_png_radio.checked() {
        return Ok(ImageUriType::Png);
    }

    let image_type_art_crop_radio = document
        .get_element_by_id(IMAGE_TYPE_ART_CROP_RADIO)
        .expect_throw("Could not retrieve include image type art crop radio element")
        .dyn_into::<HtmlInputElement>()?;

    if image_type_art_crop_radio.checked() {
        return Ok(ImageUriType::ArtCrop);
    }

    let image_type_border_crop_radio = document
        .get_element_by_id(IMAGE_TYPE_BORDER_CROP_RADIO)
        .expect_throw("Could not retrieve include image type border crop radio element")
        .dyn_into::<HtmlInputElement>()?;

    if image_type_border_crop_radio.checked() {
        return Ok(ImageUriType::BorderCrop);
    }

    throw_str("Could not find any checked image type radio button");
}

fn get_selected_options(deck_list: HashMap<CollectionCardIdentifier, usize>, old_deck_list: Option<HashMap<CollectionCardIdentifier, usize>>, custom_card_blob_urls: Vec<String>, document: &Document) -> Result<UserOptions, JsValue> {
    let include_basic_lands_checkbox = document
        .get_element_by_id(INCLUDE_BASIC_LANDS_CHECKBOX_ID)
        .expect_throw("Could not retrieve include basic lands checkbox element")
        .dyn_into::<HtmlInputElement>()?;

    let include_tokens_checkbox = document
        .get_element_by_id(INCLUDE_TOKENS_CHECKBOX_ID)
        .expect_throw("Could not retrieve include tokens checkbox element")
        .dyn_into::<HtmlInputElement>()?;

    let deck_diff_checkbox = document
        .get_element_by_id(DECK_DIFF_CHECKBOX_ID)
        .expect_throw("Could not retrieve deck diff checkbox element")
        .dyn_into::<HtmlInputElement>()?;

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

    let proxies_section = document.get_element_by_id(PROXIES_DIV_ID).expect_throw("Could not retrieve proxies div element").dyn_into::<HtmlDivElement>()?;
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
    let custom_cards: Vec<String> = custom_card_blob_urls.into_iter().map(|value: JsValue| value.as_string().expect_throw("Custom card blob URLs must be strings")).collect();

    let window = window().expect_throw("Could not retrieve global window object");
    let document = window.document().expect_throw("Could not retrieve root document object");
    let deck_list_textbox = document.get_element_by_id(DECK_LIST_TEXTBOX_ID).expect_throw("Could not find deck list textbox");

    let deck_list_text = deck_list_textbox.dyn_into::<HtmlTextAreaElement>()?.value();
    let deck_list = parse_txt_data(&deck_list_text)
        .map_err(rust_error_to_js)?;

    let old_deck_list = if old_deck_list_enabled.is_truthy() {
        let old_deck_list_textbox = document.get_element_by_id(OLD_DECK_LIST_TEXTBOX_ID).expect_throw("Could not find old deck list textbox");

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
    let custom_cards: Vec<String> = custom_card_blob_urls.into_iter().map(|value: JsValue| value.as_string().expect_throw("Custom card blob URLs must be strings")).collect();

    let window = window().expect_throw("Could not retrieve global window object");
    let document = window.document().expect_throw("Could not retrieve root document object");

    let contents = file_contents.as_string().expect_throw("File contents must be a string");
    let file_type = file_mime_type.as_string().expect_throw("File MIME type must be a string");

    let deck_list = match file_type.as_str() {
        "text/plain" => parse_txt_data(&contents).map_err(rust_error_to_js)?,
        "application/json" => parse_json_data(&contents).map_err(rust_error_to_js)?,
        _ => throw_str(&format!("Unsupported MIME type {}", file_type)),
    };

    let old_deck_list = if old_file_contents.is_null() {
        None
    } else {
        let old_contents = old_file_contents.as_string().expect_throw("File contents must be a string");
        let old_file_type = old_file_mime_type.as_string().expect_throw("File MIME type must be a string");

        Some(match old_file_type.as_str() {
            "text/plain" => parse_txt_data(&old_contents).map_err(rust_error_to_js)?,
            "application/json" => parse_json_data(&old_contents).map_err(rust_error_to_js)?,
            _ => throw_str(&format!("Unsupported MIME type {}", old_file_type)),
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