#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod api_interface;
pub mod collection_card_identifier;
pub mod api_classes;
pub mod fetch_card_list;
pub mod deck_formats;
pub mod card_images_helper;
#[cfg(feature = "std")]
pub mod reqwest_wrapper;
#[cfg(feature = "wasm")]
pub mod wasm_fetch_wrapper;
pub mod token_handling;