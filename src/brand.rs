//! Shared AUSH branding path/env helpers.

use std::env;
use std::path::PathBuf;

/// Get an environment variable.
pub fn env_var(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

/// Get a flag-like environment variable.
pub fn env_flag(name: &str, enabled_value: &str) -> bool {
    env_var(name).as_deref() == Some(enabled_value)
}

/// Return `~/.config/aush/<name>` for state files.
pub fn xdg_config_file(name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".config").join("aush").join(name))
}
