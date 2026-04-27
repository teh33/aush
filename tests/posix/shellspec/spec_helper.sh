# ShellSpec Helper Functions
# Shared utilities for POSIX compliance testing

# Get the aush binary path
rush_binary() {
    if [ -n "${AUSH_BINARY:-}" ]; then
        echo "$AUSH_BINARY"
    else
        echo "../../target/release/aush"
    fi
}

# Run a aush command
aush() {
    "$(rush_binary)" "$@"
}

# Run aush with -c flag (command string)
rush_c() {
    "$(rush_binary)" -c "$1"
}

# Check if aush binary exists
rush_exists() {
    [ -f "$(rush_binary)" ]
}

# Get aush version
rush_version() {
    "$(rush_binary)" --version 2>&1 || echo "unknown"
}
