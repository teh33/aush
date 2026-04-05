# Rush Compatibility Roadmap

_Date: 2026-04-04_

## Goal

Bring Rush built-ins to **full GNU-coreutils/GNU-grep/find style compatibility** for the commands most important to AI agents and common shell scripts, while preserving Rush-native advantages like structured output and in-process performance.

## Target compatibility baseline

- **Primary target:** GNU behavior/flags on Linux
- **Secondary target:** POSIX shell semantics where GNU extends POSIX
- **Later:** BSD/macOS compatibility shims or mode flags where needed

## Scope for this roadmap

### AI-agent-critical
- `grep`
- `find`
- `ls`
- `sort`
- `head`
- `tail`
- `wc`
- `cat`
- `stat`
- `readlink`
- `env`
- `date`

### Top file/script commands
- `cp`
- `mv`
- `rm`
- `mkdir`

---

## Priority tiers

### Tier 0 — correctness and dangerous incompatibilities

These are the highest priority because they can break scripts, agents, and verification gates even when a flag appears to exist.

1. **`grep -q` / quiet semantics**
   - Required for script probes and `mana` verify gates
   - Ensure silent success/failure semantics are correct
2. **`tail -f` compatibility**
   - Either implement true follow mode or fail clearly
   - Current parse-and-ignore behavior is incompatible
3. **`rm` recursive prompt compatibility**
   - Align with GNU semantics; do not add nonstandard prompts by default
   - Preserve safety through optional Rush-specific modes, not changed defaults
4. **`readlink -m` semantics**
   - Must canonicalize missing-path components correctly
5. **Help text / implementation alignment**
   - No flag should be documented unless behavior is implemented

### Tier 1 — AI-agent-critical parity surface

6. **`grep` parity expansion**
   - Add: `-E`, `-F`, `-w`, `-x`, `-l`, `-L`, `-c`, `-m`, `-o`
   - Later: include/exclude patterns, `-Z`, `-z`, binary modes
7. **`find` parity expansion**
   - Add: `-print0`, `-delete`, `-perm`, `-user`, `-group`, `-newer`, `-regex`, `-prune`, `-exec ... +`
8. **`ls` parity expansion**
   - Add: `-R`, `-t`, `-S`, `-r`, long options, color modes
9. **Compatibility regression suite for agent-critical commands**
   - Golden tests for GNU-compatible exit codes, stdout/stderr, and edge behavior

### Tier 2 — top command parity

10. **`cp` parity expansion**
    - Add: `-a`, `-i`, `-L/-P/-H`, `-u`, `-T`, `--parents`
11. **`mv` parity expansion**
    - Add: `-i`, `-u`, `-T`, backup behavior as needed
12. **`rm` parity expansion**
    - Add: `-I`, `--interactive=...`, `--preserve-root`, `--no-preserve-root`, `--one-file-system`
13. **`mkdir` parity expansion**
    - Add: `-m`, `-v`

### Tier 3 — supporting text utils

14. **`sort` parity expansion**
    - Add: `-f`, `-b`, `-s`, `-g`, `-h`, `-V`, `-M`, `-o`, `-z`
15. **`head` / `tail` parity expansion**
    - Add: `-q`, `-v`, `-z`; for tail also `-F`, `--retry`, `--pid`, `-s`
16. **`wc` parity expansion**
    - Add: `-L`, `--files0-from`
17. **`cat` parity expansion**
    - Add: `-b`, `-s`, `-A`, `-e`, `-E`, `-t`, `-T`, `-v`

### Tier 4 — metadata / environment utils

18. **`stat` parity expansion**
    - Add more complete format support and filesystem-stat behavior
19. **`date` parity expansion**
    - Expand input grammar and common GNU flags
20. **`env` parity expansion**
    - Add: `-0`, `-C`, `-S`

---

## Recommended implementation order

### Phase 1 — stop breaking scripts and agents
- `grep -q`
- `tail -f`
- `rm` recursive behavior alignment
- `readlink -m`
- compatibility test harness skeleton

### Phase 2 — make Rush viable for common agent loops
- `grep` full common-flag pass
- `find` common-flag pass
- `ls` common-flag pass
- regression fixtures against GNU outputs

### Phase 3 — make Rush viable as a daily shell for scripts
- `cp`, `mv`, `rm`, `mkdir`
- `sort`, `head`, `tail`, `wc`, `cat`

### Phase 4 — close the long tail
- `stat`, `date`, `env`
- documentation sync
- compatibility dashboard / scorecard

---

## Work breakdown proposals

### Workstream A — behavior correctness
- Fix flags that currently lie or diverge dangerously
- Add integration tests first, then implementation changes

### Workstream B — agent-critical tools
- `grep`, `find`, `ls`
- prioritize flags used by agents, CI, and verify commands

### Workstream C — file mutation commands
- `cp`, `mv`, `rm`, `mkdir`
- verify overwrite, recursion, symlink, and prompt semantics carefully

### Workstream D — text processing utilities
- `sort`, `head`, `tail`, `wc`, `cat`

### Workstream E — metadata / environment
- `stat`, `readlink`, `date`, `env`

### Workstream F — compatibility harness
- GNU comparison tests where host tooling exists
- Rush-only deterministic fixtures for CI portability
- exit code, stdout, stderr, and edge-case assertions

---

## Suggested success criteria

A command reaches “compatible enough” when:

1. Common GNU flags are accepted and behave correctly
2. Exit codes match GNU behavior for success/no-match/error cases
3. stdout/stderr shape matches expected script usage
4. Existing Rush-native extensions (`--json`, structured mode) remain additive
5. Help text and docs exactly reflect implementation

---

## Immediate next actions

1. Create a parent `mana` feature for compatibility parity
2. Add child units for:
   - behavior-critical fixes
   - grep parity
   - find parity
   - ls parity
   - file-op parity
   - text-util parity
   - metadata/env parity
   - compatibility regression suite
3. Run `mana` orchestration on the feature tree
