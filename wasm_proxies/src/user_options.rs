use alloc::{string::String, vec::Vec};
use hashbrown::HashMap;
use scryfall::{api_interface::collection_card_identifier::CollectionCardIdentifier, card_images_helper::ImageUriType};
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlInputElement};

const INCLUDE_BASIC_LANDS_CHECKBOX_ID: &str = "include-basic-lands";
const INCLUDE_TOKENS_CHECKBOX_ID: &str = "include-tokens";
const DECK_DIFF_CHECKBOX_ID: &str = "deck-diff";

const IMAGE_TYPE_SMALL_RADIO: &str = "image-type-small-radio";
const IMAGE_TYPE_NORMAL_RADIO: &str = "image-type-normal-radio";
const IMAGE_TYPE_LARGE_RADIO: &str = "image-type-large-radio";
const IMAGE_TYPE_PNG_RADIO: &str = "image-type-png-radio";
const IMAGE_TYPE_BORDER_CROP_RADIO: &str = "image-type-border-crop-radio";

pub struct UserOptions {
    pub exclude_basic_lands: bool,
    pub include_tokens: bool,
    pub image_type: ImageUriType,
    pub extra_cards: Vec<String>,
    pub deck_list: HashMap<CollectionCardIdentifier, usize>,
    pub old_deck: Option<HashMap<CollectionCardIdentifier, usize>>,
}

pub fn get_selected_image_type(document: &Document) -> Result<ImageUriType, JsValue> {
    let image_type_small_radio = match document.get_element_by_id(IMAGE_TYPE_SMALL_RADIO) {
        Some(image_type_small_radio) => image_type_small_radio.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not find include image type small radio element".into()),
    };

    if image_type_small_radio.checked() {
        return Ok(ImageUriType::Small);
    }

    let image_type_normal_radio = match document.get_element_by_id(IMAGE_TYPE_NORMAL_RADIO) {
        Some(image_type_normal_radio) => image_type_normal_radio.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not find include image type normal radio element".into()),
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
        None => return Err("Could not find include image type png radio element".into()),
    };

    if image_type_png_radio.checked() {
        return Ok(ImageUriType::Png);
    }

    let image_type_border_crop_radio = match document.get_element_by_id(IMAGE_TYPE_BORDER_CROP_RADIO) {
        Some(image_type_border_crop_radio) => image_type_border_crop_radio.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not find include image type border crop radio element".into()),
    };

    if image_type_border_crop_radio.checked() {
        return Ok(ImageUriType::BorderCrop);
    }

    Err("Could not find any checked image type radio button".into())
}

pub fn get_selected_options(deck_list: HashMap<CollectionCardIdentifier, usize>, old_deck_list: Option<HashMap<CollectionCardIdentifier, usize>>, custom_card_blob_urls: Vec<String>, document: &Document) -> Result<UserOptions, JsValue> {
    let include_basic_lands_checkbox = match document.get_element_by_id(INCLUDE_BASIC_LANDS_CHECKBOX_ID) {
        Some(include_basic_lands_checkbox) => include_basic_lands_checkbox.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not find include basic lands checkbox element".into()),
    };

    let include_tokens_checkbox = match document.get_element_by_id(INCLUDE_TOKENS_CHECKBOX_ID) {
        Some(include_tokens_checkbox) => include_tokens_checkbox.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not find include tokens checkbox element".into()),
    };

    let deck_diff_checkbox = match document.get_element_by_id(DECK_DIFF_CHECKBOX_ID) {
        Some(deck_diff_checkbox) => deck_diff_checkbox.dyn_into::<HtmlInputElement>()?,
        None => return Err("Could not find deck diff checkbox element".into()),
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
