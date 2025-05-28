use std::{collections::HashMap, error::Error};
use log::{error, warn, Level};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{js_sys::JsString, window, Document, Event, FileReader, HtmlDivElement, HtmlElement, HtmlImageElement, HtmlInputElement, HtmlTextAreaElement};

use scryfall::{api_interface::ApiInterface, card_images_helper::{extract_images, ImageUriType}, collection_card_identifier::CollectionCardIdentifier, deck_formats::{parse_json_data, parse_txt_data}, fetch_card_list::resolve_cards, reqwest_wrapper::ReqwestWrapper};

const OPTIONS_DIV_ID: &str = "options";
const DECK_LIST_TEXTBOX_ID: &str = "deck_list";
const PROXIES_DIV_ID: &str = "proxies";
const PROXIES_TXT_BUTTON_ID: &str = "proxies_txt_button";
const PROXIES_FILE_SELECT_ID: &str = "proxies_file_select";
const PROXIES_FILE_BUTTON_ID: &str = "proxies_file_button";

fn rust_error_to_js(error: Box<dyn Error>) -> JsValue {
    error!("{}", error);
    JsValue::from_str(&error.to_string())
}

async fn add_proxy_images_from_deck_list(mut deck_list: HashMap<CollectionCardIdentifier, usize>, document: &Document, body: &HtmlElement) -> Result<(), JsValue> {
    let mut interface = ApiInterface::<ReqwestWrapper>::new()
        .map_err(rust_error_to_js)?;

    let deck_cards = resolve_cards(&mut deck_list, true, &mut interface).await
        .map_err(rust_error_to_js)?;
    let card_images = extract_images(&Vec::from_iter(deck_cards.iter()), false, ImageUriType::Png);

    let deck_list_textbox = document.create_element("div")?.dyn_into::<HtmlDivElement>()?;
    deck_list_textbox.set_id(PROXIES_DIV_ID);
    deck_list_textbox.set_class_name(PROXIES_DIV_ID);
    body.append_child(&deck_list_textbox)?;
    
    for (card_image, count) in card_images {
        for _ in 0..count {
            let image_node = document.create_element("img")?.dyn_into::<HtmlImageElement>()?;
            image_node.set_src(card_image.as_str());
            image_node.set_class_name("card");
            deck_list_textbox.append_child(&image_node)?;
        }
    }

    Ok(())
}

async fn generate_proxies_from_textbox() -> Result<(), JsValue> {
    let window = window().expect_throw("Could not retrieve global window object");
    let document = window.document().expect_throw("Could not retrieve root document object");
    let body = document.body().expect_throw("Could not retrieve document body element");
    let deck_list_textbox = document.get_element_by_id(DECK_LIST_TEXTBOX_ID).expect_throw("Could not find deck list textbox");

    let deck_list_text = deck_list_textbox.dyn_into::<HtmlTextAreaElement>()?.value();
    let deck_list = parse_txt_data(&deck_list_text)
        .map_err(rust_error_to_js)?;

    add_proxy_images_from_deck_list(deck_list, &document, &body).await
}

fn generate_proxies_from_textbox_callback(_: Event) {
    spawn_local(async {
        if let Err(error) = generate_proxies_from_textbox().await {
            error!("{:?}", error);
        }
    });
}

async fn generate_proxies_from_file_contents(file_reader: &FileReader) -> Result<(), JsValue> {
    let window = window().expect_throw("Could not retrieve global window object");
    let document = window.document().expect_throw("Could not retrieve root document object");
    let body = document.body().expect_throw("Could not retrieve document body element");

    let contents = file_reader.result()?.dyn_into::<JsString>()?.as_string().expect_throw("File contents could not be cast to a string");
    let deck_list = parse_json_data(&contents)
        .map_err(rust_error_to_js)?;

    add_proxy_images_from_deck_list(deck_list, &document, &body).await
}

fn generate_proxies_from_file() -> Result<(), JsValue> {
    let window = window().expect_throw("Could not retrieve global window object");
    let document = window.document().expect_throw("Could not retrieve root document object");
    let deck_file_button = document.get_element_by_id(PROXIES_FILE_SELECT_ID).expect_throw("Could not find deck file select");

    let Some(deck_file_list) = deck_file_button.dyn_into::<HtmlInputElement>()?.files() else {
        warn!("No file list could be found");
        return Ok(())
    };
    let Some(deck_file) = deck_file_list.item(0) else {
        warn!("File list is empty");
        return Ok(())
    };

    let file_reader = FileReader::new()?;

    let handle_file_contents_closure = Closure::wrap(Box::new(|event: Event| {
        let Some(event_target) = event.target() else {
            error!("Event target is not accessible");
            return;
        };

        let file_reader = match event_target.dyn_into::<FileReader>()  {
            Ok(file_reader) => file_reader,
            Err(error) => {
                error!("{:?}", error);
                return;
            },
        };

        spawn_local(async move {
            if let Err(error) = generate_proxies_from_file_contents(&file_reader).await {
                error!("{:?}", error);
            }
        });
    }) as Box<dyn FnMut(_)>);

    file_reader.set_onload(Some(handle_file_contents_closure.as_ref().unchecked_ref()));
    handle_file_contents_closure.forget();

    file_reader.read_as_text(&deck_file)?;

    Ok(())
}

fn generate_proxies_from_file_callback(_: Event) {
    if let Err(error) = generate_proxies_from_file() {
        error!("{:?}", error);
    }
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn initialise() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug)
        .map_err(|error| rust_error_to_js(Box::new(error)))?;

    let window = window().expect_throw("Could not retrieve global window object");
    let document = window.document().expect_throw("Could not retrieve root document object");
    let body = document.body().expect_throw("Could not retrieve document body element");

    let options = document.create_element("div")?.dyn_into::<HtmlDivElement>()?;
    options.set_id(OPTIONS_DIV_ID);
    options.set_class_name(OPTIONS_DIV_ID);
    body.append_child(&options)?;

    let deck_list_textbox = document.create_element("textarea")?.dyn_into::<HtmlTextAreaElement>()?;
    deck_list_textbox.set_id(DECK_LIST_TEXTBOX_ID);
    deck_list_textbox.set_placeholder("Enter deck list here");
    options.append_child(&deck_list_textbox)?;

    let proxies_txt_button = document.create_element("input")?.dyn_into::<HtmlInputElement>()?;
    proxies_txt_button.set_id(PROXIES_TXT_BUTTON_ID);
    proxies_txt_button.set_value("Generate proxies");
    proxies_txt_button.set_type("button");

    let generate_proxies_text_closure = Closure::wrap(Box::new(generate_proxies_from_textbox_callback) as Box<dyn FnMut(_)>);
    proxies_txt_button.set_onclick(Some(generate_proxies_text_closure.as_ref().unchecked_ref()));
    generate_proxies_text_closure.forget();

    options.append_child(&proxies_txt_button)?;

    let proxies_file_select = document.create_element("input")?.dyn_into::<HtmlInputElement>()?;
    proxies_file_select.set_id(PROXIES_FILE_SELECT_ID);
    proxies_file_select.set_type("file");
    proxies_file_select.set_accept("application/json");
    options.append_child(&proxies_file_select)?;

    let proxies_file_button = document.create_element("input")?.dyn_into::<HtmlInputElement>()?;
    proxies_file_button.set_id(PROXIES_FILE_BUTTON_ID);
    proxies_file_button.set_value("Generate proxies");
    proxies_file_button.set_type("button");

    let generate_proxies_file_closure = Closure::wrap(Box::new(generate_proxies_from_file_callback) as Box<dyn FnMut(_)>);
    proxies_file_button.set_onclick(Some(generate_proxies_file_closure.as_ref().unchecked_ref()));
    generate_proxies_file_closure.forget();

    options.append_child(&proxies_file_button)?;

    Ok(())
}