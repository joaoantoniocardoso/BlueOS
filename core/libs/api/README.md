# blueos-api

Key layout and encoding helpers for the versioned BlueOS zenoh API (D-07, D-10, D-12). No zenoh
dependency by default.

## Key layout

All public keys are under `blueos/v1/<service>/...`:

| Helper | Key pattern |
|---|---|
| `command_key` | `blueos/v1/<service>/command/<name>` |
| `query_key` | `blueos/v1/<service>/query/<name>` |
| `state_key` | `blueos/v1/<service>/state/<name>` |
| `event_key` | `blueos/v1/<service>/event/<name>` |
| `jobs_key` | `blueos/v1/<service>/jobs` |
| `settings_key` | `blueos/v1/<service>/settings` |
| `log_key` | `blueos/v1/<service>/log` |
| `service_liveliness_key` | `blueos/v1/services/<name>` |
| `service_info_key` | `blueos/v1/services/<name>/info` |

Standard per-service state: `status_state_key`, `info_query_key`.

## Encoding

- Payload: plain CDR bytes (no framing). Use `blueos-idl` `Message::encode` / `decode`.
- Zenoh encoding: `cdr_encoding(schema_name)` -> `application/cdr;<schema_name>` (e.g.
  `application/cdr;blueos_msgs/msg/CommandAck`).
- Type hash attachment key: `TYPE_HASH_ATTACHMENT_KEY` (`blueos.type_hash`).

## Schema evolution

Summarized in `blueos-idl` README (D-06): append-only fields within a major version; bump major for
breaking changes; lock file in `core/interfaces/api.lock`.

## Shared memory (D-09)

Zenoh may use host shared memory for large payloads when both peers support it. **Extensions need
`IpcMode: host`** in Kraken permissions (or a narrower bind-mount of `/dev/shm`) to participate in
zero-copy SHM. Without that, traffic silently falls back to TCP loopback. Core services already bind
host `/dev/` and share SHM.
