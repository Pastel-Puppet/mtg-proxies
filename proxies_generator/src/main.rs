use std::{collections::HashSet, error::Error, hash::RandomState, io::Write};

use clap::{Parser, ValueEnum};
use clio::{Input, OutputPath};
use url::Url;

use scryfall::{api_interface::ApiInterface, card_images_helper::{extract_images, ImageUriType}, deck_formats::{parse_json_file, parse_txt_file}, fetch_card_list::{resolve_cards, ResolvedCard}, reqwest_wrapper::ReqwestWrapper};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ImageType {
    Small,
    Normal,
    Large,
    ArtCrop,
    BorderCrop,
    Png,
}

impl From<ImageType> for ImageUriType {
    fn from(value: ImageType) -> Self {
        match value {
            ImageType::Small => ImageUriType::Small,
            ImageType::Normal => ImageUriType::Normal,
            ImageType::Large => ImageUriType::Large,
            ImageType::ArtCrop => ImageUriType::ArtCrop,
            ImageType::BorderCrop => ImageUriType::BorderCrop,
            ImageType::Png => ImageUriType::Png,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    deck: Input,
    #[clap(value_parser, default_value="proxies.html")]
    output: OutputPath,
    #[arg(short, long)]
    exclude_basic_lands: bool,
    #[arg(long, short, value_enum)]
    image_type: Option<ImageType>,
    #[arg(short, long)]
    include_tokens: bool,
    #[arg(short, long)]
    verbose: bool,
    extra_cards: Vec<String>,
    #[clap(short, long, value_parser)]
    old_deck: Option<Input>,
}

fn generate_proxies_html(card_images: &Vec<(Url, usize)>, extra_cards: &Vec<String>) -> Result<String, Box<dyn Error>> {
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

async fn get_cards_from_file(deck_file: &mut Input, interface: &mut ApiInterface<ReqwestWrapper>, include_tokens: bool) -> Vec<ResolvedCard> {
    let deck_file_extension = match deck_file.path().extension() {
        Some(extension) => extension.to_string_lossy().into_owned(),
        None => panic!("Could not find extension of file {}", deck_file.path()),
    };

    let deck_file = deck_file.get_file().expect("Could not open deck file");

    let mut unresolved_cards = match deck_file_extension.as_str() {
        "txt" => {
            parse_txt_file(deck_file).expect("Could not parse deck file")
        },
        "json" => {
            parse_json_file(deck_file).expect("Could not parse deck file")
        },
        _ => panic!("File extension {} is not supported", deck_file_extension),
    };

    resolve_cards(&mut unresolved_cards, include_tokens, interface).await.expect("Could not resolve deck cards")
}

#[tokio::main]
async fn main() {
    let mut args = Args::parse();

    let mut interface = ApiInterface::<ReqwestWrapper>::new(args.verbose).expect("Could not initialise HTTP client");

    let cards = get_cards_from_file(&mut args.deck, &mut interface, args.include_tokens).await;

    let card_images = if let Some(mut old_deck) = args.old_deck {
        let cards_set: HashSet<ResolvedCard, RandomState> = HashSet::from_iter(cards);

        let old_cards = get_cards_from_file(&mut old_deck, &mut interface, args.include_tokens).await;
        let old_cards_set: HashSet<ResolvedCard, RandomState> = HashSet::from_iter(old_cards);

        let added_cards = cards_set.difference(&old_cards_set);
        let removed_cards = old_cards_set.difference(&cards_set);

        println!("Added:{}\n", added_cards.clone().fold("".to_owned(), |acc, card| format!("{}\n{}", acc, card)));
        println!("Removed:{}\n", removed_cards.fold("".to_owned(), |acc, card| format!("{}\n{}", acc, card)));

        let added_cards_vec = Vec::from_iter(added_cards);

        extract_images(&added_cards_vec, args.exclude_basic_lands, args.image_type.unwrap_or(ImageType::Large).into())
    } else {
        for card in &cards {
            println!("{}", card);
        }

        extract_images(&Vec::from_iter(cards.iter()), args.exclude_basic_lands, args.image_type.unwrap_or(ImageType::Large).into())
    };

    let proxies_html = generate_proxies_html(&card_images, &args.extra_cards).expect("Could not generate proxies HTML content");

    args.output.create().expect("Could not create proxies HTML file").write_all(proxies_html.as_bytes()).expect("Could not write proxies HTML file");
}
