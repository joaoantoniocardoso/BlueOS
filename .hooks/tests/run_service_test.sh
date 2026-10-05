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

# Runs stop_services from start-blueos-core as the main process of a fake container: a pane child that either
# ignores SIGTERM or stops on it, a process the main process inherited (like ArduSub, re-parented to PID 1), and a
# stand-in for the tmux server, which must outlive the services because its death hangs up their panes. The main
# process is a subreaper, like PID 1, so a pane child that leaves a process of its own session behind when it stops
# (like the autopilot manager leaving ArduSub) orphans it to the main process in the middle of the stop.
run_stop_services() {
    local folder=$1
    local pane_child_ignores_sigterm=$2
    local pane_child_leaves_an_orphan=${3:-false}
    mkdir -p "$folder"
    : > "$folder/events"
    cat > "$folder/stubborn" <<EOF
#!/usr/bin/env bash
trap '' TERM
echo started >> "$folder/events"
while true; do sleep 0.1; done
EOF
    cat > "$folder/watched" <<EOF
#!/usr/bin/env bash
trap 'echo "\$1 terminated" >> "$folder/events"; exit 0' TERM
echo "\$1 started" >> "$folder/events"
while true; do sleep 0.1; done
EOF
    cat > "$folder/slow" <<EOF
#!/usr/bin/env bash
trap 'echo "\$1 terminated" >> "$folder/events"; turns_left=5' TERM
echo "\$1 started" >> "$folder/events"
while true; do
    sleep 0.1 &
    wait \$! || true
    if [ -n "\${turns_left:-}" ]; then
        turns_left=\$((turns_left - 1))
        [ "\$turns_left" -gt 0 ] || exit 0
    fi
done
EOF
    cat > "$folder/orphaning" <<EOF
#!/usr/bin/env bash
trap 'echo "pane_child terminated" >> "$folder/events"; setsid "$folder/watched" late_orphan & exit 0' TERM
echo "pane_child started" >> "$folder/events"
while true; do sleep 0.1; done
EOF
    chmod +x "$folder/stubborn" "$folder/watched" "$folder/slow" "$folder/orphaning"
    local pane_child="$folder/watched pane_child"
    [ "$pane_child_ignores_sigterm" = true ] && pane_child="$folder/stubborn"
    [ "$pane_child_leaves_an_orphan" = true ] && pane_child="$folder/orphaning"
    cat > "$folder/main_process" <<EOF
#!/usr/bin/env bash
eval "\$(sed -n '/^function stop_services {/,/^}/p' "$ROOT_DIR/core/start-blueos-core")"
bash -c '$pane_child & wait' &
pane_pid=\$!
"$folder/watched" tmux_server &
tmux_server_pid=\$!
setsid "$folder/slow" inherited &
tmux() {
    case \$1 in
        list-panes) echo "\$pane_pid" ;;
        display-message) echo "\$tmux_server_pid" ;;
    esac
}
while [ "\$(wc -l < "$folder/events")" -lt 3 ]; do sleep 0.1; done
STOP_TIMEOUT_SECONDS=2 stop_services
EOF
    exit_status=0
    elapsed_seconds=$SECONDS
    timeout 6 python3 -c 'import ctypes, os, sys; ctypes.CDLL(None).prctl(36, 1); os.execvp(sys.argv[1], sys.argv[1:])' \
        bash "$folder/main_process" > "$folder/log" 2>&1 || exit_status=$?
    elapsed_seconds=$((SECONDS - elapsed_seconds))
}

test_stop_services_kills_a_service_that_ignores_sigterm() {
    local folder="$WORK_DIR/stubborn_service"
    run_stop_services "$folder" true
    [ "$exit_status" -eq 0 ] || fail "stop_services did not exit 0 (status $exit_status)"
    [ "$elapsed_seconds" -le 3 ] || fail "stop_services took ${elapsed_seconds}s with a 2s deadline"
    grep -q "BlueOS stopped!" "$folder/log" || fail "stop_services did not log that BlueOS stopped"
    ! pgrep -f "bash $folder/stubborn" > /dev/null || fail "the service that ignores SIGTERM is still running"
}

test_stop_services_terminates_inherited_processes_but_not_the_tmux_server() {
    local folder="$WORK_DIR/inherited"
    run_stop_services "$folder" false
    [ "$exit_status" -eq 0 ] || fail "stop_services did not exit 0 (status $exit_status)"
    grep -q "pane_child terminated" "$folder/events" || fail "the pane child did not get SIGTERM"
    grep -q "inherited terminated" "$folder/events" || fail "the process inherited by the main process got no SIGTERM"
    ! grep -q "tmux_server terminated" "$folder/events" || fail "the tmux server got SIGTERM"
    [ "$(grep -c "inherited terminated" "$folder/events")" -eq 1 ] ||
        fail "the inherited process that is slow to exit got SIGTERM more than once"
}

test_stop_services_terminates_a_process_orphaned_during_the_stop() {
    local folder="$WORK_DIR/orphaned"
    run_stop_services "$folder" false true
    [ "$exit_status" -eq 0 ] || fail "stop_services did not exit 0 (status $exit_status)"
    grep -q "late_orphan terminated" "$folder/events" || fail "the process orphaned during the stop got no SIGTERM"
    ! grep -q "tmux_server terminated" "$folder/events" || fail "the tmux server got SIGTERM"
}

main() {
    test_sigterm_stops_the_service_cleanly_without_restart
    test_restarts_a_service_that_exits_zero
    test_stop_services_kills_a_service_that_ignores_sigterm
    test_stop_services_terminates_inherited_processes_but_not_the_tmux_server
    test_stop_services_terminates_a_process_orphaned_during_the_stop
    printf 'run_service_test: ok\n'
}

main "$@"
