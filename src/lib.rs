use std::sync::OnceLock;

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub struct VersionInfo {
    pub version: &'static str,
    pub commit: &'static str,
}

pub const VERSION_INFO: VersionInfo = VersionInfo {
    version: env!("CARGO_PKG_VERSION"),
    commit: env!("GIT_HASH"),
};

static BASE_URL: OnceLock<String> = OnceLock::new();

pub fn set_base_url(url: String) {
    let _ = BASE_URL.set(url);
}

pub fn base_url() -> &'static str {
    BASE_URL.get().map_or("", std::string::String::as_str)
}
