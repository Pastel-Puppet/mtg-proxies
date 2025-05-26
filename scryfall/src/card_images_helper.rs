use url::Url;

use crate::{api_classes::ImageUris, fetch_card_list::ResolvedCard};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageUriType {
    Small,
    Normal,
    Large,
    ArtCrop,
    BorderCrop,
    Png,
}

fn extract_image(images: &ImageUris, image_type: ImageUriType) -> Url {
    match image_type {
        ImageUriType::Small => images.small.clone(),
        ImageUriType::Normal => images.normal.clone(),
        ImageUriType::Large => images.large.clone(),
        ImageUriType::ArtCrop => images.art_crop.clone(),
        ImageUriType::BorderCrop => images.border_crop.clone(),
        ImageUriType::Png => images.png.clone(),
    }
}

pub fn extract_images(cards: &Vec<&ResolvedCard>, exclude_basic_lands: bool, image_type: ImageUriType) -> Vec<(Url, usize)> {
    let filtered_cards = if exclude_basic_lands {
        &cards.iter().filter(|card| card.card.type_line.as_ref().is_none_or(|type_line| !type_line.starts_with("Basic Land"))).cloned().collect()
    } else {
        cards
    };

    let mut image_list = Vec::new();

    for card in filtered_cards {
        let mut found_image = false;

        if let Some(faces) = &card.card.card_faces {
            for face in faces {
                if let Some(image_uris) = &face.image_uris {
                    image_list.push((extract_image(image_uris, image_type), card.count));
                    found_image = true;
                }
            }
        }
        
        if let Some(image_uris) = &card.card.image_uris {
            image_list.push((extract_image(image_uris, image_type), card.count));
            found_image = true;
        }

        if !found_image {
            println!("Could not find image data for {}", card.card.name);
        }
    }

    image_list
}