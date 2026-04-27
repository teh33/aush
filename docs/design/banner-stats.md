# AUSH Banner & System Stats Design

## Overview

Display configurable system stats alongside the ASCII banner at shell startup.
Stats are cached by the daemon for near-zero latency.

## Banner Display

```
 █▀▄ █ █ █▀▀ █ █
 █   █ █ ▀▀█ █▀█  v0.1.0
 ▀   ▀▀▀ ▀▀▀ ▀ ▀
 ─────────────────────────
  asher@macbook  up 3d 14h
  macOS 14.2     mem 8.2/16G
```

No emojis - ASCII labels only for terminal compatibility.

## Configuration (~/.aushrc)

```bash
# Banner style: block, line, minimal, none
AUSH_BANNER_STYLE="block"

# Banner color: cyan, green, yellow, magenta, blue, white, none
AUSH_BANNER_COLOR="cyan"

# Show banner: always, first (first shell only), never
AUSH_BANNER_SHOW="always"

# Stats to display (space-separated)
# Mix built-in and custom stats freely
AUSH_BANNER_STATS="host uptime memory"
```

## Built-in Stats

Fast stats using direct syscalls (no subprocess overhead):

| Stat | Example | Refresh |
|------|---------|---------|
| `host` | `asher@macbook` | static |
| `os` | `macOS 14.2` | static |
| `kernel` | `Darwin 23.2.0` | static |
| `arch` | `arm64` | static |
| `cpu` | `Apple M1 Pro` | static |
| `cores` | `10` | static |
| `uptime` | `3d 14h` | 5s |
| `load` | `2.1 1.8 1.5` | 5s |
| `procs` | `312` | 5s |
| `memory` | `8.2/16G` | 5s |
| `swap` | `0/4G` | 5s |
| `disk` | `234/500G` | 30s |
| `battery` | `78%` | 30s |
| `power` | `AC` / `bat` | 30s |
| `time` | `8:06 PM` | 1s |
| `date` | `Tue Feb 4` | 60s |
| `ip` | `192.168.1.42` | 30s |
| `wifi` | `MyNetwork` | 30s |

## Custom Stats

Define any stat with a shell command. Daemon runs it periodically and caches the output.

```bash
# Format: AUSH_STAT_<name>="<command>"
AUSH_STAT_weather="curl -s 'wttr.in?format=%t'"
AUSH_STAT_todos="wc -l < ~/todo.txt"
AUSH_STAT_branch="git -C ~/projects/main branch --show-current 2>/dev/null"
AUSH_STAT_docker="docker ps -q 2>/dev/null | wc -l | tr -d ' '"
AUSH_STAT_k8s="kubectl get pods --no-headers 2>/dev/null | wc -l"
AUSH_STAT_mail="ls ~/Mail/INBOX/new | wc -l"
AUSH_STAT_spotify="osascript -e 'tell app \"Spotify\" to name of current track' 2>/dev/null"

# Custom refresh interval (optional, default 30s)
AUSH_STAT_weather_INTERVAL=300    # 5 minutes
AUSH_STAT_todos_INTERVAL=10       # 10 seconds
AUSH_STAT_docker_INTERVAL=15      # 15 seconds

# Use in banner
AUSH_BANNER_STATS="host uptime weather todos docker"
```

### Custom stat behavior

- **Timeout**: Commands killed after 2s (configurable via `AUSH_STAT_<name>_TIMEOUT`)
- **Failure**: Error cached, retried next interval
- **First run**: Daemon collects all custom stats on startup (may delay first banner by a few ms)
- **Output**: First line of stdout only, trimmed
- **No limit**: Users can define as many custom stats as they want

### Error display

- **In banner**: Show `--` for failed/empty stats (keeps it clean)
- **In `aush --info`**: Show short error like `[timeout]`, `[not found]`, `[exit 1]`

```
# Banner (clean)
  weather   --
  docker    3

# aush --info (detailed)
  weather   [timeout]       (updated 30s ago)
  docker    3               (updated 10s ago)
```

### Config reload

Daemon reads `.aushrc` on startup. To reload config:
- Send SIGHUP: `kill -HUP $(cat ~/.aush/daemon.pid)`
- Or: `aush daemon reload`

### Example: Git-aware prompt stat

```bash
# Show repo state if in a git directory
AUSH_STAT_git='
  branch=$(git branch --show-current 2>/dev/null) || exit 0
  dirty=$(git status --porcelain 2>/dev/null | head -1)
  [ -n "$dirty" ] && echo "$branch*" || echo "$branch"
'
AUSH_STAT_git_INTERVAL=5
```

## Platform Support for Built-ins

| Stat | macOS | Linux |
|------|-------|-------|
| `host`, `os`, `kernel`, `arch` | ✓ uname/sysctl | ✓ uname |
| `cpu`, `cores` | ✓ sysctl | ✓ /proc/cpuinfo |
| `uptime` | ✓ sysctl | ✓ /proc/uptime |
| `load` | ✓ getloadavg | ✓ /proc/loadavg |
| `procs` | ✓ sysctl | ✓ /proc |
| `memory`, `swap` | ✓ vm_stat | ✓ /proc/meminfo |
| `disk` | ✓ statfs | ✓ statfs |
| `battery`, `power` | ✓ pmset | ✓ /sys/class/power_supply |
| `ip` | ✓ getifaddrs | ✓ getifaddrs |
| `wifi` | ✓ airport | ✓ iwconfig/iw |
| `time`, `date` | ✓ | ✓ |

Anything not built-in → use custom stats with shell commands.

## Protocol Extension

Add to `Message` enum:

```rust
/// Request system stats from daemon
StatsRequest(StatsRequest),
/// Daemon returns cached stats  
StatsResponse(StatsResponse),
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsRequest {
    /// Which stats to fetch (empty = all cached)
    pub stats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    /// Static stats (host, os, etc.)
    pub static_stats: HashMap<String, String>,
    /// Dynamic stats (uptime, memory, etc.)  
    pub dynamic_stats: HashMap<String, String>,
    /// When dynamic stats were last updated
    pub updated_at: u64,
}
```

## Daemon Implementation

### Custom stat execution

```rust
struct CustomStat {
    name: String,
    command: String,
    interval: Duration,
    timeout: Duration,
    last_value: String,
    last_update: Instant,
    last_error: Option<String>,
}

impl CustomStat {
    fn needs_update(&self) -> bool {
        self.last_update.elapsed() >= self.interval
    }
    
    fn update(&mut self) {
        // Run command with timeout, capture stdout
        let result = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .timeout(self.timeout)
            .output();
            
        match result {
            Ok(output) => {
                // First line only, trimmed
                self.last_value = output.stdout
                    .lines().next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                self.last_error = None;
            }
            Err(e) => {
                self.last_value = String::new();
                self.last_error = Some(e.to_string());
            }
        }
        self.last_update = Instant::now();
    }
}
```

### On daemon start:
```rust
struct StatsCache {
    // Static built-ins (computed once)
    hostname: String,
    username: String,
    os_name: String,
    os_version: String,
    kernel: String,
    arch: String,
    cpu_model: String,
    cpu_cores: u32,
    total_memory_bytes: u64,
    total_disk_bytes: u64,
    
    // Dynamic built-ins (updated periodically)
    uptime_secs: u64,
    used_memory_bytes: u64,
    used_swap_bytes: u64,
    used_disk_bytes: u64,
    load_avg: [f64; 3],
    proc_count: u32,
    battery_percent: Option<u8>,
    battery_charging: Option<bool>,
    local_ip: Option<String>,
    wifi_ssid: Option<String>,
    
    // Custom stats
    custom: HashMap<String, CustomStat>,
    
    // Metadata
    last_builtin_update: Instant,
}
```

### In health check loop:
```rust
fn update_stats(&mut self) {
    let now = Instant::now();
    
    // Update built-in dynamic stats every 5s
    if now.duration_since(self.stats.last_builtin_update) >= Duration::from_secs(5) {
        self.stats.update_builtins();
    }
    
    // Update custom stats based on their individual intervals
    for stat in self.stats.custom.values_mut() {
        if stat.needs_update() {
            stat.update();  // Runs in background thread to avoid blocking
        }
    }
}
```

Custom stat updates run in a thread pool to avoid blocking the main loop if a command is slow.

### On StatsRequest:
```rust
// Just return cached values - no syscalls needed
Message::StatsResponse(StatsResponse {
    static_stats: self.stats.static_to_map(),
    dynamic_stats: self.stats.dynamic_to_map(),
    updated_at: self.stats.last_update.elapsed().as_secs(),
})
```

## Client Flow

```
┌─────────┐     ┌────────┐
│  aush   │     │ aushd  │
└────┬────┘     └────┬───┘
     │               │
     │ StatsRequest  │
     │──────────────►│
     │               │ (return cached)
     │ StatsResponse │
     │◄──────────────│
     │               │
     │ Display banner│
     ▼               │
```

## Behavior by Mode

### With daemon running:
- Banner + configured stats
- Stats fetched from daemon cache (<1ms)
- `aush --info` returns full stats instantly

### Without daemon:
- Banner only, no stats (zero penalty)
- `aush --info` collects stats on-demand (~10-15ms, acceptable for explicit command)

This keeps AUSH fast by default. Stats are a **daemon perk**, not a core feature.

## Platform Support

| Platform | uptime | memory | disk | load | cpu |
|----------|--------|--------|------|------|-----|
| macOS | sysctl | vm_stat | statfs | getloadavg | sysctl |
| Linux | /proc/uptime | /proc/meminfo | statfs | /proc/loadavg | /proc/cpuinfo |

## `aush --info` Command

Always available, shows all stats (built-in + custom):

```
$ aush --info
aush v0.1.0

Built-in:
  host      asher@macbook
  os        macOS 14.2
  kernel    Darwin 23.2.0
  arch      arm64
  cpu       Apple M1 Pro (10 cores)
  uptime    3d 14h 22m
  load      2.14 1.82 1.53
  procs     312
  memory    8.2/16G (51%)
  swap      0/4G (0%)
  disk      234/500G (47%)
  battery   78% (charging)
  ip        192.168.1.42
  wifi      MyNetwork

Custom:
  weather   72°F          (updated 2m ago)
  todos     12            (updated 5s ago)
  docker    3             (updated 10s ago)
  branch    main          (updated 5s ago)

Daemon:
  status    running (pid 12345)
  uptime    1d 2h
  workers   4 pooled
  requests  1,247 served
```

- With daemon: instant (reads cache)
- Without daemon: collects built-ins on-demand (~15ms), skips custom

### `aush --info <stat>`

Show single stat value (useful for scripting):

```
$ aush --info memory
8.2/16G

$ aush --info weather  
72°F
```

### `aush --info --json`

Machine-readable output:

```json
{
  "version": "0.1.0",
  "builtin": {
    "host": "asher@macbook",
    "os": "macOS 14.2",
    "memory": "8.2/16G",
    ...
  },
  "custom": {
    "weather": {"value": "72°F", "updated_ago_secs": 120},
    "todos": {"value": "12", "updated_ago_secs": 5},
    ...
  },
  "daemon": {
    "running": true,
    "pid": 12345,
    "uptime_secs": 93600
  }
}
```

## Future Ideas

- `git` stat - show branch/status if in repo
- `jobs` stat - background job count
- `last_cmd` stat - last command exit code/time
- Custom stats via plugins
