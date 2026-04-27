//! Configuration parsing for AUSH shell
//!
//! This module provides configuration parsing from .aushrc files.
//! Re-exports the config types from the daemon module for convenience.

pub mod banner;

// Re-export daemon config types for convenience
pub use crate::daemon::config::{
    BannerConfig, BannerShow, BannerStyle, CustomStatConfig, DaemonConfig,
};
