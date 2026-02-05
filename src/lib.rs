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
