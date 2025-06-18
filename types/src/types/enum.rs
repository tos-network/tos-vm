use std::{borrow::Cow, hash::{Hash, Hasher}, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::IdentifierType;
use super::Type;

// Represents a variant of an enum
// This is similar to a struct
#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct EnumVariant(Vec<(Cow<'static, str>, Type)>);

impl EnumVariant {
    #[inline(always)]
    pub fn new(types: Vec<(Cow<'static, str>, Type)>) -> Self {
        Self(types)
    }

    #[inline(always)]
    pub fn fields(&self) -> &[(Cow<'static, str>, Type)] {
        &self.0
    }
}

impl From<Vec<(Cow<'static, str>, Type)>> for EnumVariant {
    fn from(value: Vec<(Cow<'static, str>, Type)>) -> Self {
        Self(value)
    }
}

impl From<Vec<(&'static str, Type)>> for EnumVariant {
    fn from(value: Vec<(&'static str, Type)>) -> Self {
        Self(value.into_iter().map(|(k, v)| (Cow::Borrowed(k), v)).collect())
    }
}

// Represents an enum like in Rust with variants
// Support up to 255 variants
#[derive(Clone, Eq, Debug, Serialize, Deserialize)]
pub struct Enum {
    id: IdentifierType,
    name: Cow<'static, str>,
    variants: Vec<(Cow<'static, str>, EnumVariant)>,
}

impl Hash for Enum {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Enum {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// Selected enum variant with associated type
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct EnumType(Arc<Enum>);

// Represents the type of an enum variant
// This is embed in the value to determine easily which variant it is
#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct EnumValueType {
    enum_type: EnumType,
    variant_id: u8
}

impl EnumType {
    // Create a new enum type
    pub fn new(id: IdentifierType, name: impl Into<Cow<'static, str>>, variants: Vec<(Cow<'static, str>, EnumVariant)>) -> Self {
        Self(Arc::new(Enum { id, name: name.into(), variants }))
    }

    // Get the unique identifier of the enum
    #[inline(always)]
    pub fn id(&self) -> IdentifierType {
        self.0.id
    }

    // Get the unique identifier of the enum
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.0.name
    }

    // Get the variants of the enum
    #[inline(always)]
    pub fn variants(&self) -> &Vec<(Cow<'static, str>, EnumVariant)> {
        &self.0.variants
    }

    // Get a variant by its id
    #[inline(always)]
    pub fn get_variant(&self, id: u8) -> Option<&(Cow<'static, str>, EnumVariant)> {
        self.0.variants.get(id as usize)
    }
}

impl EnumValueType {
    // Create a new enum value type
    pub fn new(enum_type: EnumType, variant_id: u8) -> Self {
        Self { enum_type, variant_id }
    }

    // Get the unique identifier of the enum
    #[inline(always)]
    pub fn id(&self) -> IdentifierType {
        self.enum_type.id()
    }

    // Get the enum type
    #[inline(always)]
    pub fn enum_type(&self) -> &EnumType {
        &self.enum_type
    }

    // Get the variant id
    #[inline(always)]
    pub fn variant_id(&self) -> u8 {
        self.variant_id
    }
}

impl Serialize for EnumType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for EnumType {
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(Arc::new(Enum::deserialize(deserializer)?)))
    }
}