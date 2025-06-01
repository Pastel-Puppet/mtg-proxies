use crate::api_classes::{Card, RelatedCard};

pub trait Token {
    fn is_token(&self) -> bool;
}

impl Token for Card {
    fn is_token(&self) -> bool {
        let Some(type_line) = &self.type_line else {
            return false;
        };

        is_typeline_or_name_token(type_line, &self.name)
    }
}

impl Token for RelatedCard {
    fn is_token(&self) -> bool {
        if self.component == "token" {
            return true;
        }

        is_typeline_or_name_token(&self.type_line, &self.name)
    }
}

fn is_typeline_or_name_token(type_line: &str, name: &str) -> bool {
    let token_types = ["token", "emblem", "card"];

    for word in type_line.split_ascii_whitespace() {
        for token_type in token_types {
            if word.to_ascii_lowercase().contains(token_type) {
                if token_type == "card" && name.to_ascii_lowercase().contains("checklist") {
                    continue;
                }

                return true
            }
        }
    }

    false
}