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

Samples are passed as `blueos_comms::Payload` clones to the MCAP writer thread. For shared-memory Zenoh
between MCM and the recorder, use host IPC (`IpcMode: host` or `/dev/shm` bind) as documented in D-09.
