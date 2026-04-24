//! Command effect metadata and human-facing presentation helpers.
//!
//! Effects are intentionally split into two surfaces:
//! - stable machine IDs such as `delete_file` for JSON, schemas, receipts, and policy
//! - polished labels such as `Delete files` for terminal UI
//!
//! Do not print raw enum/debug names in interactive output.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable machine-readable effect identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandEffect {
    ReadFile,
    WriteFile,
    DeleteFile,
    NetworkAccess,
    SpawnProcess,
    ModifyGitHistory,
}

impl CommandEffect {
    /// Stable snake_case identifier for schemas, JSON receipts, and policy files.
    pub const fn id(self) -> &'static str {
        match self {
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::DeleteFile => "delete_file",
            Self::NetworkAccess => "network_access",
            Self::SpawnProcess => "spawn_process",
            Self::ModifyGitHistory => "modify_git_history",
        }
    }

    /// Polished terminal label. Prefer this over `id()` for human UI.
    pub const fn label(self) -> &'static str {
        match self {
            Self::ReadFile => "Read files",
            Self::WriteFile => "Write files",
            Self::DeleteFile => "Delete files",
            Self::NetworkAccess => "Access the network",
            Self::SpawnProcess => "Start processes",
            Self::ModifyGitHistory => "Modify Git history",
        }
    }

    /// Short human description suitable for approval prompts and receipts.
    pub const fn description(self) -> &'static str {
        match self {
            Self::ReadFile => "Reads file contents or metadata.",
            Self::WriteFile => "Creates or changes files on disk.",
            Self::DeleteFile => "Removes files or directories from disk.",
            Self::NetworkAccess => "Connects to a network service.",
            Self::SpawnProcess => "Starts another program or subprocess.",
            Self::ModifyGitHistory => "Changes commits, refs, or repository history.",
        }
    }

    /// Default risk for the effect before command-specific context is known.
    pub const fn risk(self) -> RiskLevel {
        match self {
            Self::ReadFile => RiskLevel::Low,
            Self::SpawnProcess => RiskLevel::Medium,
            Self::WriteFile | Self::NetworkAccess => RiskLevel::Medium,
            Self::DeleteFile | Self::ModifyGitHistory => RiskLevel::High,
        }
    }

    /// Full presentation metadata for terminal UI.
    pub const fn presentation(self) -> EffectPresentation {
        EffectPresentation {
            id: self.id(),
            label: self.label(),
            description: self.description(),
            risk: self.risk(),
        }
    }
}

impl fmt::Display for CommandEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Human-facing risk level for prompts, receipts, and policy summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Low => "Low risk",
            Self::Medium => "Medium risk",
            Self::High => "High risk",
        }
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Display-ready metadata for one command effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectPresentation {
    /// Machine ID retained for details/debug views and links back to schemas.
    pub id: &'static str,
    /// Polished terminal label.
    pub label: &'static str,
    /// Short terminal-safe explanation.
    pub description: &'static str,
    /// Default risk for this effect.
    pub risk: RiskLevel,
}

/// Render an approval/receipt-friendly effect summary.
///
/// The output deliberately uses labels, not raw machine IDs.
pub fn render_effect_summary(effects: &[CommandEffect]) -> String {
    if effects.is_empty() {
        return "No declared effects.".to_string();
    }

    let max_risk = effects
        .iter()
        .map(|effect| effect.risk())
        .max()
        .unwrap_or(RiskLevel::Low);

    let mut output = String::new();
    output.push_str("This command may:\n\n");
    output.push_str(&format!("  {}\n", max_risk.label()));

    for effect in effects {
        output.push_str(&format!(
            "  • {}: {}\n",
            effect.label(),
            effect.description()
        ));
    }

    output.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::{render_effect_summary, CommandEffect, RiskLevel};

    #[test]
    fn effects_serialize_to_machine_ids() {
        let encoded = serde_json::to_string(&CommandEffect::DeleteFile).unwrap();
        assert_eq!(encoded, "\"delete_file\"");

        let decoded: CommandEffect = serde_json::from_str("\"network_access\"").unwrap();
        assert_eq!(decoded, CommandEffect::NetworkAccess);
    }

    #[test]
    fn effects_have_polished_human_labels() {
        assert_eq!(CommandEffect::DeleteFile.id(), "delete_file");
        assert_eq!(CommandEffect::DeleteFile.label(), "Delete files");
        assert_eq!(CommandEffect::NetworkAccess.label(), "Access the network");
        assert_eq!(
            CommandEffect::ModifyGitHistory.label(),
            "Modify Git history"
        );
        assert_eq!(CommandEffect::DeleteFile.to_string(), "Delete files");
    }

    #[test]
    fn effects_expose_default_risk_levels() {
        assert_eq!(CommandEffect::ReadFile.risk(), RiskLevel::Low);
        assert_eq!(CommandEffect::WriteFile.risk(), RiskLevel::Medium);
        assert_eq!(CommandEffect::DeleteFile.risk(), RiskLevel::High);
        assert_eq!(RiskLevel::High.id(), "high");
        assert_eq!(RiskLevel::High.label(), "High risk");
    }

    #[test]
    fn effect_summary_is_human_readable() {
        let summary =
            render_effect_summary(&[CommandEffect::DeleteFile, CommandEffect::NetworkAccess]);

        assert!(summary.contains("High risk"));
        assert!(summary.contains("Delete files"));
        assert!(summary.contains("Access the network"));
        assert!(!summary.contains("delete_file"));
        assert!(!summary.contains("network_access"));
    }
}
