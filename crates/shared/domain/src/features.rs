use crate::constants::{QUIZ, SURVEY};
use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

bitflags! {
    /// Represents a set of features.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Features: u32 {
        const QUIZ = 1 << 0;
        const SURVEY = 1 << 1;

        const ALL = Self::QUIZ.bits() | Self::SURVEY.bits();
    }
}

impl From<&str> for Features {
    fn from(s: &str) -> Self {
        match s {
            QUIZ => Self::QUIZ,
            SURVEY => Self::SURVEY,
            "all" | "*" => Self::ALL,
            _ => Self::empty(),
        }
    }
}

impl From<u32> for Features {
    fn from(bits: u32) -> Self {
        Self::from_bits_truncate(bits)
    }
}

impl Serialize for Features {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

impl<'de> Deserialize<'de> for Features {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u32::deserialize(deserializer)?;
        Ok(Self::from_bits_retain(bits))
    }
}
