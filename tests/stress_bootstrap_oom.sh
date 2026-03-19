#!/usr/bin/env bash
# Stress test: bootstrap self-compilation under cgroup memory limits
#
# Reproduces the OOM scenario from March 18, 2026 where ore-native2
# (the bootstrapped native compiler) consumed 28.5GB RSS trying to
# self-compile, triggering kernel OOM kills 3 times in 30 minutes.
#
# This test runs the bootstrap self-compile step inside a 16GB
# systemd-run scope so the OOM kill is contained and doesn't affect
# the rest of the system.
#
# Requirements:
#   - cargo, cc, systemd-run --user
#   - LLVM 19.1 at /tmp/LLVM-19.1.0-Linux-X64 (or LLVM_SYS_191_PREFIX set)
#
# Usage:
#   ./tests/stress_bootstrap_oom.sh [--memory-limit 16G]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$SCRIPT_DIR"

MEMORY_LIMIT="${1:-16G}"
if [[ "$1" == "--memory-limit" ]] 2>/dev/null; then
    MEMORY_LIMIT="${2:-16G}"
fi

LOG="stress-test-bootstrap-oom.log"
TMPDIR_BASE="/tmp/ore-stress-$$"
mkdir -p "$TMPDIR_BASE"

# Cleanup on exit
cleanup() {
    rm -rf "$TMPDIR_BASE"
}
trap cleanup EXIT

log() {
    local msg="$1"
    local ts
    ts="$(date -Iseconds)"
    echo "[$ts] $msg" | tee -a "$LOG"
}

# Read cgroup memory stats for a scope
read_cgroup_memory() {
    local scope_name="$1"
    local cgroup_path="/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/${scope_name}"
    if [[ -d "$cgroup_path" ]]; then
        local current peak
        current=$(cat "$cgroup_path/memory.current" 2>/dev/null || echo "0")
        peak=$(cat "$cgroup_path/memory.peak" 2>/dev/null || echo "0")
        echo "current=$((current / 1048576))MB peak=$((peak / 1048576))MB"
    else
        echo "cgroup=$cgroup_path (not found)"
    fi
}

# Check system health: are key services still running?
check_system_health() {
    local healthy=true
    # Check that we can still allocate memory
    if ! python3 -c "x = bytearray(1024*1024); print('alloc ok')" >/dev/null 2>&1; then
        log "WARNING: system memory allocation failed"
        healthy=false
    fi
    # Check dolt is still running (critical for Gas Town)
    if pgrep -f "dolt sql-server" >/dev/null 2>&1; then
        log "  dolt: running"
    else
        log "  dolt: not running (may not have been started)"
    fi
    # Check basic system responsiveness
    if ! uptime >/dev/null 2>&1; then
        log "WARNING: system unresponsive"
        healthy=false
    fi
    $healthy
}

# Check for OOM events in journal
check_oom_events() {
    local scope_name="$1"
    local since="$2"
    # Check both journalctl and dmesg for OOM events
    local oom_count=0
    if journalctl --user --since="$since" 2>/dev/null | grep -c "oom-kill\|oom_reaper\|Out of memory\|memory\.max" || true; then
        oom_count=$(journalctl --user --since="$since" 2>/dev/null | grep -c "oom-kill\|oom_reaper\|Out of memory\|memory\.max" || echo "0")
    fi
    # Also check dmesg
    local dmesg_ooms=0
    dmesg_ooms=$(dmesg --time-format iso 2>/dev/null | grep "oom-kill\|oom_reaper\|Out of memory" | awk -v since="$since" '$0 >= since' | wc -l || echo "0")
    echo "$((oom_count + dmesg_ooms))"
}

echo "=== ORE STRESS TEST: Bootstrap Self-Compilation (OOM Trigger) ===" | tee "$LOG"
log "Memory limit: $MEMORY_LIMIT"
log "Working directory: $SCRIPT_DIR"
log "Temp directory: $TMPDIR_BASE"

# Verify prerequisites
if ! command -v systemd-run >/dev/null 2>&1; then
    log "ERROR: systemd-run not found (required for cgroup isolation)"
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    log "ERROR: cargo not found"
    exit 1
fi

export LLVM_SYS_191_PREFIX="${LLVM_SYS_191_PREFIX:-/tmp/LLVM-19.1.0-Linux-X64}"
if [[ ! -d "$LLVM_SYS_191_PREFIX" ]]; then
    log "ERROR: LLVM not found at $LLVM_SYS_191_PREFIX"
    exit 1
fi

# Record system state before test
log "System memory before test:"
free -m | tee -a "$LOG"

# --- Step 1: Build the Rust compiler (cargo build --release) ---
log "--- Step 1: cargo build --release ---"
cargo build --release 2>&1 | tail -5 | tee -a "$LOG"
log "Step 1 complete"

# Verify the ore binary exists
ORE_BIN="target/release/ore"
if [[ ! -x "$ORE_BIN" ]]; then
    log "ERROR: $ORE_BIN not found after cargo build"
    exit 1
fi

# --- Step 2: Compile native/main.ore via C backend → ore-native1 ---
log "--- Step 2: Build ore-native1 via C backend ---"
ORE_NATIVE1="$TMPDIR_BASE/ore-native1"
cargo run --release -- build --backend c src/main.ore -o "$ORE_NATIVE1" 2>&1 | tee -a "$LOG"
if [[ ! -x "$ORE_NATIVE1" ]]; then
    log "ERROR: ore-native1 not built"
    exit 1
fi
log "Step 2 complete: ore-native1 built at $ORE_NATIVE1"

# --- Step 3: Bootstrap self-compile under cgroup limit ---
# This is THE step that OOM'd on March 18 — ore-native1 trying to compile
# its own source (src/main.ore) into ore-native2.
log "--- Step 3: Bootstrap self-compile under ${MEMORY_LIMIT} cgroup limit ---"

ORE_NATIVE2="$TMPDIR_BASE/ore-native2"
SCOPE_NAME="ore-stress-bootstrap-$$.scope"
BEFORE_TS="$(date -Iseconds)"

# Record pre-run memory
log "Pre-bootstrap system state:"
free -m | tee -a "$LOG"

# Run the self-compile inside a memory-limited scope
# We expect this to either:
#   a) Complete successfully if the compiler has been fixed
#   b) Get OOM-killed by the cgroup limit if it still leaks
log "Running: ore-native1 build src/main.ore -o ore-native2 (inside $MEMORY_LIMIT scope)"

set +e  # Don't exit on error — we expect potential OOM
BOOTSTRAP_START="$(date +%s)"

systemd-run --user --scope \
    -p MemoryMax="$MEMORY_LIMIT" \
    -p MemoryHigh="$((${MEMORY_LIMIT%G} * 3 / 4))G" \
    --unit="$SCOPE_NAME" \
    -- "$ORE_NATIVE1" build src/main.ore -o "$ORE_NATIVE2" \
    >"$TMPDIR_BASE/bootstrap-stdout.log" 2>"$TMPDIR_BASE/bootstrap-stderr.log"
BOOTSTRAP_EXIT=$?

BOOTSTRAP_END="$(date +%s)"
BOOTSTRAP_DURATION=$((BOOTSTRAP_END - BOOTSTRAP_START))
set -e

log "Bootstrap exit code: $BOOTSTRAP_EXIT (duration: ${BOOTSTRAP_DURATION}s)"

# Capture stdout/stderr
if [[ -s "$TMPDIR_BASE/bootstrap-stdout.log" ]]; then
    log "Bootstrap stdout:"
    cat "$TMPDIR_BASE/bootstrap-stdout.log" | tee -a "$LOG"
fi
if [[ -s "$TMPDIR_BASE/bootstrap-stderr.log" ]]; then
    log "Bootstrap stderr:"
    cat "$TMPDIR_BASE/bootstrap-stderr.log" | tee -a "$LOG"
fi

# --- Step 4: Analyze results ---
log "--- Step 4: Analyze results ---"

# Check for OOM events
OOM_COUNT=$(check_oom_events "$SCOPE_NAME" "$BEFORE_TS")
log "OOM events detected since bootstrap start: $OOM_COUNT"

# Check cgroup memory peak (may not be available after scope exits)
# Try reading from journal instead
if journalctl --user --since="$BEFORE_TS" -u "$SCOPE_NAME" 2>/dev/null | head -20 | tee -a "$LOG"; then
    true
fi

# Report outcome
if [[ $BOOTSTRAP_EXIT -eq 0 ]] && [[ -x "$ORE_NATIVE2" ]]; then
    log "RESULT: Bootstrap self-compile SUCCEEDED"
    log "  ore-native2 built successfully at $ORE_NATIVE2"
    # Verify the binary works
    if "$ORE_NATIVE2" help >/dev/null 2>&1; then
        log "  ore-native2 --help: OK"
    else
        log "  ore-native2 --help: FAILED (binary may be corrupt)"
    fi
elif [[ $BOOTSTRAP_EXIT -eq 137 ]] || [[ $BOOTSTRAP_EXIT -eq 143 ]] || [[ $OOM_COUNT -gt 0 ]]; then
    log "RESULT: Bootstrap self-compile was OOM-KILLED (exit=$BOOTSTRAP_EXIT)"
    log "  This confirms the memory leak / excessive allocation in self-compilation."
    log "  The ${MEMORY_LIMIT} cgroup limit successfully contained the OOM."
else
    log "RESULT: Bootstrap self-compile FAILED (exit=$BOOTSTRAP_EXIT)"
    log "  Non-OOM failure — check stderr above for details."
fi

# --- Step 5: Verify system health ---
log "--- Step 5: System health check ---"
log "Post-test system state:"
free -m | tee -a "$LOG"

if check_system_health; then
    log "System health: OK"
else
    log "System health: DEGRADED"
fi

log ""
log "=== STRESS TEST COMPLETE ==="
log "Results written to: $LOG"
