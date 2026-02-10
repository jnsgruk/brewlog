pub mod analytics;
pub mod auth;
pub mod coffee;
pub mod images;
pub(crate) mod macros;
pub mod pagination;

// Re-exports for backward compatibility
pub use analytics::{ai_usage, stats, timeline_events};
pub use auth::{passkey_credentials, registration_tokens, sessions, tokens, users};
pub use coffee::{bags, brews, cafes, cups, gear, roasters, roasts};
