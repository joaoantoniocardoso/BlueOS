# Tier-2 mutating smoke — restore toolkit

Play Pi (`192.168.0.177`) is owned test hardware. Reimage is acceptable. Tier-2 grows by **declared restore**, not by avoiding mutation.

## Rule

Every `--mutating-smoke` allowlist entry must name how state returns to a known-good baseline after the mutate step(s). Prefer the lightest restore that works.

**Preconditions are setup, not only skip gates.** The same repair toolkit may **provision** missing preconditions before a journey runs (install an extension, seed a recording, enable pirate/advanced, select a board, pull a local image, start a sock/stream, etc.). Skip only when provisioning is impossible or out of scope for that entry. Typed `FixtureInventory` remains the declaration of “what this run provides”; runners may populate it by repairing the Pi rather than requiring the human to have pre-staged everything.

## Allowed restore / setup mechanisms (preferred order)

1. **HTTP round-trip** — GET snapshot → mutate → PUT/POST restore; or create → destroy pair (theme, branding, NMEA sock, manifest).
2. **Filesystem** — download/copy config from container or host, mutate via API, re-upload/replace files under `/root/.config/blueos`, `/usr/blueos/userdata`, etc.
3. **Service tmux restart** — attach service session, Ctrl-C, Up, Enter (reload process without full container bounce).
4. **Container restart** — `docker restart blueos-core` (or equivalent); wait for nginx + health.
5. **Host reboot** — reboot Pi; wait until BlueOS HTTP is back; resume smoke.
6. **External dependency proxy** — cache/serve Docker Hub and similar so version/extension pulls do not require live upstream.

## Hard exclude

- **Shutdown / power-off** with no automated wake path.

## Allowlist entry shape (target)

Future runner work should encode roughly:

```text
MutatingSmokeEntry {
  journey_id,
  setup: optional provisioning of preconditions (same toolkit as restore),
  mutate steps (+ bodies / path bindings),
  restore: HttpPair | SnapshotRestore | TmuxRestart | ContainerRestart | HostReboot | FilesystemReplace | ProxyBackedPull,
}
```

Until that type exists, document setup + restore in the allowlist decision log / comments when adding journeys.

## Not a RouteRef commit gate

Tier-1 GET `--smoke` remains the blocking gate. Tier-2 stays optional deeper coverage.
