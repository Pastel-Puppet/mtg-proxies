use core::{fmt::Display, hash::{Hasher, Hash}};
use alloc::{borrow::ToOwned, string::{String, ToString}};
use uuid::Uuid;
use serde::{ser::SerializeStruct, Serialize, Deserialize};

#[derive(Deserialize, Debug, Clone, Eq)]
#[serde(untagged)]
pub enum CollectionCardIdentifier {
    Id { id: Uuid },
    MtgoId { mtgo_id: usize },
    MultiverseId { multiverse_id: usize },
    OracleId { oracle_id: Uuid },
    IllustrationId { illustration_id: Uuid },
    Name { name: String },
    NameSet { name: String, set: String },
    CollectorNumberSet { collector_number: String, set: String},
}

impl Hash for CollectionCardIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            CollectionCardIdentifier::Id { id } => id.hash(state),
            CollectionCardIdentifier::MtgoId { mtgo_id } => mtgo_id.hash(state),
            CollectionCardIdentifier::MultiverseId { multiverse_id}  => multiverse_id.hash(state),
            CollectionCardIdentifier::OracleId { oracle_id } => oracle_id.hash(state),
            CollectionCardIdentifier::IllustrationId { illustration_id } => illustration_id.hash(state),
            CollectionCardIdentifier::Name { name } => name.to_ascii_lowercase().hash(state),
            CollectionCardIdentifier::NameSet { name, set } => {
                name.to_ascii_lowercase().hash(state);
                set.to_ascii_lowercase().hash(state);
            },
            CollectionCardIdentifier::CollectorNumberSet { collector_number, set } => {
                collector_number.to_ascii_lowercase().hash(state);
                set.to_ascii_lowercase().hash(state);
            },
        }
    }
}

impl PartialEq for CollectionCardIdentifier {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Id { id: l0 }, Self::Id { id: r0 }) => l0 == r0,
            (Self::MtgoId { mtgo_id: l0 }, Self::MtgoId { mtgo_id: r0 }) => l0 == r0,
            (Self::MultiverseId { multiverse_id: l0 }, Self::MultiverseId { multiverse_id: r0 }) => l0 == r0,
            (Self::OracleId { oracle_id: l0 }, Self::OracleId { oracle_id: r0 }) => l0 == r0,
            (Self::IllustrationId { illustration_id: l0 }, Self::IllustrationId { illustration_id: r0 }) => l0 == r0,
            (Self::Name { name: l0 }, Self::Name {name: r0 }) => l0.to_ascii_lowercase() == r0.to_ascii_lowercase(),
            (Self::NameSet { name: l0, set: l1 }, Self::NameSet { name: r0, set: r1 }) => l0.to_ascii_lowercase() == r0.to_ascii_lowercase() && l1.to_ascii_lowercase() == r1.to_ascii_lowercase(),
            (Self::CollectorNumberSet { collector_number: l0, set: l1 }, Self::CollectorNumberSet { collector_number: r0, set: r1 }) => l0.to_ascii_lowercase() == r0.to_ascii_lowercase() && l1.to_ascii_lowercase() == r1.to_ascii_lowercase(),
            _ => false,
        }
    }
}

impl Display for CollectionCardIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = match self {
            CollectionCardIdentifier::Id { id } => "Id(".to_owned() + &id.to_string() + ")",
            CollectionCardIdentifier::MtgoId { mtgo_id } => "MtgoId(".to_owned() + &mtgo_id.to_string() + ")",
            CollectionCardIdentifier::MultiverseId { multiverse_id } => "MultiverseId(".to_owned() + &multiverse_id.to_string() + ")",
            CollectionCardIdentifier::OracleId { oracle_id } => "OracleId(".to_owned() + &oracle_id.to_string() + ")",
            CollectionCardIdentifier::IllustrationId { illustration_id } => "IllustrationId(".to_owned() + &illustration_id.to_string() + ")",
            CollectionCardIdentifier::Name { name } => "Name(".to_owned() + &name.to_string() + ")",
            CollectionCardIdentifier::NameSet { name, set } => "NameSet(".to_owned() + name + ", " + set + ")",
            CollectionCardIdentifier::CollectorNumberSet { collector_number, set } => "CollectorNumberSet(".to_owned() + collector_number + ", " + set + ")",
        };

        write!(f, "{text}")
    }
}

impl Serialize for CollectionCardIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let mut root = match self {
            Self::NameSet { name: _, set: _ } | Self::CollectorNumberSet { collector_number: _, set: _ } => serializer.serialize_struct("CollectionCardIdentifier", 2)?,
            _ => serializer.serialize_struct("CollectionCardIdentifier", 1)?,
        };

        match self {
            CollectionCardIdentifier::Id { id } => root.serialize_field("id", id)?,
            CollectionCardIdentifier::MtgoId { mtgo_id } => root.serialize_field("mtgo_id", mtgo_id)?,
            CollectionCardIdentifier::MultiverseId { multiverse_id } => root.serialize_field("multiverse_id", multiverse_id)?,
            CollectionCardIdentifier::OracleId { oracle_id } => root.serialize_field("oracle_id", oracle_id)?,
            CollectionCardIdentifier::IllustrationId { illustration_id } => root.serialize_field("illustration_id", illustration_id)?,
            CollectionCardIdentifier::Name { name } => root.serialize_field("name", name)?,
            CollectionCardIdentifier::NameSet { name, set } => 
            {
                root.serialize_field("set", set)?;
                root.serialize_field("name", name)?;
            },
            CollectionCardIdentifier::CollectorNumberSet { collector_number, set } => {
                root.serialize_field("set", set)?;
                root.serialize_field("collector_number", collector_number)?;
            },
        };

        root.end()
    }
}