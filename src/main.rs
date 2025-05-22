mod generate_proxies;

use std::{fs::File, io::Write};
use url::Url;

use clap::Parser;
use clio::{Input, OutputPath};

use generate_proxies::{extract_images, generate_proxies_html, ImageUriType};
use scryfall::{api_interface::ApiInterface, deck_formats::{images_from_json_file, parse_json_file, parse_txt_file}, fetch_card_list::resolve_cards};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    deck: Input,
    #[clap(long, short, value_parser, default_value="proxies.html")]
    output: OutputPath,
    #[arg(short, long)]
    exclude_basic_lands: bool,
    #[arg(long, short, value_enum)]
    image_type: Option<ImageUriType>,
    #[arg(short, long)]
    include_tokens: bool,
    #[arg(short, long)]
    verbose: bool,
    extra_cards: Vec<String>,
}

fn get_card_images_from_file(deck_file: &File, deck_file_extension: &str, interface: &mut ApiInterface, exclude_basic_lands: bool, image_type: Option<ImageUriType>, include_tokens: bool) -> Vec<(Url, usize)> {
    let mut unresolved_cards = match deck_file_extension {
        "txt" => {
            parse_txt_file(deck_file).expect("Could not parse deck file")
        },
        "json" => {
            if let None = image_type {
                return images_from_json_file(deck_file, exclude_basic_lands).expect("Could not parse deck file");
            }

            parse_json_file(deck_file).expect("Could not parse deck file")
        },
        _ => panic!("File extension {} is not supported", deck_file_extension),
    };

    let cards = resolve_cards(&mut unresolved_cards, include_tokens, interface).expect("Could not resolve deck cards");

    for card in &cards {
        println!("{} {}", card.count, card.card.name);
    }

    extract_images(&cards, exclude_basic_lands, image_type.unwrap_or(ImageUriType::Large))
}

fn main() {
    let mut args = Args::parse();

    let mut interface = ApiInterface::new(args.verbose).expect("Could not initialise HTTP client");

    let deck_file_extension = match args.deck.path().extension() {
        Some(extension) => extension.to_string_lossy().into_owned(),
        None => panic!("Could not find extension of file {}", args.deck.path()),
    };

    let deck_file = args.deck.get_file().expect("Could not open deck file");

    let card_images = get_card_images_from_file(&deck_file, &deck_file_extension, &mut interface, args.exclude_basic_lands, args.image_type, args.include_tokens);
    let proxies_html = generate_proxies_html(&card_images, &args.extra_cards).expect("Could not generate proxies HTML content");

    args.output.create().expect("Could not create proxies HTML file").write_all(proxies_html.as_bytes()).expect("Could not write proxies HTML file");
}
