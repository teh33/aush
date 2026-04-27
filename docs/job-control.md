# Job Control Implementation

## Overview

AUSH shell implements full job control functionality, allowing users to run commands in the background, manage multiple jobs, and bring them to the foreground. This feature enables efficient multitasking within the shell.

## Features

### Background Job Execution

Run any command in the background by appending `&` to the command:

```bash
sleep 100 &
```

When a job is started in background, AUSH displays:
```
[1] 12345
```

Where:
- `[1]` is the job number
- `12345` is the process ID (PID)

### Job Management Builtins

#### `jobs` - List Background Jobs

List all background jobs with their status:

```bash
jobs
```

Output format:
```
[1] Running    sleep 100
[2] Stopped    vim file.txt
[3] Done       find / -name "*.txt"
```

**Options:**
- `-l` - Show process IDs
- `-r` - Show only running jobs
- `-s` - Show only stopped jobs

Example with `-l` flag:
```bash
jobs -l
```
Output:
```
[1] Running 12345    sleep 100
[2] Stopped 12346    vim file.txt
```

#### `fg` - Foreground a Job

Bring a background job to the foreground:

```bash
fg              # Bring most recent job to foreground
fg %1           # Bring job 1 to foreground
fg %sleep       # Bring job matching "sleep" to foreground
```

**Job Specifications:**
- `%1`, `%2`, etc. - Job number
- `%%` or `%+` - Current (most recent) job
- `%-` - Previous job
- `%sleep` - Job whose command starts with "sleep"

#### `bg` - Continue Stopped Job in Background

Continue a stopped job in the background:

```bash
bg              # Continue most recent stopped job
bg %1           # Continue job 1 in background
```

This is useful when you've suspended a job with Ctrl-Z and want it to continue running in the background.

### Kill Jobs

Use the standard `kill` command with job specifications:

```bash
kill %1         # Terminate job 1
kill %sleep     # Terminate job matching "sleep"
```

## Job Status

Jobs can be in one of four states:

1. **Running** - Job is actively executing
2. **Stopped** - Job has been suspended (e.g., with Ctrl-Z)
3. **Done** - Job has completed successfully
4. **Terminated** - Job was killed or terminated by a signal

## Implementation Details

### Architecture

The job control system consists of several components:

1. **JobManager** (`src/jobs/mod.rs`)
   - Tracks all background jobs
   - Maintains job state (Running, Stopped, Done, Terminated)
   - Provides job lookup by ID, PID, or command prefix
   - Handles job lifecycle management

2. **Job Struct**
   ```rust
   pub struct Job {
       pub id: usize,           // Job number
       pub pid: u32,            // Process ID
       pub command: String,     // Command string
       pub status: JobStatus,   // Current status
   }
   ```

3. **Parser Integration**
   - New `Ampersand` token in lexer
   - `BackgroundCommand` variant in AST
   - Parser detects `&` at end of command

4. **Executor Integration**
   - `execute_background()` method spawns processes
   - Background processes have stdin/stdout/stderr redirected to /dev/null
   - Job ID and PID returned to user

5. **Runtime Integration**
   - JobManager embedded in Runtime
   - Accessible via `runtime.job_manager()` and `runtime.job_manager_mut()`

### Job Status Updates

Job statuses are updated in two ways:

1. **Periodic Updates** - Main loop calls `update_all_jobs()` before each prompt
2. **On-Demand** - Builtins like `jobs`, `fg`, and `bg` update status before executing

The update mechanism uses `waitpid()` with `WNOHANG` flag to non-blockingly check process status:

```rust
match waitpid(pid, Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED)) {
    Ok(WaitStatus::Exited(_, _)) => status = JobStatus::Done,
    Ok(WaitStatus::Signaled(_, _, _)) => status = JobStatus::Terminated,
    Ok(WaitStatus::Stopped(_, _)) => status = JobStatus::Stopped,
    Ok(WaitStatus::StillAlive) => status = JobStatus::Running,
    // ...
}
```

### Job Completion Notifications

The main interactive loop displays notifications when jobs complete:

```
[1] Done        sleep 100
[2] Terminated  rogue-process
```

Completed and terminated jobs are automatically cleaned up after notification.

## Limitations

1. **Builtin Commands** - Cannot run builtin commands (cd, echo, etc.) in background
2. **Complex Statements** - Currently only simple commands can be backgrounded
3. **Job Control Signals** - Ctrl-Z to suspend jobs is not yet implemented
4. **Process Groups** - Jobs don't have their own process groups yet

## Future Enhancements

- [ ] Support for Ctrl-Z to suspend foreground jobs
- [ ] Process group management for proper signal handling
- [ ] Background pipelines (e.g., `cmd1 | cmd2 &`)
- [ ] Background subshells (e.g., `(cmd1; cmd2) &`)
- [ ] Job status in prompt
- [ ] Configurable job notification timing
- [ ] Job persistence across shell sessions

## Examples

### Basic Usage

```bash
# Start a long-running task in background
find / -name "*.log" > logs.txt 2>&1 &

# List jobs
jobs

# Start another job
sleep 300 &

# List with PIDs
jobs -l

# Bring job to foreground
fg %1

# (After Ctrl-Z in future version)
# Continue in background
bg %1
```

### Multiple Jobs

```bash
# Start multiple background jobs
./build.sh &
./test.sh &
./deploy.sh &

# Check status
jobs

# Only show running jobs
jobs -r

# Foreground the build job
fg %build
```

### Job Specifications

```bash
# By job number
fg %1
bg %2

# Current job
fg %%
fg %+

# Previous job
fg %-

# By command prefix
fg %vim
bg %sleep
```

## Testing

Comprehensive tests are provided in `tests/job_control_tests.rs`:

- Background command parsing and execution
- Multiple background jobs
- Job listing with various flags
- Job completion detection
- Job specification parsing
- Job manager lifecycle

Run tests:
```bash
cargo test job_control
```

## Technical Notes

### Signal Handling

Job control relies on Unix signals:
- `SIGCHLD` - Notified when child process state changes (handled via waitpid)
- `SIGCONT` - Continue stopped process (sent by `bg` command)
- `SIGTERM` - Terminate process (sent by `kill` command)

### Process Management

Background jobs are spawned with:
- Redirected I/O (stdin, stdout, stderr to /dev/null)
- Independent process lifetime
- Non-blocking status checks

### Thread Safety

The JobManager uses Arc<Mutex<>> internally to ensure thread-safe access to job state across different parts of the executor.

## References

- POSIX Job Control: IEEE Std 1003.1-2008
- Bash Job Control: GNU Bash Reference Manual, Chapter 7
- AUSH Implementation: `src/jobs/mod.rs`, `src/builtins/jobs.rs`
