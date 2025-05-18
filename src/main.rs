mod generate_proxies;

use std::io::Write;

use clap::Parser;
use clio::{Input, OutputPath};

use generate_proxies::generate_proxies::{generate_proxies_html, generate_proxies_html_from_cards};
use scryfall::{api_interface::api_interface::ApiInterface, deck_formats::deck_formats::{images_from_json_file, parse_txt_file}, fetch_card_list::fetch_card_list::resolve_cards};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    deck: Input,
    #[clap(long, short, value_parser, default_value="proxies.html")]
    output: OutputPath,
    #[arg(long)]
    exclude_basic_lands: bool,
    extra_cards: Vec<String>,
}

fn main() {
    let mut args = Args::parse();

    let mut interface = ApiInterface::new().expect("Could not initialise HTTP client");

    let Some(deck_file_extension) = args.deck.path().extension() else {
        panic!("Could not find extension of file {}", args.deck.path());
    };

    let proxies_html = match deck_file_extension.to_string_lossy().to_string().as_str() {
        "txt" => {
            let cards = resolve_cards(&mut parse_txt_file(args.deck.get_file().expect("Could not open deck file")).expect("Could not parse deck file"), &mut interface).expect("Could not resolve deck cards");
            for card in &cards {
                println!("{} {}", card.count, card.card.name);
            }
            generate_proxies_html_from_cards(&cards, &args.extra_cards, args.exclude_basic_lands).expect("Could not generate proxies HTML content")
        },
        "json" => {
            let card_images = images_from_json_file(args.deck.get_file().expect("Could not open deck file"), args.exclude_basic_lands).expect("Could not parse deck file");
            generate_proxies_html(&card_images, &args.extra_cards).expect("Could not generate proxies HTML content")
        },
        _ => panic!("File extension {} is not supported", deck_file_extension.to_string_lossy()),
    };

    args.output.create().expect("Could not create proxies HTML file").write_all(proxies_html.as_bytes()).expect("Could not write proxies HTML file");
}
