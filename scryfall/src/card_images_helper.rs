use alloc::{string::{String, ToString}, vec::Vec};
use log::error;

use crate::api_classes::{Card, ImageUris};

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