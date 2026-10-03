# blueos-api

Key layout and encoding helpers for the versioned BlueOS zenoh API (D-07, D-10, D-12). No zenoh
dependency.

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

Standard per-service state: `status_state_key`, `info_query_key`.

## CommandAck

Commands are Zenoh queries that submit a Job or control one, replied with
`CommandAck { accepted, job_id, status, reason }` (D-10, D-36). Use `blueos_api::CommandAck` (from `blueos-idl`) and
`Message::encode` / `decode`.

- The query attachment is the Job id, the UUID text the client generated: the new Job for a submit, the Job to
  control for `CancelJob`, `PauseJob`, `ResumeJob` and `AnswerPermission`.
- `accepted`: whether the service took the command.
- `job_id`: the Job id from the attachment, empty when the Command named no Job.
- `status`: the Job's status after the command was applied, so a Job that ended at once returns its final status.
- `reason`: human-readable detail (empty when none).

Zenoh encoding: `cdr_encoding(CommandAck::SCHEMA_NAME)`.

## Encoding

- Payload: plain CDR bytes (no framing). Use `blueos-idl` `Message::encode` / `decode`.
- Zenoh encoding: `cdr_encoding(schema_name)` -> `application/cdr;<schema_name>` (e.g.
  `application/cdr;blueos_msgs/msg/CommandAck`).
- Type hash attachment key: `TYPE_HASH_ATTACHMENT_KEY` (`blueos.type_hash`).

## Schema evolution

Summarized in `blueos-idl` README (D-06): append-only fields within a major version; bump major for
breaking changes; lock file in `core/libs/idl/api.lock`.

## Shared memory (D-09)

Zenoh may use host shared memory for large payloads when both peers support it. **Extensions need
`IpcMode: host`** in Kraken permissions (or a narrower bind-mount of `/dev/shm`) to participate in
zero-copy SHM. Without that, traffic silently falls back to TCP loopback. Core services already bind
host `/dev/` and share SHM.
