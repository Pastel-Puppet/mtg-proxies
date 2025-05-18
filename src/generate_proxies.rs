use std::error::Error;
use url::Url;

use scryfall::fetch_card_list::ResolvedCard;

fn extract_images(cards: &Vec<ResolvedCard>) -> Vec<(Url, usize)> {
    let mut image_list = Vec::new();

    for card in cards {
        if let Some(faces) = &card.card.card_faces {
            for face in faces {
                if let Some(image_uris) = &face.image_uris {
                    image_list.push((image_uris.png.clone(), card.count));
                }
            }
        } else if let Some(image_uris) = &card.card.image_uris {
            image_list.push((image_uris.png.clone(), card.count));
        }
    }

    image_list
}

pub fn generate_proxies_html_from_cards(cards: &Vec<ResolvedCard>, extra_cards: &Vec<String>, exclude_basic_lands: bool) -> Result<String, Box<dyn Error>> {
    let filtered_cards = if exclude_basic_lands {
        &cards.iter().filter(|card| !card.card.type_line.starts_with("Basic Land")).cloned().collect()
    } else {
        cards
    };

    generate_proxies_html(&extract_images(filtered_cards), extra_cards)
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