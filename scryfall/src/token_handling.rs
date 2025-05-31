use crate::api_classes::RelatedCard;

pub fn is_token(card: &RelatedCard) -> bool {
    if card.component == "token" {
        return true;
    }

    let token_types = ["token", "emblem", "card"];

    for word in card.type_line.split_ascii_whitespace() {
        for token_type in token_types {
            if word.to_ascii_lowercase().contains(token_type) {
                if token_type == "card" && card.name.to_ascii_lowercase().contains("checklist") {
                    continue;
                }

                return true
            }
        }
    }

    false
}