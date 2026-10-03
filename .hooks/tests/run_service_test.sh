#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
RUN_SERVICE="$ROOT_DIR/core/run-service.sh"
WORK_DIR=$(mktemp -d)
trap 'pkill -KILL -f "$WORK_DIR/" || true; rm -rf "$WORK_DIR"' EXIT
# run-service creates its cgroup under /sys/fs/cgroup/$DOCKER_CGROUP, so point it at the work folder
export DOCKER_CGROUP="../../..$WORK_DIR/cgroup"

fail() {
    printf 'run_service_test: %s\n' "$1" >&2
    exit 1
}

# Writes a service that logs each start, exits 0 on its first run when asked to, and on SIGTERM takes a while to
# finish cleanly, like the recorder writing its MCAP footer
write_service() {
    local folder=$1
    local exit_on_first_run=$2
    mkdir -p "$folder"
    cat > "$folder/service" <<EOF
#!/usr/bin/env bash
trap 'sleep 0.5; echo stopped >> "$folder/events"; exit 0' TERM
echo started >> "$folder/events"
if [ "$exit_on_first_run" = true ] && [ "\$(grep -c started "$folder/events")" -eq 1 ]; then
    exit 0
fi
while true; do sleep 0.1; done
EOF
    chmod +x "$folder/service"
    touch "$folder/events"
}

wait_for_events() {
    local folder=$1
    local event=$2
    local count=$3
    local attempts=150
    while [ "$(grep -c "$event" "$folder/events")" -lt "$count" ]; do
        attempts=$((attempts - 1))
        [ "$attempts" -gt 0 ] || fail "timed out waiting for $count '$event' in $folder"
        sleep 0.1
    done
}

wait_for_exit() {
    local pid=$1
    local attempts=50
    while kill -0 "$pid" 2>/dev/null; do
        attempts=$((attempts - 1))
        [ "$attempts" -gt 0 ] || fail "run-service $pid did not exit after SIGTERM"
        sleep 0.1
    done
}

test_restarts_a_service_that_exits_zero() {
    local folder="$WORK_DIR/exits_zero"
    write_service "$folder" true
    bash "$RUN_SERVICE" exits_zero "$folder/service" > "$folder/log" 2>&1 &
    local run_service_pid=$!
    wait_for_events "$folder" started 2
    kill -TERM "$run_service_pid"
    wait_for_exit "$run_service_pid"
}

test_sigterm_stops_the_service_cleanly_without_restart() {
    local folder="$WORK_DIR/sigterm"
    write_service "$folder" false
    bash "$RUN_SERVICE" sigterm "$folder/service" > "$folder/log" 2>&1 &
    local run_service_pid=$!
    wait_for_events "$folder" started 1
    kill -TERM "$run_service_pid"
    wait_for_exit "$run_service_pid"
    grep -q stopped "$folder/events" || fail "run-service exited before the service finished its shutdown"
    [ "$(grep -c started "$folder/events")" -eq 1 ] || fail "run-service restarted a service it was asked to stop"
}

main() {
    test_sigterm_stops_the_service_cleanly_without_restart
    test_restarts_a_service_that_exits_zero
    printf 'run_service_test: ok\n'
}

main "$@"
