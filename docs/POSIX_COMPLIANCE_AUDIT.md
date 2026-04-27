# AUSH POSIX Compliance Audit

**Date**: 2026-01-24
**Purpose**: Determine what AUSH needs to legitimately call itself "Unix"

## Executive Summary

AUSH has **solid POSIX foundations** but gaps remain. To call itself Unix:

**Current Status**: ~60% POSIX compliant
- ✅ Core execution model (pipes, redirects, subshells, jobs)
- ✅ 26 builtins implemented (including critical ones: cd, export, set)
- ❌ Missing 12+ required POSIX builtins
- ❌ Incomplete I/O redirection (no arbitrary FDs, no here-docs)
- ❌ No trap builtin for signal handling

**Verdict**: AUSH is a **capable shell** with excellent performance, but not yet a **POSIX shell**.

---

# Part 1: What AUSH HAS Implemented ✅

## 1.1 Core Execution Model

**Pipelines** (src/executor/pipeline.rs):
- Multi-stage pipelines with proper streaming
- Works with builtins and external commands
- SIGPIPE handling (broken pipe errors)
- Exit code: last command in pipeline
- `set -o pipefail` option

**I/O Redirection** (src/executor/mod.rs:191-268):
- `>` stdout to file
- `>>` append stdout to file
- `<` stdin from file
- `2>` stderr to file
- `2>&1` stderr to stdout
- `&>` both to file
- Proper path resolution, error handling

**Subshells** (src/executor/mod.rs:788-807):
- `(command)` syntax
- Isolated environment (variables, cwd)
- Proper result propagation

**Command Execution** (src/executor/mod.rs:162-189):
- Resolution order: functions → builtins → external commands
- PATH lookup for external commands
- Command-not-found with suggestions

## 1.2 Job Control

**Background Jobs** (src/jobs/mod.rs, src/executor/mod.rs:809-857):
- `command &` syntax
- Job tracking with IDs
- `jobs`, `fg`, `bg` builtins
- Job status: Running, Stopped, Done, Terminated
- SIGCONT/SIGTERM support
- Current/previous job tracking (%+, %-)

## 1.3 Signal Handling

**Basic Signals** (src/signal.rs):
- SIGINT, SIGTERM, SIGHUP handling
- Proper exit codes (130 for SIGINT, 143 for SIGTERM)
- Signal propagation to child processes
- Thread-safe atomic flags
- Non-blocking signal checking

## 1.4 Shell Options

**set Builtin** (src/runtime/mod.rs:11-20):
- `set -e` / `errexit` - exit on error
- `set -u` / `nounset` - error on undefined variable
- `set -x` / `xtrace` - print commands before execution
- `set -o pipefail` - pipeline fails if any command fails

## 1.5 Variable Expansion

**Basic Variables** (src/executor/mod.rs:913-943):
- `$VAR` simple expansion
- `${VAR}` braced expansion
- `$?` last exit code

**Parameter Expansion** (src/runtime/mod.rs):
- `${VAR:-default}` - use default if unset
- `${VAR:=default}` - assign default if unset
- `${VAR:?error}` - error if unset
- `${VAR#pattern}` - remove shortest prefix
- `${VAR##pattern}` - remove longest prefix
- `${VAR%pattern}` - remove shortest suffix
- `${VAR%%pattern}` - remove longest suffix

**Command Substitution** (src/executor/mod.rs:1049-1083):
- `$(command)` modern form
- `` `command` `` backtick form
- Proper stdout capture
- Trailing newline trimming

## 1.6 Control Flow

**Conditionals** (src/executor/mod.rs):
- `if/then/else/fi` statements
- `&&` and `||` operators
- `test` and `[` builtins

**Loops**:
- `for` loops
- `match` expressions (similar to case)

**Functions**:
- Function definitions
- Function calls with parameters
- Proper scoping with scope stack
- Call stack tracking with recursion limits

## 1.7 Implemented Builtins (26 total)

**Special Builtins** (POSIX required):
- `.` (source) - execute file in current shell
- `exit` - exit shell
- `export` - export variables to environment
- `set` - set shell options
- `unset` - unset variables

**Regular Builtins** (POSIX required):
- `alias` / `unalias` - command aliases
- `bg` / `fg` - background/foreground jobs
- `cd` - change directory
- `false` / `true` - return false/true
- `jobs` - list background jobs
- `pwd` - print working directory
- `type` - show command type
- `wait` - wait for background jobs

**Test/Comparison**:
- `test` - test conditions
- `[` - test conditions (bracket syntax)

**AUSH-Enhanced Builtins** (optimized versions):
- `cat` - concatenate files
- `echo` - print arguments
- `find` - find files
- `git` - git status integration
- `grep` - search patterns
- `ls` - list directory
- `mkdir` - make directory
- `printf` - formatted output

**AUSH-Specific**:
- `builtin` - run builtin bypassing functions
- `help` - show help
- `undo` - undo filesystem operations

## 1.8 Parsing & Expansion

**Alias Expansion** (src/executor/mod.rs:140-160):
- Proper alias resolution
- Arguments passed through
- Alias chaining

**Glob Expansion** (src/glob_expansion.rs):
- Pattern matching (`*.txt`, `foo*.rs`)
- Proper cwd-relative expansion
- Works in arguments

## 1.9 Environment & History

**Environment**:
- `export` sets environment variables
- Proper environment passing to child processes
- Current directory tracking

**History** (lazy-initialized):
- Command history tracking
- History storage

---

# Part 2: What AUSH NEEDS to Implement ❌

These are **required for POSIX compliance** and calling AUSH "Unix".

## 2.1 Critical Missing Builtins

**Loop Control** (POSIX required):
- `break` - exit from for/while/until loop
- `continue` - skip to next loop iteration

**No-Op**:
- `:` (colon) - always returns 0, used in conditionals

**Function Control**:
- `return` - ⚠️ EXISTS (commented out) - return from function

**Variable Control**:
- `readonly` - make variables immutable
- `shift` - ⚠️ EXISTS (commented out) - shift positional parameters

**Execution Control**:
- `eval` - ⚠️ EXISTS (commented out) - evaluate string as command
- `exec` - ⚠️ EXISTS (commented out) - replace shell with command
- `command` - run command bypassing functions/aliases

**I/O**:
- `read` - ⚠️ EXISTS (commented out) - read from stdin

**Signal Control**:
- `trap` - ⚠️ EXISTS (commented out) - catch signals and execute commands

**Process Control**:
- `kill` - ⚠️ EXISTS (commented out) - send signals to processes

**File Location**:
- Disabled builtins are in: src/builtins/{eval,exec,kill,local,read,return_builtin,shift,trap}.rs
- They're commented out in src/builtins/mod.rs:22-32,74-86

## 2.2 I/O Redirection Gaps

**Arbitrary File Descriptor Redirection**:
```bash
exec 3> file.txt        # Open FD 3 for writing
echo "hello" >&3        # Write to FD 3
exec 3>&-               # Close FD 3
command 3>&1 1>&2 2>&3  # Swap stdout and stderr
```

**Here Documents**:
```bash
cat <<EOF
line 1
line 2
EOF

cat <<-EOF              # Strip leading tabs
	indented
EOF
```

**FD Management**:
- Need FD table in Runtime
- Track open descriptors
- Proper FD inheritance to subshells
- exec builtin for permanent redirections

## 2.3 Positional Parameters

**Parameter Variables**:
```bash
$0                      # Script/shell name
$1, $2, ..., $9         # Arguments 1-9
${10}, ${11}, ...       # Arguments 10+
$#                      # Argument count
$@                      # All arguments (individually quoted)
$*                      # All arguments (merged)
```

AUSH has `positional_params` field in Runtime but not fully wired up.

**shift Builtin**:
- Shifts $1, $2, etc. down
- Code exists but is disabled

## 2.4 Special Variables

```bash
$$                      # Current shell PID
$!                      # Last background job PID
$-                      # Current shell options (flags)
$_                      # Last argument of previous command
```

Currently only `$?` (exit code) is implemented.

## 2.5 Control Flow Statements

**While Loop**:
```bash
while condition; do
    commands
done
```

**Until Loop**:
```bash
until condition; do
    commands
done
```

**Case Statement**:
```bash
case $var in
    pattern1) commands1 ;;
    pattern2) commands2 ;;
    *) default ;;
esac
```

AUSH has `match` which is similar but not POSIX syntax.

## 2.6 Process Groups & Terminal Control

**Process Groups**:
- Need `setpgid()` calls for all jobs
- Jobs should be in their own process group
- Shell should be in its own process group

**Terminal Control**:
- `tcsetpgrp()` for foreground process group
- Foreground jobs control terminal
- Background jobs don't control terminal

**Terminal Signals**:
- SIGTSTP - terminal stop
- SIGTTIN - background read from terminal
- SIGTTOU - background write to terminal

**SIGCHLD Handling**:
- Async reaping of zombie processes
- Automatic job status updates
- Proper wait() in SIGCHLD handler

## 2.7 Quoting & Escaping (NEEDS AUDIT)

Need comprehensive testing of:

**Single Quotes** (literal, no expansion):
```bash
echo 'literal $VAR'     # Should print: literal $VAR
```

**Double Quotes** (expansion allowed):
```bash
echo "expanded $VAR"    # Should expand VAR
echo "preserve \$VAR"   # Should print: preserve $VAR
```

**Escape Sequences**:
```bash
echo "line1\nline2"
echo "tab\there"
echo \$var              # Literal $var
```

**Quote Nesting**:
```bash
echo "outer 'inner' outer"
echo 'outer "inner" outer'
```

**Field Splitting & IFS**:
- `$IFS` variable controls word splitting
- Need to implement IFS-based splitting

---

# Part 3: What AUSH SHOULD Implement 💡

These are **nice to have** for better compatibility, but not strictly required for basic POSIX.

## 3.1 Secondary Builtins

**Argument Parsing**:
- `getopts` - parse command-line options in scripts

**Performance**:
- `hash` - cache command locations from PATH

**Resource Limits**:
- `ulimit` - set/get resource limits
- `umask` - set file creation mask

## 3.2 Shell Options

**Additional set Options**:
```bash
set -f     # noglob - disable glob expansion
set -n     # noexec - read but don't execute
set -v     # verbose - print input lines
set -C     # noclobber - prevent > from overwriting
set -m     # monitor - enable job control
set -a     # allexport - auto-export variables
```

AUSH has `noclobber` and `verbose` fields but they're not wired up.

**Positional Parameter Setting**:
```bash
set -- arg1 arg2 arg3   # Set positional parameters
set --                  # Clear positional parameters
```

## 3.3 Shell Variables

**Standard Variables**:
```bash
SHELL=/path/to/aush     # Path to shell executable
PWD=/current/dir        # Current directory
OLDPWD=/previous/dir    # Previous directory (for cd -)
PPID=12345             # Parent process ID
SHLVL=2                # Shell nesting level
```

**cd - Support**:
- `cd -` to return to previous directory
- Requires tracking OLDPWD

## 3.4 Job Specification

**Full Job Specs**:
```bash
%1          # Job 1
%+          # Current job (partially implemented)
%-          # Previous job (partially implemented)
%?string    # Job containing 'string'
%%          # Alias for %+
```

AUSH has `get_current_job()` and `get_previous_job()` but not full parsing.

## 3.5 Advanced I/O

**Named Pipes**:
- Support for FIFOs (mkfifo)
- Opening named pipes for read/write

**Stderr in Pipelines**:
- Test that `command1 2>&1 | command2` works
- Proper stderr redirection in pipeline context

## 3.6 Bash Extensions (Optional)

**String Operations** (not POSIX, but common):
```bash
${#VAR}                # String length
${VAR:offset:length}   # Substring
${VAR/pattern/repl}    # Substitution
```

**Brace Expansion** (not POSIX):
```bash
echo {a,b,c}           # Expands to: a b c
echo {1..5}            # Expands to: 1 2 3 4 5
```

**Process Substitution** (not POSIX):
```bash
diff <(sort file1) <(sort file2)
command >(tee log.txt)
```

**Tilde Expansion**:
```bash
~/file                 # Expands to: /home/user/file
~user/file             # Expands to: /home/user/file
```

## 3.7 Performance & Optimization

**Command Hashing**:
- Cache PATH lookups in hash table
- `hash` builtin to manage cache

**Completion** (already partially implemented):
- Tab completion for commands, files
- Programmable completion

---

# Implementation Roadmap

## Priority 1: Core POSIX Compliance (Critical)

**Estimated Effort**: 3-4 weeks

### Week 1: Re-enable Disabled Builtins
- [ ] Uncomment and test: `eval`, `exec`, `kill`, `read`, `return`, `shift`, `trap`
- [ ] Fix any issues preventing their use
- [ ] Add tests for each builtin

### Week 2: Missing Critical Builtins
- [ ] Implement `:` (colon/no-op)
- [ ] Implement `break`
- [ ] Implement `continue`
- [ ] Implement `command` (bypass aliases/functions)
- [ ] Implement `readonly`

### Week 3: Positional Parameters & Quoting
- [ ] Wire up positional parameters: $0, $1-$9, ${10}+, $#, $@, $*
- [ ] Implement special variables: $$, $!, $-, $_
- [ ] Audit quoting implementation
- [ ] Fix quoting bugs if found
- [ ] Implement IFS-based field splitting

### Week 4: Here-Docs & Control Flow
- [ ] Implement here-documents (<<EOF)
- [ ] Implement `while` loop
- [ ] Implement `until` loop
- [ ] Implement `case` statement (or make `match` POSIX-compatible)

## Priority 2: Job Control & Signals (Important)

**Estimated Effort**: 2 weeks

### Week 5: Process Groups
- [ ] Implement process groups (setpgid)
- [ ] Terminal control (tcsetpgrp)
- [ ] Full job spec parsing (%1, %+, %-, %?pattern)

### Week 6: Signal Handling
- [ ] Implement SIGCHLD handler for zombie reaping
- [ ] Add SIGTSTP/SIGTTIN/SIGTTOU handling
- [ ] Test trap builtin thoroughly
- [ ] Process group signal delivery

## Priority 3: Advanced I/O (Nice to Have)

**Estimated Effort**: 2 weeks

### Week 7: File Descriptors
- [ ] Implement arbitrary FD redirection (N>&M, N<&M, N>&-)
- [ ] Add FD table to Runtime
- [ ] Proper FD inheritance in subshells
- [ ] Wire up exec builtin for permanent redirections

### Week 8: Polish
- [ ] Implement secondary builtins (getopts, hash, ulimit, umask)
- [ ] Complete shell variables (SHELL, PWD, OLDPWD, PPID)
- [ ] Test stderr in pipelines
- [ ] Named pipe support

## Priority 4: Testing & Validation

**Estimated Effort**: 1-2 weeks

### Week 9-10: Testing
- [ ] Run POSIX test suite
- [ ] Fix failures
- [ ] Add regression tests
- [ ] Test real shell scripts
- [ ] Document POSIX compliance level
- [ ] Mark non-POSIX extensions clearly

---

# What Makes AUSH "Unix"?

To legitimately call AUSH a Unix shell, implement **Priority 1 + Priority 2**:

**Must Have** (Priority 1):
- ✅ All required POSIX builtins
- ✅ Positional parameters ($1, $@, etc.)
- ✅ Here-documents
- ✅ While/until/case
- ✅ Proper quoting

**Should Have** (Priority 2):
- ✅ Process groups
- ✅ SIGCHLD handling
- ✅ Full job control

**Nice to Have** (Priority 3-4):
- Arbitrary FD redirection
- Secondary builtins
- POSIX test suite passing

---

# Strategic Decision

AUSH can be:

**A) Strict POSIX Shell**
- Focus: 100% POSIX compliance
- Effort: 8-12 weeks
- Result: Run any standard shell script

**B) AI-Native Unix Shell** ⭐ (RECOMMENDED)
- Focus: Core POSIX + AI-native extensions
- Effort: 5-7 weeks for core, then AI features
- Result: Unix legitimacy + AI superpowers
- This aligns with your vision!

**C) Fast Shell, Unix-Inspired**
- Focus: Performance, selected POSIX features
- Effort: Ongoing, pick features as needed
- Result: Shell-like tool, not Unix-compatible

**Recommendation**: Go with **Option B**. Implement Priority 1 + Priority 2 for Unix legitimacy, then build AI-native tools that compose with Unix principles.

---

# Next Steps

1. **Decide**: Which priority level do you want to reach?
2. **Start**: Re-enable disabled builtins (easiest wins)
3. **Test**: Build test suite as you implement
4. **Document**: Be clear about what's POSIX vs extension

**Quick Win**: Uncomment builtins in `src/builtins/mod.rs` and see what works immediately!
