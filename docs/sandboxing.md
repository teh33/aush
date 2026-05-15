---
title: "Sandboxing Design for AUSH"
tags: [sandboxing, security, architecture, agent-tools]
status: active
created: 2026-05-07
---

# Sandboxing Design for AUSH

**Author:** Goose (Goosetown orchestrator)  
**Context:** Discussion with aush author comparing aush, external shell behavior corpora, Goosetown, and imp  
**Goal:** Capture sandboxing architecture decisions for aush as the native shell for imp

---

## Why AUSH Needs Sandboxing

AUSH is the native shell for [imp](https://github.com/kfcafe/imp) — an extensible terminal-based coding agent. imp delegates work to child agents via [mana](https://github.com/kfcafe/mana) jobs. Those child agents need to execute shell commands safely on the local machine.

**Threat model:** Accident prevention, not malicious code execution. Agents run locally, in git worktrees, under user supervision.

---

## What We Are NOT Building

We are **not** building a VM-level sandbox or a bash simulation. Tools like
just-bash are useful reference points for virtual-shell ergonomics: in-memory
filesystems, WASM runtimes, URL allow-lists, and explicit security models. AUSH
chooses a different release contract: it is a real local shell for real
git/cargo/system workflows.

| Approach | Example | Use Case |
|----------|---------|----------|
| Virtual bash simulation | just-bash-style environments | Untrusted code, browser environments, no real filesystem needed |
| Real shell with guardrails | **aush** | Trusted local execution, real git/cargo/system tools |

AUSH's value is **real processes on the real system** — `git`, `cargo`, `rg`,
`find`, etc. Sandboxing adds guardrails without sacrificing that. Until those
guardrails are implemented and enforced, the public alpha security model is
simple: AUSH is not sandboxed by default and commands have the authority of the
current user.

---

## Sandboxing Architecture

### Layer 1: Effects and Risk (Already Exists)

AUSH already tracks command effects:

```rust
pub enum CommandEffect {
    ReadFile,
    WriteFile,
    DeleteFile,
    NetworkAccess,
    SpawnProcess,
    ModifyGitHistory,
}

pub enum RiskLevel {
    Low,
    Medium,
    High,
}
```

Each builtin declares its effects and default risk. The approval framework (`AUSH_APPROVAL_MODE`) prompts users for high-risk commands.

**What exists:**
- `CommandMetadata` with effects and risk
- `ApprovalMode` (Off / Medium / High)
- `CommandReceipt` with approval decisions
- Human-readable effect summaries

**What is missing:** Programmatic enforcement for agent use.

---

### Layer 2: SandboxConfig (To Add)

Extend `RunOptions` in `run_api.rs`:

```rust
pub struct RunOptions {
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub timeout: Option<u64>,
    pub json_output: bool,
    pub max_output_bytes: Option<usize>,
    pub sandbox: SandboxConfig, // NEW
}

pub struct SandboxConfig {
    pub filesystem: FsPolicy,
    pub network: NetworkPolicy,
    pub resources: ResourceLimits,
}

pub enum FsPolicy {
    AllowAll,
    AllowWritesTo(Vec<PathBuf>),
    ReadOnlyExcept(Vec<PathBuf>),
}

pub enum NetworkPolicy {
    Deny,
    AllowHosts(Vec<String>),
    AllowAll, // dangerous
}

pub struct ResourceLimits {
    pub max_commands: Option<usize>,
    pub max_loop_iterations: Option<usize>,
    pub max_call_depth: Option<usize>,
    pub max_processes: Option<usize>,
    pub max_memory_mb: Option<usize>,
}
```

---

### Layer 3: Enforcement Hooks (To Add)

#### 3.1 Path Validation

In `executor/commands.rs`, before executing any command:

```rust
fn validate_command_against_sandbox(
    command: &Command,
    sandbox: &SandboxConfig,
) -> Result<()> {
    // Check redirects
    for redirect in &command.redirections {
        validate_path(&redirect.path, &sandbox.filesystem)?;
    }
    
    // Check command effects against policy
    let effects = infer_effects(command)?;
    for effect in effects {
        match effect {
            CommandEffect::WriteFile => {
                ensure_filesystem_allows_write(&sandbox.filesystem)?;
            }
            CommandEffect::NetworkAccess => {
                ensure_network_allowed(&sandbox.network)?;
            }
            // ... etc
        }
    }
    
    Ok(())
}
```

#### 3.2 Network Interception

For `curl`, `wget`, `fetch` builtins:
- Check `NetworkPolicy` before execution
- If `Deny`, return "network access denied" error
- If `AllowHosts`, validate URL against allow-list
- External commands (`/usr/bin/curl`) harder to intercept — may need `LD_PRELOAD` or proxy env vars

#### 3.3 Resource Limits

In `executor/mod.rs`, track during execution:

```rust
pub struct Executor {
    // ... existing fields ...
    sandbox: Option<SandboxConfig>,
    command_count: usize,
    loop_iteration_count: HashMap<LoopId, usize>,
    call_depth: usize,
}
```

Check before:
- Each statement execution (`max_commands`)
- Each loop iteration (`max_loop_iterations`)
- Each function call (`max_call_depth`)

---

### Layer 4: macOS Considerations

macOS lacks Linux user namespaces. Options:

| Approach | Feasibility | Notes |
|----------|-------------|-------|
| `sandbox-exec` | Medium | Deprecated but functional; Apple uses internally |
| `ptrace` restrictions | Low | Complex, fragile, performance cost |
| Path validation in executor | **High** | Sufficient for accident prevention |
| `chroot` | Low | Requires root |

**Recommendation:** Rely on path validation + resource limits for macOS. Accept that perfect isolation requires Linux containers or just-bash.

---

## Integration with imp

imp configures sandbox per-subagent:

```rust
// In imp's shell tool
let result = aush::run(command, &RunOptions {
    cwd: Some(worktree_path),
    timeout: Some(300),
    max_output_bytes: Some(50 * 1024),
    sandbox: SandboxConfig {
        filesystem: FsPolicy::AllowWritesTo(vec![
            worktree_path.clone(),
            tmp_dir.clone(),
        ]),
        network: NetworkPolicy::Deny,
        resources: ResourceLimits {
            max_commands: Some(1000),
            max_loop_iterations: Some(10000),
            max_processes: Some(10),
            ..Default::default()
        },
    },
});
```

imp's `ask_agent` tool (spawning child agents) would pass the same sandbox config to child processes.

---

## Relationship to just-bash

If stronger isolation is needed later, integrate just-bash as a separate tool:

```rust
enum ShellBackend {
    Aush(aush::RunOptions),      // Trusted, real tools, guardrailed
    JustBash(just_bash::Config), // Untrusted, virtualized, fully isolated
}
```

Use aush for:
- Git operations
- Build/test (`cargo`, `npm`)
- File editing with real tools

Use just-bash for:
- Running untrusted code from the internet
- Browser environments
- Maximum isolation requirements

---

## Implementation Phases

### Phase 1: Path Validation (1-2 days)
- Add `SandboxConfig` to `RunOptions`
- Thread through to `Executor`
- Validate file paths in redirects and command args
- Block network commands unless allowed

### Phase 2: Resource Limits (1 day)
- Command count tracking
- Loop iteration limits
- Call depth limits
- Process spawn limits (where feasible)

### Phase 3: Network Policy (1 day)
- URL allow-list for `curl` builtin
- Header injection for credentials (like just-bash)
- Block/allow at builtin level

### Phase 4: macOS Hardening (if needed)
- `sandbox-exec` wrapper for external commands
- Or accept path-validation-only on macOS

---

## Open Questions

1. **External command sandboxing:** How to restrict `/usr/bin/curl` when called directly? `PATH` manipulation? `LD_PRELOAD`? Or only sandbox aush builtins?

2. **Process tree limits:** How to enforce `max_processes` when external commands fork? `setrlimit`? `cgroups` (Linux only)?

3. **Structured output compatibility:** Sandbox errors should emit as structured JSON when `json_output: true`.

4. **WASM backends:** Should aush optionally use WASM Python/JS like just-bash? Or leave that to just-bash integration?

---

## References

- [just-bash](https://github.com/vercel/just-bash) — Vercel's virtual bash environment
- External shell behavior corpora — useful for validating shell semantics without making a product-level compatibility promise
- [Goosetown](https://github.com/aaif/goosetown) — Multi-agent orchestration framework
- [imp](https://github.com/kfcafe/imp) — Terminal-based coding agent
- [mana](https://github.com/kfcafe/mana) — Task graph with verify gates

---

*This document was authored by Goose (Goosetown orchestrator) during a session comparing agent shell sandboxing approaches. It captures the design direction for adding programmatic sandbox enforcement to aush's existing effects and approval framework.*
