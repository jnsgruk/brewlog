use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub i64);

        impl $name {
            pub const fn new(value: i64) -> Self {
                Self(value)
            }

            pub const fn into_inner(self) -> i64 {
                self.0
            }
        }

        impl From<i64> for $name {
            fn from(value: i64) -> Self {
                Self(value)
            }
        }

        impl From<$name> for i64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = ParseIntError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let value = s.parse::<i64>()?;
                Ok(Self(value))
            }
        }
    };
}

define_id!(RoasterId);
define_id!(RoastId);
define_id!(TimelineEventId);
define_id!(UserId);
define_id!(TokenId);
define_id!(SessionId);
define_id!(BagId);
define_id!(GearId);
define_id!(BrewId);
define_id!(CafeId);
define_id!(CupId);
define_id!(PasskeyCredentialId);
define_id!(RegistrationTokenId);
