//! Shared AUSH/Rush migration compatibility helpers.
//!
//! The shell is being rebranded from Rush to AUSH. Runtime surfaces must remain
//! compatible with existing users while new installs and docs move to AUSH.

use std::env;
use std::path::PathBuf;

/// Get an AUSH-prefixed environment variable with fallback to its legacy
/// RUSH-prefixed name.
pub fn env_var(primary: &str, legacy: &str) -> Option<String> {
    env::var(primary).ok().or_else(|| env::var(legacy).ok())
}

/// Get a flag-like environment variable with AUSH preferred over Rush.
pub fn env_flag(primary: &str, legacy: &str, enabled_value: &str) -> bool {
    env_var(primary, legacy).as_deref() == Some(enabled_value)
}

/// Return `~/.config/aush/<name>` for new state files.
pub fn xdg_config_file(name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".config").join("aush").join(name))
}

/// Return the legacy `~/.config/rush/<name>` path.
pub fn legacy_xdg_config_file(name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".config").join("rush").join(name))
}

/// Prefer the AUSH config path for writes, but read existing Rush state when it
/// already exists so users do not silently lose data during migration.
pub fn migrated_xdg_config_file(name: &str) -> Option<PathBuf> {
    let primary = xdg_config_file(name)?;
    if primary.exists() {
        return Some(primary);
    }

    if let Some(legacy) = legacy_xdg_config_file(name) {
        if legacy.exists() {
            return Some(legacy);
        }
    }

    Some(primary)
}

/// Return the first existing path, or the primary path when none exist.
pub fn first_existing_or_primary(
    primary: PathBuf,
    fallbacks: impl IntoIterator<Item = PathBuf>,
) -> PathBuf {
    if primary.exists() {
        return primary;
    }

    for fallback in fallbacks {
        if fallback.exists() {
            return fallback;
        }
    }

    primary
}
