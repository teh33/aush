//! Banner and custom stats configuration parsing
//!
//! This module handles parsing banner and custom stat configuration from .aushrc
//! with legacy .rushrc compatibility:
//!
//! ## Banner Configuration
//! - `AUSH_BANNER_STYLE` / `RUSH_BANNER_STYLE` - block, line, minimal, none
//! - `AUSH_BANNER_COLOR` / `RUSH_BANNER_COLOR` - cyan, green, yellow, magenta, blue, white, none
//! - `AUSH_BANNER_SHOW` / `RUSH_BANNER_SHOW` - always, first, never
//! - `AUSH_BANNER_STATS` / `RUSH_BANNER_STATS` - space-separated stat names to display
//!
//! ## Custom Stats Configuration
//! - `AUSH_STAT_<name>` / `RUSH_STAT_<name>` - Shell command to execute
//! - `AUSH_STAT_<name>_INTERVAL` / `RUSH_STAT_<name>_INTERVAL` - Refresh interval (default 30s)
//! - `AUSH_STAT_<name>_TIMEOUT` / `RUSH_STAT_<name>_TIMEOUT` - Command timeout (default 2s)

pub use crate::daemon::config::{
    BannerConfig, BannerShow, BannerStyle, CustomStatConfig, DaemonConfig,
};

/// Parse banner configuration from .aushrc or legacy .rushrc content
pub fn parse_banner_config(content: &str) -> BannerConfig {
    DaemonConfig::parse(content).banner
}

/// Parse custom stats from .aushrc or legacy .rushrc content
pub fn parse_custom_stats(content: &str) -> Vec<CustomStatConfig> {
    DaemonConfig::parse(content).custom_stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // =========================================================================
    // Banner Style Tests
    // =========================================================================

    #[test]
    fn test_banner_style_block() {
        let content = "RUSH_BANNER_STYLE=block";
        let config = parse_banner_config(content);
        assert_eq!(config.style, BannerStyle::Block);
    }

    #[test]
    fn test_banner_style_line() {
        let content = "RUSH_BANNER_STYLE=line";
        let config = parse_banner_config(content);
        assert_eq!(config.style, BannerStyle::Line);
    }

    #[test]
    fn test_banner_style_minimal() {
        let content = "RUSH_BANNER_STYLE=minimal";
        let config = parse_banner_config(content);
        assert_eq!(config.style, BannerStyle::Minimal);
    }

    #[test]
    fn test_banner_style_none() {
        let content = "RUSH_BANNER_STYLE=none";
        let config = parse_banner_config(content);
        assert_eq!(config.style, BannerStyle::None);
    }

    #[test]
    fn test_banner_style_case_insensitive() {
        let content = "RUSH_BANNER_STYLE=MINIMAL";
        let config = parse_banner_config(content);
        assert_eq!(config.style, BannerStyle::Minimal);
    }

    #[test]
    fn test_banner_style_quoted() {
        let content = r#"RUSH_BANNER_STYLE="line""#;
        let config = parse_banner_config(content);
        assert_eq!(config.style, BannerStyle::Line);
    }

    #[test]
    fn test_banner_style_default() {
        let content = "";
        let config = parse_banner_config(content);
        assert_eq!(config.style, BannerStyle::Block);
    }

    #[test]
    fn test_banner_style_invalid_defaults_to_block() {
        let content = "RUSH_BANNER_STYLE=invalid";
        let config = parse_banner_config(content);
        assert_eq!(config.style, BannerStyle::Block);
    }

    // =========================================================================
    // Banner Color Tests
    // =========================================================================

    #[test]
    fn test_banner_color() {
        let content = "RUSH_BANNER_COLOR=green";
        let config = parse_banner_config(content);
        assert_eq!(config.color, "green");
    }

    #[test]
    fn test_banner_color_quoted() {
        let content = r#"RUSH_BANNER_COLOR="magenta""#;
        let config = parse_banner_config(content);
        assert_eq!(config.color, "magenta");
    }

    #[test]
    fn test_banner_color_default() {
        let content = "";
        let config = parse_banner_config(content);
        assert!(config.color.is_empty());
    }

    // =========================================================================
    // Banner Show Tests
    // =========================================================================

    #[test]
    fn test_banner_show_always() {
        let content = "RUSH_BANNER_SHOW=always";
        let config = parse_banner_config(content);
        assert_eq!(config.show, BannerShow::Always);
    }

    #[test]
    fn test_banner_show_first() {
        let content = "RUSH_BANNER_SHOW=first";
        let config = parse_banner_config(content);
        assert_eq!(config.show, BannerShow::First);
    }

    #[test]
    fn test_banner_show_never() {
        let content = "RUSH_BANNER_SHOW=never";
        let config = parse_banner_config(content);
        assert_eq!(config.show, BannerShow::Never);
    }

    #[test]
    fn test_banner_show_case_insensitive() {
        let content = "RUSH_BANNER_SHOW=FIRST";
        let config = parse_banner_config(content);
        assert_eq!(config.show, BannerShow::First);
    }

    #[test]
    fn test_banner_show_default() {
        let content = "";
        let config = parse_banner_config(content);
        assert_eq!(config.show, BannerShow::Always);
    }

    // =========================================================================
    // Banner Stats Tests
    // =========================================================================

    #[test]
    fn test_banner_stats_single() {
        let content = "RUSH_BANNER_STATS=uptime";
        let config = parse_banner_config(content);
        assert_eq!(config.stats, vec!["uptime"]);
    }

    #[test]
    fn test_banner_stats_multiple() {
        let content = "RUSH_BANNER_STATS=host uptime memory";
        let config = parse_banner_config(content);
        assert_eq!(config.stats, vec!["host", "uptime", "memory"]);
    }

    #[test]
    fn test_banner_stats_quoted() {
        let content = r#"RUSH_BANNER_STATS="host uptime memory""#;
        let config = parse_banner_config(content);
        assert_eq!(config.stats, vec!["host", "uptime", "memory"]);
    }

    #[test]
    fn test_banner_stats_default_empty() {
        let content = "";
        let config = parse_banner_config(content);
        assert!(config.stats.is_empty());
    }

    // =========================================================================
    // Custom Stats Tests
    // =========================================================================

    #[test]
    fn test_custom_stat_basic() {
        let content = r#"RUSH_STAT_weather="curl -s wttr.in""#;
        let stats = parse_custom_stats(content);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "weather");
        assert_eq!(stats[0].command, "curl -s wttr.in");
    }

    #[test]
    fn test_custom_stat_with_interval() {
        let content = r#"
RUSH_STAT_weather="curl -s wttr.in"
RUSH_STAT_weather_INTERVAL=300
"#;
        let stats = parse_custom_stats(content);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "weather");
        assert_eq!(stats[0].interval, Duration::from_secs(300));
    }

    #[test]
    fn test_custom_stat_with_timeout() {
        let content = r#"
RUSH_STAT_weather="curl -s wttr.in"
RUSH_STAT_weather_TIMEOUT=5
"#;
        let stats = parse_custom_stats(content);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "weather");
        assert_eq!(stats[0].timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_custom_stat_full_config() {
        let content = r#"
RUSH_STAT_weather="curl -s wttr.in"
RUSH_STAT_weather_INTERVAL=300
RUSH_STAT_weather_TIMEOUT=5
"#;
        let stats = parse_custom_stats(content);

        assert_eq!(stats.len(), 1);
        let weather = &stats[0];
        assert_eq!(weather.name, "weather");
        assert_eq!(weather.command, "curl -s wttr.in");
        assert_eq!(weather.interval, Duration::from_secs(300));
        assert_eq!(weather.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_custom_stat_defaults() {
        let content = r#"RUSH_STAT_test="echo hello""#;
        let stats = parse_custom_stats(content);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].interval, Duration::from_secs(30)); // default
        assert_eq!(stats[0].timeout, Duration::from_secs(2)); // default
    }

    #[test]
    fn test_multiple_custom_stats() {
        let content = r#"
RUSH_STAT_weather="curl -s wttr.in"
RUSH_STAT_todos="wc -l < ~/todo.txt"
RUSH_STAT_disk="df -h / | tail -1"
"#;
        let stats = parse_custom_stats(content);

        assert_eq!(stats.len(), 3);
        // Stats are sorted by name
        assert_eq!(stats[0].name, "disk");
        assert_eq!(stats[1].name, "todos");
        assert_eq!(stats[2].name, "weather");
    }

    #[test]
    fn test_custom_stat_name_lowercase() {
        let content = r#"RUSH_STAT_WEATHER="curl -s wttr.in""#;
        let stats = parse_custom_stats(content);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "weather"); // name is lowercased
    }

    // =========================================================================
    // Full Config Tests
    // =========================================================================

    #[test]
    fn test_full_config() {
        let content = r#"
# AUSH banner configuration
RUSH_BANNER_STYLE="minimal"
RUSH_BANNER_COLOR="green"
RUSH_BANNER_SHOW="first"
RUSH_BANNER_STATS="host uptime memory weather"

# Custom stats
RUSH_STAT_weather="curl -s wttr.in"
RUSH_STAT_weather_INTERVAL=300
RUSH_STAT_weather_TIMEOUT=5
"#;
        let banner = parse_banner_config(content);
        let stats = parse_custom_stats(content);

        assert_eq!(banner.style, BannerStyle::Minimal);
        assert_eq!(banner.color, "green");
        assert_eq!(banner.show, BannerShow::First);
        assert_eq!(banner.stats, vec!["host", "uptime", "memory", "weather"]);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "weather");
    }

    #[test]
    fn test_config_with_export() {
        let content = r#"
export RUSH_BANNER_STYLE="line"
export RUSH_STAT_test="echo test"
"#;
        let banner = parse_banner_config(content);
        let stats = parse_custom_stats(content);

        assert_eq!(banner.style, BannerStyle::Line);
        assert_eq!(stats.len(), 1);
    }

    #[test]
    fn test_config_with_comments() {
        let content = r#"
# This is a comment
RUSH_BANNER_STYLE=minimal
# Another comment
RUSH_STAT_test="echo test"
"#;
        let banner = parse_banner_config(content);
        let stats = parse_custom_stats(content);

        assert_eq!(banner.style, BannerStyle::Minimal);
        assert_eq!(stats.len(), 1);
    }

    #[test]
    fn test_config_empty() {
        let content = "";
        let banner = parse_banner_config(content);
        let stats = parse_custom_stats(content);

        // Defaults
        assert_eq!(banner.style, BannerStyle::Block);
        assert_eq!(banner.show, BannerShow::Always);
        assert!(banner.stats.is_empty());
        assert!(stats.is_empty());
    }
}
