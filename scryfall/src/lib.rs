pub mod api_interface;
pub mod collection_card_identifier;
pub mod api_classes;
pub mod fetch_card_list;
pub mod deck_formats;
pub mod card_images_helper;

#[cfg(not(target_family = "wasm"))]
pub mod reqwest_wrapper;