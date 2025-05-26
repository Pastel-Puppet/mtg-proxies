mod generate_proxies;

use std::{collections::HashSet, hash::RandomState, io::Write};

use clap::Parser;
use clio::{Input, OutputPath};

use generate_proxies::{extract_images, generate_proxies_html, ImageUriType};
use scryfall::{api_interface::ApiInterface, deck_formats::{parse_json_file, parse_txt_file}, fetch_card_list::{resolve_cards, ResolvedCard}};

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
    image_type: Option<ImageUriType>,
    #[arg(short, long)]
    include_tokens: bool,
    #[arg(short, long)]
    verbose: bool,
    extra_cards: Vec<String>,
    #[clap(short, long, value_parser)]
    old_deck: Option<Input>,
}

fn get_cards_from_file(deck_file: &mut Input, interface: &mut ApiInterface, include_tokens: bool) -> Vec<ResolvedCard> {
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

    resolve_cards(&mut unresolved_cards, include_tokens, interface).expect("Could not resolve deck cards")
}

fn main() {
    let mut args = Args::parse();

    let mut interface = ApiInterface::new(args.verbose).expect("Could not initialise HTTP client");

    let cards = get_cards_from_file(&mut args.deck, &mut interface, args.include_tokens);

    let card_images = if let Some(mut old_deck) = args.old_deck {
        let cards_set: HashSet<ResolvedCard, RandomState> = HashSet::from_iter(cards);

        let old_cards = get_cards_from_file(&mut old_deck, &mut interface, args.include_tokens);
        let old_cards_set: HashSet<ResolvedCard, RandomState> = HashSet::from_iter(old_cards);

        let added_cards = cards_set.difference(&old_cards_set);
        let removed_cards = old_cards_set.difference(&cards_set);

        println!("Added:{}\n", added_cards.clone().fold("".to_owned(), |acc, card| format!("{}\n{}", acc, card)));
        println!("Removed:{}\n", removed_cards.fold("".to_owned(), |acc, card| format!("{}\n{}", acc, card)));

        let added_cards_vec = Vec::from_iter(added_cards);

        extract_images(&added_cards_vec, args.exclude_basic_lands, args.image_type.unwrap_or(ImageUriType::Large))
    } else {
        for card in &cards {
            println!("{}", card);
        }

        extract_images(&Vec::from_iter(cards.iter()), args.exclude_basic_lands, args.image_type.unwrap_or(ImageUriType::Large))
    };

    let proxies_html = generate_proxies_html(&card_images, &args.extra_cards).expect("Could not generate proxies HTML content");

    args.output.create().expect("Could not create proxies HTML file").write_all(proxies_html.as_bytes()).expect("Could not write proxies HTML file");
}
