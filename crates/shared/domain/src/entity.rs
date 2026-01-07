use crate::constants::{QUIZ, STUDENT, SURVEY, USER, WORKSPACE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Entity {
    Workspace,
    User,
    Student,
    Quiz,
    Survey,
}

impl Entity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => WORKSPACE,
            Self::User => USER,
            Self::Student => STUDENT,
            Self::Quiz => QUIZ,
            Self::Survey => SURVEY,
        }
    }
}

impl TryFrom<&str> for Entity {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            WORKSPACE => Ok(Self::Workspace),
            USER => Ok(Self::User),
            STUDENT => Ok(Self::Student),
            QUIZ => Ok(Self::Quiz),
            SURVEY => Ok(Self::Survey),
            _ => Err("Unknown entity type"),
        }
    }
}
