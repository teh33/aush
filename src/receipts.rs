//! Command receipts and JSONL ledger helpers.
//!
//! Receipts are the audit substrate for future semantic history, approval
//! prompts, agent supervision, and timelines. This module only defines the data
//! model and append/render helpers; it does not record executions automatically.

use crate::command_metadata::metadata_for_command;
use crate::effects::{render_effect_summary, CommandEffect, RiskLevel};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Serializable receipt for one command execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandReceipt {
    /// Full command string as executed or planned.
    pub command: String,
    /// Best-effort command name used for metadata lookup.
    pub command_name: String,
    /// Working directory where the command ran.
    pub cwd: PathBuf,
    /// Execution start timestamp.
    pub started_at: DateTime<Utc>,
    /// Execution finish timestamp.
    pub finished_at: DateTime<Utc>,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
    /// Process/shell exit code.
    pub exit_code: i32,
    /// Declared effects from command metadata.
    pub effects: Vec<CommandEffect>,
    /// Declared risk from command metadata or conservative fallback.
    pub risk: RiskLevel,
}

impl CommandReceipt {
    /// Build a receipt and infer known effects/risk from command metadata.
    pub fn new(
        command: impl Into<String>,
        cwd: impl Into<PathBuf>,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        exit_code: i32,
    ) -> Self {
        let command = command.into();
        let command_name = first_command_word(&command).unwrap_or_default();
        let metadata = metadata_for_command(&command_name);
        let duration_ms = duration_ms_between(started_at, finished_at);
        let effects = metadata
            .map(|metadata| metadata.effects.to_vec())
            .unwrap_or_default();
        let risk = metadata
            .map(|metadata| metadata.risk)
            .unwrap_or(RiskLevel::Medium);

        Self {
            command,
            command_name,
            cwd: cwd.into(),
            started_at,
            finished_at,
            duration_ms,
            exit_code,
            effects,
            risk,
        }
    }

    /// Build a receipt with explicit effects/risk, useful for tests and future
    /// execution paths that have richer argument-specific metadata.
    pub fn with_effects(
        command: impl Into<String>,
        command_name: impl Into<String>,
        cwd: impl Into<PathBuf>,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        exit_code: i32,
        effects: Vec<CommandEffect>,
        risk: RiskLevel,
    ) -> Self {
        Self {
            command: command.into(),
            command_name: command_name.into(),
            cwd: cwd.into(),
            started_at,
            finished_at,
            duration_ms: duration_ms_between(started_at, finished_at),
            exit_code,
            effects,
            risk,
        }
    }

    /// Render a polished terminal receipt. Raw machine effect IDs are reserved
    /// for JSON/debug output and are intentionally not shown here.
    pub fn render_human_receipt(&self) -> String {
        let mut output = String::new();
        output.push_str("Command receipt\n");
        output.push_str(&format!("  Command: {}\n", self.command));
        output.push_str(&format!("  Directory: {}\n", self.cwd.display()));
        output.push_str(&format!("  Exit: {}\n", self.exit_code));
        output.push_str(&format!("  Duration: {} ms\n", self.duration_ms));
        output.push_str(&format!("  Risk: {}\n", self.risk.label()));
        output.push_str(&render_effect_summary(&self.effects));
        output
    }
}

/// Append a receipt as one JSON line, creating parent directories as needed.
pub fn append_receipt_jsonl(path: impl AsRef<Path>, receipt: &CommandReceipt) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create receipt directory {}", parent.display()))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open receipt ledger {}", path.display()))?;

    let json = serde_json::to_string(receipt).context("Failed to serialize command receipt")?;
    writeln!(file, "{json}").context("Failed to append command receipt")?;
    Ok(())
}

/// Convenience wrapper for callers that prefer a free function.
pub fn render_human_receipt(receipt: &CommandReceipt) -> String {
    receipt.render_human_receipt()
}

fn first_command_word(command: &str) -> Option<String> {
    command
        .split_whitespace()
        .next()
        .map(|word| word.trim_matches(|ch| ch == ';' || ch == '(' || ch == ')'))
        .filter(|word| !word.is_empty())
        .map(ToOwned::to_owned)
}

fn duration_ms_between(started_at: DateTime<Utc>, finished_at: DateTime<Utc>) -> u64 {
    let millis = finished_at
        .signed_duration_since(started_at)
        .num_milliseconds();
    millis.max(0) as u64
}

#[cfg(test)]
mod tests {
    use super::{append_receipt_jsonl, render_human_receipt, CommandReceipt};
    use crate::effects::{CommandEffect, RiskLevel};
    use chrono::{TimeZone, Utc};
    use tempfile::tempdir;

    fn fixed_times() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
        (
            Utc.with_ymd_and_hms(2026, 4, 24, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 4, 24, 10, 0, 1).unwrap(),
        )
    }

    #[test]
    fn receipts_serialize_effects_as_machine_ids() {
        let (started_at, finished_at) = fixed_times();
        let receipt = CommandReceipt::with_effects(
            "rm old.log",
            "rm",
            "/repo",
            started_at,
            finished_at,
            0,
            vec![CommandEffect::DeleteFile],
            RiskLevel::High,
        );

        let json = serde_json::to_string(&receipt).unwrap();
        assert!(json.contains("delete_file"));
        assert!(json.contains("high"));
    }

    #[test]
    fn human_receipts_render_pretty_effect_labels() {
        let (started_at, finished_at) = fixed_times();
        let receipt = CommandReceipt::with_effects(
            "rm old.log",
            "rm",
            "/repo",
            started_at,
            finished_at,
            0,
            vec![CommandEffect::DeleteFile],
            RiskLevel::High,
        );

        let rendered = render_human_receipt(&receipt);
        assert!(rendered.contains("Command receipt"));
        assert!(rendered.contains("High risk"));
        assert!(rendered.contains("Delete files"));
        assert!(!rendered.contains("delete_file"));
    }

    #[test]
    fn append_receipt_jsonl_writes_deserializable_receipt() {
        let (started_at, finished_at) = fixed_times();
        let receipt = CommandReceipt::new("fetch https://example.com", "/repo", started_at, finished_at, 0);
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("receipts").join("ledger.jsonl");

        append_receipt_jsonl(&ledger_path, &receipt).unwrap();

        let contents = std::fs::read_to_string(ledger_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 1);

        let decoded: CommandReceipt = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(decoded.command, "fetch https://example.com");
        assert_eq!(decoded.effects, vec![CommandEffect::NetworkAccess]);
    }

    #[test]
    fn receipts_infer_rm_metadata() {
        let (started_at, finished_at) = fixed_times();
        let receipt = CommandReceipt::new("rm old.log", "/repo", started_at, finished_at, 0);

        assert_eq!(receipt.command_name, "rm");
        assert_eq!(receipt.risk, RiskLevel::High);
        assert_eq!(receipt.effects, vec![CommandEffect::DeleteFile]);
        assert_eq!(receipt.duration_ms, 1000);
    }

    #[test]
    fn receipts_unknown_commands_use_conservative_empty_metadata() {
        let (started_at, finished_at) = fixed_times();
        let receipt = CommandReceipt::new("custom-tool --flag", "/repo", started_at, finished_at, 0);

        assert_eq!(receipt.command_name, "custom-tool");
        assert_eq!(receipt.risk, RiskLevel::Medium);
        assert!(receipt.effects.is_empty());
    }
}
