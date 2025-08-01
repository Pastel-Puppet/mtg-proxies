#![cfg_attr(not(feature = "std"), no_std)]
#![feature(test)]
extern crate alloc;

pub mod api_interface;
pub mod fetch_card_list;
pub mod deck_formats;
pub mod card_images_helper;
pub mod token_handling;