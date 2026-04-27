# AUSH Daemon Architecture

**Version:** 1.0
**Date:** January 2026
**Status:** Design Phase
**Goal:** Achieve ~0.4ms startup overhead via persistent daemon

## Table of Contents

1. [Overview](#overview)
2. [Motivation](#motivation)
3. [Architecture](#architecture)
4. [Protocol Specification](#protocol-specification)
5. [Session Management](#session-management)
6. [Security Model](#security-model)
7. [Crash Recovery](#crash-recovery)
8. [Performance Optimization](#performance-optimization)
9. [Implementation Plan](#implementation-plan)
10. [Testing Strategy](#testing-strategy)
11. [Risks and Mitigations](#risks-and-mitigations)

---

## Overview

AUSH daemon mode implements a client-server architecture where:
- **Daemon Process**: Long-running background server maintaining shell runtime state
- **Thin Client**: Lightweight binary (~200KB) that connects to daemon via Unix socket
- **Session Isolation**: Each client connection gets independent shell session
- **Zero-Copy IPC**: Unix domain sockets for sub-millisecond communication

### Target Performance

| Metric | Current | Daemon Mode | Improvement |
|--------|---------|-------------|-------------|
| Startup (cold) | 4.9ms | 4.9ms (daemon init) | - |
| Startup (warm) | 4.9ms | **0.4ms** (client connect) | **12x faster** |
| Binary size | 4.1MB | Client: 200KB, Daemon: 4.1MB | - |
| Memory | 0MB (ephemeral) | ~15MB (persistent daemon) | Trade-off |

---

## Motivation

### The Startup Problem

AUSH's excellent builtin performance (17-427x faster than traditional shells) is masked by startup overhead:

```bash
# Current: Pay startup cost every time
aush -c "ls"        # 4.9ms (startup dominates 109µs builtin)
aush -c "cat file"  # 4.9ms (startup dominates 10µs builtin)

# With daemon: Pay startup cost once
aushd start         # 4.9ms (daemon init, done once)
aush -c "ls"        # 0.4ms (just connection + 109µs builtin)
aush -c "cat file"  # 0.4ms (just connection + 10µs builtin)
```

### Use Cases

1. **Script Execution**: Scripts with 100+ commands amortize startup
2. **Build Systems**: CI/CD pipelines spawning many shell instances
3. **Interactive Workflows**: Developers opening 20-50 terminals per day
4. **Automated Testing**: Test suites creating shell processes per test

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────┐
│                     User Terminal                        │
└───────────────┬─────────────────────────────────────────┘
                │
                │ spawn
                ↓
┌─────────────────────────────────────────────────────────┐
│              aush (thin client binary)                   │
│  - Parse command-line args                               │
│  - Connect to daemon via Unix socket                     │
│  - Forward stdin/stdout/stderr                           │
│  - Handle signals (Ctrl-C, etc.)                         │
│  Size: ~200KB                                            │
└───────────────┬─────────────────────────────────────────┘
                │
                │ Unix socket: ~/.aush/daemon.sock
                ↓
┌─────────────────────────────────────────────────────────┐
│              aushd (daemon process)                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Socket Listener (accept() loop)                 │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                                │
│                         │ fork per client                │
│                         ↓                                │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Session Worker (per-client child process)      │   │
│  │  - Independent Runtime state                     │   │
│  │  - Independent Executor                          │   │
│  │  - Independent signal handlers                   │   │
│  │  - Communicates via inherited socket FD          │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  Shared (loaded once):                                   │
│  - Lexer/Parser code (~2MB .text)                       │
│  - Builtin implementations                               │
│  - Regex engines, completion databases                   │
│  Size: 4.1MB (in memory once)                           │
└─────────────────────────────────────────────────────────┘
```

### Process Model: Fork-per-Session

**Why Fork-Based Sessions?**

✅ **Chosen: Fork-per-session**
- Perfect isolation (separate address space)
- Separate working directory, process groups, signal handlers
- Worker crash doesn't affect daemon or other sessions
- Simple memory model (no complex locking)
- Copy-on-write minimizes overhead (~2-3MB per session, not 15MB)

---

## Protocol Specification

### Message Format

Length-prefixed binary messages:

```
┌────────────┬──────────────┬─────────────────────┐
│   Length   │  Message ID  │   Payload (JSON)    │
│  (4 bytes) │  (4 bytes)   │  (variable length)  │
└────────────┴──────────────┴─────────────────────┘
```

### Message Types

#### 1. Session Init (Client → Daemon)

```json
{
  "type": "session_init",
  "working_dir": "/Users/asher/project",
  "env": {
    "PATH": "/usr/local/bin:/usr/bin",
    "HOME": "/Users/asher"
  },
  "args": ["-c", "ls -la"],
  "stdin_mode": "inherit"
}
```

#### 2. Execution Result (Daemon → Client)

```json
{
  "type": "result",
  "exit_code": 0,
  "stdout_len": 1024,
  "stderr_len": 0
}
```

### File Descriptor Passing

Uses Unix `SCM_RIGHTS` to pass stdin/stdout/stderr FDs for zero-copy I/O.

---

## Security Model

### Unix Socket Permissions

**Location:** `$HOME/.aush/daemon.sock`

**Permissions:**
```bash
drwx------ 2 asher staff 64 .aush/         # 0700
-rw------- 1 asher staff  0 daemon.sock    # 0600
```

Only the user who started the daemon can connect.

### Connection Limits

```rust
const MAX_CONCURRENT_SESSIONS: usize = 100;
```

Prevents resource exhaustion from connection spam.

---

## Crash Recovery

### Automatic Daemon Restart

**Client-triggered:**
```rust
impl AUSHClient {
    fn execute_command(&mut self, cmd: &str) -> Result<i32> {
        match self.send_message(cmd) {
            Ok(_) => self.receive_result(),
            Err(BrokenPipe) => {
                log::warn!("Daemon crashed, restarting...");
                self.restart_daemon()?;
                self.reconnect()?;
                self.execute_command(cmd) // Retry
            }
        }
    }
}
```

### macOS Launchd Integration

```xml
<!-- ~/Library/LaunchAgents/com.aush.daemon.plist -->
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.aush.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/aushd</string>
        <string>start</string>
    </array>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

---

## Performance Optimization

### Target Breakdown

| Phase | Time | Cumulative |
|-------|------|-----------|
| Client startup | 0.05ms | 0.05ms |
| Socket connect | 0.10ms | 0.15ms |
| Message send | 0.05ms | 0.20ms |
| Daemon dispatch | 0.10ms | 0.30ms |
| Session fork | 0.05ms | 0.35ms |
| Command execute | 0.05ms | 0.40ms |

**Total: 0.40ms** (vs current 4.9ms = **12x improvement**)

### Client Binary Optimization

**Goal:** 200KB binary

```toml
[dependencies]
# Minimal deps for client:
serde_json = "1"
nix = { version = "0.29", features = ["socket"] }
```

### Zero-Copy I/O

Worker writes directly to client's stdout FD (terminal FD) → no intermediate buffers.

---

## Implementation Plan

### Phase 1: MVP Daemon (Week 1-2)

**Goal:** Basic daemon with `-c` flag support, prove <1ms startup

**Files to Create:**
```
src/daemon/
├── mod.rs           # Daemon main module
├── server.rs        # Unix socket server, accept loop
├── worker.rs        # Session worker (forked process)
├── protocol.rs      # Message framing, JSON payload
└── client.rs        # Thin client binary logic

src/bin/
├── aush.rs          # Thin client (rewrite)
└── aushd.rs         # Daemon server (new)
```

**Steps:**
1. Create `daemon/protocol.rs` - message framing, JSON serialization
2. Create `daemon/server.rs` - Unix socket listener, accept loop
3. Create `daemon/worker.rs` - fork session worker, execute commands
4. Create `daemon/client.rs` - connect to daemon, send command, receive result
5. Create `bin/aushd.rs` - daemon entry point (`aushd start/stop/status`)
6. Create `bin/aush.rs` - thin client
7. Test: `aushd start && hyperfine 'aush -c exit'` → verify <1ms

**Success Criteria:**
- `aush -c "echo test"` completes in <1ms (after daemon started)
- Multiple concurrent commands work correctly
- Daemon survives worker crashes
- Clean shutdown with `aushd stop`

### Phase 2: Interactive Mode (Week 3)

Support interactive REPL over daemon connection with signal handling.

### Phase 3: Optimization (Week 4)

Reduce startup from ~1ms to ~0.4ms through profiling and targeted optimizations.

### Phase 4: Crash Recovery (Week 5)

Auto-restart daemon, macOS Launchd / Linux systemd integration.

### Phase 5: Documentation & Polish (Week 6)

User guide, developer guide, benchmarks, migration guide.

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_message_framing() {
    let msg = SessionInit { ... };
    let bytes = encode_message(&msg).unwrap();
    let decoded = decode_message(&bytes).unwrap();
    assert_eq!(msg, decoded);
}

#[test]
fn test_session_isolation() {
    // Session 1 sets variable, Session 2 should not see it
}
```

### Integration Tests

```bash
# Startup performance (100 commands should average <1ms)
aushd start
hyperfine --warmup 10 --runs 100 'aush -c exit'

# Concurrent sessions (10 parallel clients)
for i in {1..10}; do aush -c "echo $i" & done

# Crash recovery
kill -9 $(pgrep aushd)
aush -c "echo test"  # Should auto-restart
```

---

## Risks and Mitigations

### Risk 1: Complexity vs. Benefit

**Mitigation:** Implement as optional feature with fallback to traditional process model.

### Risk 2: Subtle State Bugs

**Mitigation:** Extensive integration tests for session isolation, fuzzing.

### Risk 3: Platform Compatibility

**Mitigation:** Test on macOS, Linux, BSD. Use `nix` crate for portability.

### Risk 4: Performance Regression

**Mitigation:** Continuous benchmarking, abort if <2x improvement not achieved.

---

## Critical Files for Implementation

### 1. `src/daemon/protocol.rs` - **Protocol foundation**
All communication depends on message framing, serialization, FD passing.

### 2. `src/daemon/server.rs` - **Daemon core**
Main daemon loop, session registry, worker management.

### 3. `src/daemon/worker.rs` - **Session execution**
Where actual shell commands run, isolated per session.

### 4. `src/bin/aush.rs` - **Thin client**
User-facing binary, must be minimal (<200KB) and fast.

### 5. `src/executor/mod.rs` - **Pattern to follow**
Shows current execution model, must replicate in worker with full isolation.

---

## Success Metrics

### MVP Success (End of Phase 1)

- [ ] `aush -c "exit"` completes in <1ms (90th percentile)
- [ ] 100 concurrent commands complete successfully
- [ ] Daemon survives 10,000 connections without memory leak
- [ ] Client binary <500KB (goal: 200KB)

### Production Ready (End of Phase 4)

- [ ] Startup <0.5ms (90th percentile)
- [ ] Interactive mode works with Ctrl-C, Ctrl-D, job control
- [ ] Auto-restart on daemon crash <100ms
- [ ] Documentation complete

---

## References

- LSP Daemon Research: Language Server Protocol architecture
- Emacs Server: Original client-server shell pattern
- Unix Domain Socket Benchmarks: 2-3x faster than TCP localhost
- AUSH Performance Summary: Current startup profiling (4.9ms)

---

*Last updated: January 2026*
