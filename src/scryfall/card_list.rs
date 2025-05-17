pub mod card_list {
    use std::{collections::HashMap, error::Error, fmt::Display};

    use url::Url;

    use crate::scryfall::{api_classes::api_classes::{ApiObject, Card, CardNotFound}, api_interface::api_interface::ApiInterface, collection_card_identifier::collection_card_identifier::CollectionCardIdentifier};

    #[derive(Debug, Clone)]
    enum CardParseErrorCause {
        ObjectNotCard(ApiObject),
        ObjectNotList(ApiObject),
        //CardCountNotFound(String),
    }

    #[derive(Debug, Clone)]
    pub struct CardParseError {
        cause: CardParseErrorCause,
    }

    impl Display for CardParseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.cause {
                CardParseErrorCause::ObjectNotCard(object) => write!(f, "API returned object other than a card:\n{:?}", object),
                CardParseErrorCause::ObjectNotList(object) => write!(f, "API returned object other than a list:\n{:?}", object),
                //CardParseErrorCause::CardCountNotFound(identifier) => write!(f, "Card {} could not be found in unresolved list", identifier),
            }
        }
    }

    impl Error for CardParseError {}

    #[derive(Debug)]
    pub struct ResolvedCard {
        pub count: usize,
        pub card: Card,
    }

    pub struct CardList {
        unresolved_cards: HashMap<String, usize>,
        pub resolved_cards: Vec<ResolvedCard>,
    }

    impl CardList {
        pub fn new() -> Self {
            Self {
                unresolved_cards: HashMap::new(),
                resolved_cards: Vec::new(),
            }
        }

        pub fn add(&mut self, line: String) {
            let text = line.trim();
            if text.len() == 0 || text.starts_with("//") {
                return
            }

            match text.split_once(" ") {
                None => self.unresolved_cards.insert(text.to_string(), 1),
                Some((digits, card_name)) => {
                    match digits.parse() {
                        Err(_) => self.unresolved_cards.insert(text.to_string(), 1),
                        Ok(digits) => self.unresolved_cards.insert(card_name.to_string(), digits),
                    }
                },
            };
        }

        fn fuzzy_resolve(&mut self, api_interface: &mut ApiInterface, identifier: &str) -> Result<(), Box<dyn Error>> {
            let count = match self.unresolved_cards.get(identifier) {
                Some(count) => count,
                None => {
                    println!("Could not find card {} on the deck list, assuming it has one copy", identifier);
                    &1
                    //return Err(Box::new(CardParseError { cause: CardParseErrorCause::CardCountNotFound(card.name) }))
                },
            };

            let object = api_interface.get_card(&parse_card_name(identifier))?;
            if let ApiObject::Card(card) = object {
                self.resolved_cards.push(ResolvedCard { count: *count, card: card });
                Ok(())
            } else {
                Err(Box::new(CardParseError { cause: CardParseErrorCause::ObjectNotCard(object) }))
            }
        }

        pub fn resolve_cards(&mut self, api_interface: &mut ApiInterface) -> Result<(), Box<dyn Error>> {
            let mut not_found_cards_list: Vec<CardNotFound> = Vec::new();

            for unresolved_cards in self.unresolved_cards.keys().collect::<Vec<&String>>().chunks(75) {
                let unresolved_card_identifiers: Vec<CollectionCardIdentifier> = unresolved_cards.iter().map(|card: &&String| parse_card_name(card)).collect();
                let list = match api_interface.get_cards_from_list(&unresolved_card_identifiers.as_slice())? {
                    ApiObject::List(list) => list,
                    other => return Err(Box::new(CardParseError { cause: CardParseErrorCause::ObjectNotList(other) })),
                };

                if let Some(not_found_cards) = list.not_found {
                    not_found_cards_list.append(&mut not_found_cards.clone());
                }

                for object in list.data {
                    match object {
                        ApiObject::Card(card) => {
                            let count = match self.unresolved_cards.get(&card.name) {
                                Some(count) => count,
                                None => {
                                    println!("Could not find card {} on the deck list, assuming it has one copy", card.name);
                                    &1
                                    //return Err(Box::new(CardParseError { cause: CardParseErrorCause::CardCountNotFound(card.name) }))
                                },
                            };

                            self.resolved_cards.push(ResolvedCard { count: *count, card: card })
                        },
                        other => return Err(Box::new(CardParseError { cause: CardParseErrorCause::ObjectNotCard(other) })),
                    }
                }
            }

            for not_found_card in not_found_cards_list {
                self.fuzzy_resolve(api_interface, &not_found_card.name)?;
            }

            Ok(())
        }

        pub fn extract_images(&self) -> Vec<Url> {
            let mut image_list = Vec::new();

            for card in &self.resolved_cards {
                if let Some(faces) = &card.card.card_faces {
                    for face in faces {
                        if let Some(image_uris) = &face.image_uris {
                            image_list.push(image_uris.png.clone());
                        }
                    }
                } else {
                    if let Some(image_uris) = &card.card.image_uris {
                        image_list.push(image_uris.png.clone());
                    }
                }
            }

            image_list
        }
    }

    fn parse_card_name(name: &str) -> CollectionCardIdentifier {
        let name = name.trim();
        if name.starts_with("[") && name.ends_with("]") {
            let name = name.trim_start_matches("[").trim_end_matches("]");
            if let Some((set, collector_number)) = name.split_once("/") {
                CollectionCardIdentifier::CollectorNumberSet((collector_number.to_string(), set.to_string()))
            } else {
                CollectionCardIdentifier::Name(name.to_string())
            }
        } else {
            CollectionCardIdentifier::Name(name.to_string())
        }
    }
}