mod scryfall;

use std::{fs::{read_to_string, File}, io::Write};

use scryfall::{api_interface::api_interface::ApiInterface, card_list::card_list::CardList};

fn main() {
    let mut interface = ApiInterface::new().expect("Could not initialise HTTP client");

    let mut deck_list = CardList::new();

    let deck_file = read_to_string("/workspaces/mtg_proxies/deck.txt").expect("Oops");
    for deck_file_line in deck_file.lines() {
        deck_list.add(deck_file_line.to_string());
    }

    deck_list.resolve_cards(&mut interface).expect("Oops");

    for resolved_card in &deck_list.resolved_cards {
        println!("{} {}", resolved_card.count, resolved_card.card.name);
    }

    let mut html = "<!DOCTYPE html><html><style>@media print {@page {size: auto;margin: 5mm 10mm;}}</style><body style=\"margin: 0 0 30px;padding: 0;font-size: 0;\">".to_owned();

    for image_url in &deck_list.extract_images() {
        html += &format!("<img src={} style=\"margin: 0;page-break-inside: avoid;width: 63mm;height: 88mm;\"/>", image_url);
    }

    html += "</body></html>";

    let mut html_file = File::create("proxies.html").expect("Oops");
    html_file.write(html.as_bytes()).expect("Oops");
}
