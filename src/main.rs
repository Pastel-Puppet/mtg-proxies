mod scryfall_interface;

use scryfall_interface::scryfall_interface::CardDataInterface;

#[tokio::main]
async fn main() {
    let interface = match CardDataInterface::new() {
        Ok(interface) => interface,
        Err(error) => panic!("Could not initialise HTTP client: {error:?}"),
    };

    let response = match interface.get_named_card("Errant Street Artist").await {
        Ok(response) => response,
        Err(error) => panic!("Could not retrieve card data: {error:?}"),
    };

    println!("{}", response);
}
