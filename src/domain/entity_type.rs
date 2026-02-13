use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Roaster,
    Roast,
    Bag,
    Brew,
    Cup,
    Cafe,
    Gear,
}

impl EntityType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Roaster => "roaster",
            Self::Roast => "roast",
            Self::Bag => "bag",
            Self::Brew => "brew",
            Self::Cup => "cup",
            Self::Cafe => "cafe",
            Self::Gear => "gear",
        }
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EntityType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "roaster" => Ok(Self::Roaster),
            "roast" => Ok(Self::Roast),
            "bag" => Ok(Self::Bag),
            "brew" => Ok(Self::Brew),
            "cup" => Ok(Self::Cup),
            "cafe" => Ok(Self::Cafe),
            "gear" => Ok(Self::Gear),
            _ => Err(()),
        }
    }
}
