mod generate_proxies;

use std::{fs::File, io::Write};

use clap::Parser;
use clio::{Input, OutputPath};

use generate_proxies::{generate_proxies_html, generate_proxies_html_from_cards, ImageUriType};
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
    extra_cards: Vec<String>,
}

fn get_proxies_from_file(deck_file: &File, deck_file_extension: &str, interface: &mut ApiInterface, exclude_basic_lands: bool, image_type: Option<ImageUriType>, extra_cards: &Vec<String>) -> String {
    let cards = match deck_file_extension {
        "txt" => {
            resolve_cards(&mut parse_txt_file(deck_file).expect("Could not parse deck file"), interface).expect("Could not resolve deck cards")
        },
        "json" => {
            if let None = image_type {
                let card_images = images_from_json_file(deck_file, exclude_basic_lands).expect("Could not parse deck file");
                return generate_proxies_html(&card_images, extra_cards).expect("Could not generate proxies HTML content");
            }

            resolve_cards(&mut parse_json_file(deck_file).expect("Could not parse deck file"), interface).expect("Could not resolve deck cards")
        },
        _ => panic!("File extension {} is not supported", deck_file_extension),
    };

    for card in &cards {
        println!("{} {}", card.count, card.card.name);
    }

    generate_proxies_html_from_cards(&cards, extra_cards, exclude_basic_lands, image_type.unwrap_or(ImageUriType::Large)).expect("Could not generate proxies HTML content")
}

fn main() {
    let mut args = Args::parse();

    let mut interface = ApiInterface::new().expect("Could not initialise HTTP client");

    let deck_file_extension = match args.deck.path().extension() {
        Some(extension) => extension.to_string_lossy().into_owned(),
        None => panic!("Could not find extension of file {}", args.deck.path()),
    };

    let deck_file = args.deck.get_file().expect("Could not open deck file");

    let proxies_html = get_proxies_from_file(&deck_file, &deck_file_extension, &mut interface, args.exclude_basic_lands, args.image_type, &args.extra_cards);

    args.output.create().expect("Could not create proxies HTML file").write_all(proxies_html.as_bytes()).expect("Could not write proxies HTML file");
}
