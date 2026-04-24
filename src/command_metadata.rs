//! Command metadata registry for inspectable AUSH builtins.
//!
//! This module is intentionally read-only substrate: it does not change command
//! execution. It gives later schema, receipt, approval, and agent-policy layers a
//! stable place to ask "what can this command do?" while preserving polished
//! terminal presentation.

use crate::effects::{render_effect_summary, CommandEffect, RiskLevel};

const LS_EFFECTS: &[CommandEffect] = &[CommandEffect::ReadFile];
const CAT_EFFECTS: &[CommandEffect] = &[CommandEffect::ReadFile];
const GREP_EFFECTS: &[CommandEffect] = &[CommandEffect::ReadFile];
const FIND_EFFECTS: &[CommandEffect] = &[CommandEffect::ReadFile];
const GIT_STATUS_EFFECTS: &[CommandEffect] = &[CommandEffect::ReadFile];
const RM_EFFECTS: &[CommandEffect] = &[CommandEffect::DeleteFile];
const MV_EFFECTS: &[CommandEffect] = &[CommandEffect::ReadFile, CommandEffect::WriteFile];
const CP_EFFECTS: &[CommandEffect] = &[CommandEffect::ReadFile, CommandEffect::WriteFile];
const WRITE_FILE_EFFECTS: &[CommandEffect] = &[CommandEffect::WriteFile];
const FETCH_EFFECTS: &[CommandEffect] = &[CommandEffect::NetworkAccess];

/// Static metadata for an AUSH command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandMetadata {
    /// Command name as typed at the prompt.
    pub name: &'static str,
    /// One-line human summary for docs, approvals, and command inspectors.
    pub summary: &'static str,
    /// Declared high-level effects for this command.
    pub effects: &'static [CommandEffect],
    /// Default command risk before arguments/path context is considered.
    pub risk: RiskLevel,
    /// Whether this command currently exposes structured JSON output.
    pub supports_json: bool,
    /// Whether this command currently has a native preview/dry-run mode.
    pub supports_preview: bool,
}

impl CommandMetadata {
    /// Render a concise human summary suitable for terminal inspection.
    ///
    /// Uses polished labels from `CommandEffect`; raw machine IDs are reserved
    /// for JSON/schema/debug output.
    pub fn render_human_summary(&self) -> String {
        let mut output = String::new();
        output.push_str(self.name);
        output.push_str(" — ");
        output.push_str(self.summary);
        output.push('\n');
        output.push_str(&format!("Risk: {}\n", self.risk.label()));
        output.push_str(&render_effect_summary(self.effects));
        output
    }

    /// Stable machine effect IDs for future schema/receipt output.
    pub fn effect_ids(&self) -> Vec<&'static str> {
        self.effects.iter().map(|effect| effect.id()).collect()
    }

    /// Polished effect labels for terminal output.
    pub fn effect_labels(&self) -> Vec<&'static str> {
        self.effects.iter().map(|effect| effect.label()).collect()
    }
}

const COMMAND_METADATA: &[CommandMetadata] = &[
    CommandMetadata {
        name: "ls",
        summary: "List directory contents",
        effects: LS_EFFECTS,
        risk: RiskLevel::Low,
        supports_json: true,
        supports_preview: false,
    },
    CommandMetadata {
        name: "cat",
        summary: "Read and print file contents",
        effects: CAT_EFFECTS,
        risk: RiskLevel::Low,
        supports_json: false,
        supports_preview: false,
    },
    CommandMetadata {
        name: "grep",
        summary: "Search file contents for matching text",
        effects: GREP_EFFECTS,
        risk: RiskLevel::Low,
        supports_json: true,
        supports_preview: false,
    },
    CommandMetadata {
        name: "find",
        summary: "Find files and directories by criteria",
        effects: FIND_EFFECTS,
        risk: RiskLevel::Low,
        supports_json: true,
        supports_preview: false,
    },
    CommandMetadata {
        name: "git_status",
        summary: "Inspect repository status",
        effects: GIT_STATUS_EFFECTS,
        risk: RiskLevel::Low,
        supports_json: true,
        supports_preview: false,
    },
    CommandMetadata {
        name: "rm",
        summary: "Remove files or directories",
        effects: RM_EFFECTS,
        risk: RiskLevel::High,
        supports_json: false,
        supports_preview: false,
    },
    CommandMetadata {
        name: "mv",
        summary: "Move or rename files and directories",
        effects: MV_EFFECTS,
        risk: RiskLevel::Medium,
        supports_json: false,
        supports_preview: false,
    },
    CommandMetadata {
        name: "cp",
        summary: "Copy files and directories",
        effects: CP_EFFECTS,
        risk: RiskLevel::Medium,
        supports_json: false,
        supports_preview: false,
    },
    CommandMetadata {
        name: "write_file",
        summary: "Write content to a file",
        effects: WRITE_FILE_EFFECTS,
        risk: RiskLevel::Medium,
        supports_json: false,
        supports_preview: false,
    },
    CommandMetadata {
        name: "fetch",
        summary: "Fetch a URL over the network",
        effects: FETCH_EFFECTS,
        risk: RiskLevel::Medium,
        supports_json: true,
        supports_preview: false,
    },
];

/// Return metadata for all registered commands.
pub const fn all_command_metadata() -> &'static [CommandMetadata] {
    COMMAND_METADATA
}

/// Look up metadata for a command by exact name.
pub fn metadata_for_command(name: &str) -> Option<&'static CommandMetadata> {
    COMMAND_METADATA
        .iter()
        .find(|metadata| metadata.name == name)
}

#[cfg(test)]
mod tests {
    use super::{all_command_metadata, metadata_for_command};
    use crate::effects::{CommandEffect, RiskLevel};

    #[test]
    fn command_metadata_rm_declares_delete_file_high_risk() {
        let metadata = metadata_for_command("rm").expect("rm metadata should exist");

        assert_eq!(metadata.risk, RiskLevel::High);
        assert!(metadata.effects.contains(&CommandEffect::DeleteFile));
        assert_eq!(metadata.effect_ids(), vec!["delete_file"]);
        assert_eq!(metadata.effect_labels(), vec!["Delete files"]);
    }

    #[test]
    fn command_metadata_fetch_declares_network_access() {
        let metadata = metadata_for_command("fetch").expect("fetch metadata should exist");

        assert_eq!(metadata.risk, RiskLevel::Medium);
        assert!(metadata.supports_json);
        assert!(metadata.effects.contains(&CommandEffect::NetworkAccess));
    }

    #[test]
    fn command_metadata_renders_pretty_effect_labels() {
        let metadata = metadata_for_command("ls").expect("ls metadata should exist");
        let summary = metadata.render_human_summary();

        assert!(summary.contains("ls — List directory contents"));
        assert!(summary.contains("Low risk"));
        assert!(summary.contains("Read files"));
        assert!(!summary.contains("read_file"));
    }

    #[test]
    fn command_metadata_unknown_commands_return_none() {
        assert!(metadata_for_command("definitely_not_a_builtin").is_none());
    }

    #[test]
    fn command_metadata_registry_exposes_starter_set() {
        let names: Vec<&str> = all_command_metadata()
            .iter()
            .map(|metadata| metadata.name)
            .collect();

        assert!(names.contains(&"ls"));
        assert!(names.contains(&"cat"));
        assert!(names.contains(&"grep"));
        assert!(names.contains(&"find"));
        assert!(names.contains(&"git_status"));
        assert!(names.contains(&"rm"));
        assert!(names.contains(&"mv"));
        assert!(names.contains(&"cp"));
        assert!(names.contains(&"write_file"));
        assert!(names.contains(&"fetch"));
    }
}
