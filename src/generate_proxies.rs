use std::error::Error;
use clap::ValueEnum;
use url::Url;

use scryfall::{api_classes::ImageUris, fetch_card_list::ResolvedCard};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
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

pub fn extract_images(cards: &Vec<ResolvedCard>, exclude_basic_lands: bool, image_type: ImageUriType) -> Vec<(Url, usize)> {
    let filtered_cards = if exclude_basic_lands {
        &cards.iter().filter(|card| !card.card.type_line.starts_with("Basic Land")).cloned().collect()
    } else {
        cards
    };

    let mut image_list = Vec::new();

    for card in filtered_cards {
        if let Some(faces) = &card.card.card_faces {
            for face in faces {
                if let Some(image_uris) = &face.image_uris {
                    image_list.push((extract_image(image_uris, image_type), card.count));
                }
            }
        } else if let Some(image_uris) = &card.card.image_uris {
            image_list.push((extract_image(image_uris, image_type), card.count));
        }
    }

    image_list
}

pub fn generate_proxies_html(card_images: &Vec<(Url, usize)>, extra_cards: &Vec<String>) -> Result<String, Box<dyn Error>> {
    let mut html = "<!DOCTYPE html><html><style>@page {size: auto;margin: 5mm 10mm;}.card{margin: 0;page-break-inside: avoid;width: 63mm;height: 88mm;}</style><body style=\"margin: 0 0 30px;padding: 0;font-size: 0;\">".to_owned();

    for extra_card in extra_cards {
        html += &format!("<img src=\"{}\" class=\"card\"/>", extra_card);
    }

    for (image_url, count) in card_images {
        for _ in 0..*count {
            html += &format!("<img src={} class=\"card\"/>", image_url);
        }
    }

    html += "</body></html>";

    Ok(html)
}