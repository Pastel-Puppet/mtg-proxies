#[cfg(feature = "std")]
use core::error::Error;
#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::BufReader;
use alloc::{string::{String, ToString}, vec::Vec};
use log::error;

use crate::api_interface::api_classes::{Card, ImageUris};
#[cfg(feature = "std")]
use crate::api_interface::api_classes::Deck;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageUriType {
    Small,
    Normal,
    Large,
    ArtCrop,
    BorderCrop,
    Png,
}

fn extract_image(images: &ImageUris, image_type: ImageUriType) -> String {
    match image_type {
        ImageUriType::Small => images.small.as_str().to_string(),
        ImageUriType::Normal => images.normal.as_str().to_string(),
        ImageUriType::Large => images.large.as_str().to_string(),
        ImageUriType::ArtCrop => images.art_crop.as_str().to_string(),
        ImageUriType::BorderCrop => images.border_crop.as_str().to_string(),
        ImageUriType::Png => images.png.as_str().to_string(),
    }
}

pub fn extract_images(cards: Vec<Card>, exclude_basic_lands: bool, image_type: ImageUriType) -> Vec<(Card, Vec<String>)> {
    let filtered_cards = if exclude_basic_lands {
        cards.iter().filter(|card| card.type_line.as_ref().is_none_or(|type_line| !type_line.starts_with("Basic Land"))).cloned().collect()
    } else {
        cards
    };

    let mut image_list = Vec::new();

    for card in filtered_cards {
        let mut card_face_urls = Vec::new();
        let mut found_image = false;

        if let Some(faces) = &card.card_faces {
            for face in faces {
                if let Some(image_uris) = &face.image_uris {
                    card_face_urls.push(extract_image(image_uris, image_type));
                    found_image = true;
                }
            }
        }
        
        if let Some(image_uris) = &card.image_uris {
            card_face_urls.push(extract_image(image_uris, image_type));
            found_image = true;
        }

        if found_image {
            image_list.push((card, card_face_urls));
        } else {
            error!("Could not find image data for {}", card.name);
        }
    }

    image_list
}

#[cfg(feature = "std")]
pub fn extract_images_from_json_file(file: &File, exclude_basic_lands: bool) -> Result<Vec<(String, usize)>, Box<dyn Error>> {
    let mut image_list: Vec<(String, usize)> = Vec::new();

    let buffered_reader = BufReader::new(file);
    let deck: Deck = serde_json::from_reader(buffered_reader)?;

    for (section_name, deck_section) in deck.entries.iter() {
        if section_name == "maybeboard" {
            continue;
        }

        for card in deck_section {
            if let Some(card_digest) = &card.card_digest {
                if exclude_basic_lands && card_digest.type_line.starts_with("Basic Land") {
                    continue;
                }

                image_list.push((card_digest.image_uris.front.clone(), card.count));
                if let Some(back_image) = card_digest.image_uris.back.clone() {
                    image_list.push((back_image, card.count));
                }
                continue;
            }

            if card.found {
                error!("Could not find image for card {}", &card.raw_text);
            }
        }
    }

    Ok(image_list)
}