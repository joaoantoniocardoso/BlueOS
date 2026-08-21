# H4 Oracle Classifier counts

Derived from `derive_oracle_class` after dropping `journey_has_frontend_capability` trigger.

| OracleClass | Count |
|-------------|-------|
| ClientOrchestrated | 30 |
| ClientComposed | 47 |
| HttpPassthrough | 23 |
| other | 0 |

**TRIGGER3 (`frontend_capability_def` on `capability_refs`): DROP** — all 11 frontend-origin
capabilities appear only on journeys that already have `Actor::Frontend` steps; redundant with
plan's `Actor::Frontend` trigger and would not change any tally.

**Active ClientOrchestrated triggers:** `Actor::Frontend` | page `frontend_features` (non-empty).

## ClientOrchestrated missing `ui_plan` (not in `PAGE_LOAD_UI`)

17 ids — mostly `ardupilot_manager` / `helper` journeys on VehicleSetup page with
`frontend_features`:

- VehicleFirstBoot, ChangeBoard, RunSitlSimulation, StartAutopilot, StopAutopilot,
  RestartAutopilot, UpdateFirmwareOnline, UploadCustomFirmware, RestoreDefaultFirmware,
  RebootOnboardComputer, ShutdownOnboardComputer, SyncSystemTime, EnableLegacyCameraSupport,
  InspectRaspberryEepromBootloader, UpdateRaspberryEepromBootloader, ResetBlueosSettings,
  RunHostCommand
