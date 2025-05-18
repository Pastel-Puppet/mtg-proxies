use uuid::Uuid;
use serde::{ser::SerializeStruct, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CollectionCardIdentifier {
    Id(Uuid),
    MtgoId(usize),
    MultiverseId(usize),
    OracleId(Uuid),
    IllustrationId(Uuid),
    Name(String),
    NameSet((String, String)),
    CollectorNumberSet((String, String)),
}

impl Serialize for CollectionCardIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let mut root = match self {
            Self::NameSet(_) | Self::CollectorNumberSet(_) => serializer.serialize_struct("CollectionCardIdentifier", 2)?,
            _ => serializer.serialize_struct("CollectionCardIdentifier", 1)?,
        };

        match self {
            CollectionCardIdentifier::Id(id) => root.serialize_field("id", id)?,
            CollectionCardIdentifier::MtgoId(mtgo_id) => root.serialize_field("mtgo_id", mtgo_id)?,
            CollectionCardIdentifier::MultiverseId(multiverse_id) => root.serialize_field("multiverse_id", multiverse_id)?,
            CollectionCardIdentifier::OracleId(oracle_id) => root.serialize_field("oracle_id", oracle_id)?,
            CollectionCardIdentifier::IllustrationId(illustration_id) => root.serialize_field("illustration_id", illustration_id)?,
            CollectionCardIdentifier::Name(name) => root.serialize_field("name", name)?,
            CollectionCardIdentifier::NameSet((name, set)) => 
            {
                root.serialize_field("set", set)?;
                root.serialize_field("name", name)?;
            },
            CollectionCardIdentifier::CollectorNumberSet((collector_number, set)) => {
                root.serialize_field("set", set)?;
                root.serialize_field("collector_number", collector_number)?;
            },
        };

        root.end()
    }
}