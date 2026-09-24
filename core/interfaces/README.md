# BlueOS interface definitions (D-05)

ROS 2 `.msg` sources for the versioned BlueOS IDL live under `core/libs/idl/interfaces/` so they
ship inside the publishable `blueos-idl` crate (`cargo package` includes that tree).

Package directories here are symlinks into that tree. Edit messages there, then run:

```bash
BLUEOS_IDL_UPDATE_LOCK=1 cargo test -p blueos-idl api_lock_matches_interfaces -- --nocapture
```

The API-break lock file is `core/interfaces/api.lock` (D-06).
