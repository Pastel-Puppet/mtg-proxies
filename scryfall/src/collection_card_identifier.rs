use core::fmt::Display;
use alloc::{borrow::ToOwned, string::{String, ToString}};
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

impl Display for CollectionCardIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = match self {
            CollectionCardIdentifier::Id(uuid) => "Id(".to_owned() + &uuid.to_string() + ")",
            CollectionCardIdentifier::MtgoId(id) => "MtgoId(".to_owned() + &id.to_string() + ")",
            CollectionCardIdentifier::MultiverseId(id) => "MultiverseId(".to_owned() + &id.to_string() + ")",
            CollectionCardIdentifier::OracleId(uuid) => "OracleId(".to_owned() + &uuid.to_string() + ")",
            CollectionCardIdentifier::IllustrationId(uuid) => "IllustrationId(".to_owned() + &uuid.to_string() + ")",
            CollectionCardIdentifier::Name(name) => "Name(".to_owned() + &name.to_string() + ")",
            CollectionCardIdentifier::NameSet((name, set)) => "NameSet(".to_owned() + name + ", " + set + ")",
            CollectionCardIdentifier::CollectorNumberSet((collector_number, set)) => "CollectorNumberSet(".to_owned() + collector_number + ", " + set + ")",
        };

        write!(f, "{}", text)
    }
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