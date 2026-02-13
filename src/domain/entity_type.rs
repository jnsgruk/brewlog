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

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_VARIANTS: [EntityType; 7] = [
        EntityType::Roaster,
        EntityType::Roast,
        EntityType::Bag,
        EntityType::Brew,
        EntityType::Cup,
        EntityType::Cafe,
        EntityType::Gear,
    ];

    #[test]
    fn as_str_roundtrip() {
        for variant in ALL_VARIANTS {
            assert_eq!(variant.as_str().parse::<EntityType>(), Ok(variant));
        }
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", EntityType::Brew), "brew");
    }

    #[test]
    fn from_str_unknown_is_err() {
        assert!("unknown".parse::<EntityType>().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        for variant in ALL_VARIANTS {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: EntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }
}
