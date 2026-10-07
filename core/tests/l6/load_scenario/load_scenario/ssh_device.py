import subprocess
from pathlib import Path

from load_scenario.sampling import ResourceSample, parse_remote_sample_line

_REMOTE_SAMPLE_SCRIPT = """\
set -euo pipefail
state_file="/tmp/load_scenario_cpu.state"
read_stat() {
  awk '/^cpu / {
    idle=$5+$6; total=0
    for (i=2; i<=NF; i++) { total+=$i }
    print idle, total
  }' /proc/stat
}
current_stat="$(read_stat)"
system_cpu="0"
if [ -f "${state_file}" ]; then
  read -r previous_idle previous_total < "${state_file}"
  read -r current_idle current_total <<< "${current_stat}"
  idle_delta=$((current_idle - previous_idle))
  total_delta=$((current_total - previous_total))
  if [ "${total_delta}" -gt 0 ]; then
    system_cpu="$(awk -v idle="${idle_delta}" -v total="${total_delta}" \\
      'BEGIN { printf "%.2f", (1 - idle / total) * 100 }')"
  fi
fi
echo "${current_stat}" > "${state_file}"
process_line() {
  local name="$1"
  local pattern="$2"
  local process_id
  process_id="$(pgrep -xo "${pattern}" || true)"
  if [ -z "${process_id}" ]; then
    echo "${name}:0:0"
    return
  fi
  local cpu memory
  cpu="$(ps -p "${process_id}" -o %cpu= | tr -d ' ')"
  memory="$(awk '/^RssAnon:/ {print $2}' /proc/${process_id}/status | head -1)"
  echo "${name}:${cpu}:${memory}"
}
printf '%s ' \
  "$(process_line recorder recorder)" \
  "$(process_line mavlink_camera_manager mavlink-camera-manager)" \
  "$(process_line zenohd zenohd)" \
  "system:${system_cpu}:0"
"""


class SshDevice:
    def __init__(self, host: str, user: str = "pi") -> None:
        self._target = f"{user}@{host}"

    def run(self, remote_command: str, timeout_seconds: float = 60.0) -> str:
        completed = subprocess.run(
            [
                "ssh",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                self._target,
                remote_command,
            ],
            check=True,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
        )
        return completed.stdout

    def read_cpu_governor(self) -> str:
        return self.run(
            "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown",
            timeout_seconds=10.0,
        ).strip()

    def set_cpu_governor_performance(self) -> None:
        self.run(
            "echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor >/dev/null",
            timeout_seconds=15.0,
        )

    def read_throttled(self) -> str:
        return self.run("vcgencmd get_throttled 2>/dev/null || echo throttled=0x0", timeout_seconds=10.0).strip()

    def collect_resource_sample(self) -> ResourceSample:
        output = self.run(_REMOTE_SAMPLE_SCRIPT, timeout_seconds=15.0)
        return parse_remote_sample_line(output)

    def copy_file_from_device(self, remote_path: str, local_path: Path) -> None:
        subprocess.run(
            [
                "scp",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                f"{self._target}:{remote_path}",
                str(local_path),
            ],
            check=True,
            timeout=600.0,
        )

    def collect_resource_samples(
        self,
        sample_count: int,
        interval_seconds: float,
    ) -> list[ResourceSample]:
        self.run("rm -f /tmp/load_scenario_cpu.state", timeout_seconds=10.0)
        samples: list[ResourceSample] = []
        for _ in range(sample_count):
            samples.append(self.collect_resource_sample())
            if interval_seconds > 0 and len(samples) < sample_count:
                subprocess.run(
                    [
                        "ssh",
                        "-o",
                        "BatchMode=yes",
                        self._target,
                        f"sleep {interval_seconds}",
                    ],
                    check=True,
                    timeout=interval_seconds + 30.0,
                )
        return samples
