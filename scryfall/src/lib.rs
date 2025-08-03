#![cfg_attr(not(feature = "std"), no_std)]
#![feature(test)]
extern crate alloc;

pub mod api_interface;
pub mod deck_diff;
pub mod deck_parsers;
pub mod card_images_helper;
pub mod token_handling;
pub mod fetch_card_data;