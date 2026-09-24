# Recorder (Rust)

Records the Zenoh backbone into MCAP files under `--recorder-path` (default legacy layout:
`recorder_YYYYMMDD_HHMMSS.mcap` in that directory). Python `recorder_extractor` reads the same path.

## What gets recorded

| Topic prefix | When |
|---|---|
| `mavlink/`, `mavlink_raw/` | While the vehicle is armed (configurable via settings) |
| `video/...` | While MCM external capture is active for that stream |
| Everything else | While a recording session is active |

MAVLink on `mavlink_raw/out` is always parsed for policy; only gated topics are written to MCAP.

## MCM contract

With `mavlink-camera-manager --recorder=external`, the recorder answers MAVLink video capture commands,
publishes capture status on `mavlink_raw/in`, and gates `video/<stream>/stream` topics during capture.

## Control vs data plane

Policy (armed state, video stream recording flags, session lifecycle, capture command decisions) lives in
`logic/policy` and is driven through the service inbox. The data plane subscribes to `**`, reads a
[`TapPolicy`] snapshot from a watch channel updated when state is published, and writes allowed samples to
MCAP without sending payloads through the inbox (D-03, D-09).

## Zero-copy and SHM

Samples are passed as `blueos_comms::Payload` clones to the MCAP writer thread. Zenoh uses shared memory
when both peers are on the same host and payloads are large enough; without host IPC, traffic falls back
to TCP silently.

The core container already bind-mounts `/dev/` (see `bootstrap/startup.json.default`), so `/dev/shm` is
shared with the host. **Extension images** must set Kraken permissions accordingly, for example:

```json
"HostConfig": {
  "IpcMode": "host"
}
```

The narrower alternative is a bind mount of `/dev/shm`. See also `core/libs/api/README.md` and
`doc/architecture/decisions.md` (D-09). External extension developer docs should repeat this requirement.

## Docker image binary (local builds)

CI cross-builds the multicall `blueos` binary with the `recorder` feature. To build the core image
locally, produce musl binaries under `core/target/build/` first:

```bash
cd core
./build_cross.sh
```

One target only (matches your machine or the platform you build with buildx):

```bash
cd core
TARGETS='x86_64-unknown-linux-musl' ./build_cross.sh
```

Binaries land at `core/target/build/<target>/<target>/release/blueos`. The Dockerfile copies the file
matching `TARGETARCH` into `/usr/bin/blueos` and adds `/usr/bin/recorder` as a symlink to it.
