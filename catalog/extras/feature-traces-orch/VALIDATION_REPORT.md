# Feature traces validation report

generated_at: `2026-07-19T13:52:32-03:00`
schema_version: **2**
journeys: 94 | clusters: 30 | commits: 2506 | PRs: 750 | issues: 197
file size: 3138796 bytes

## Gap results

1. **Backports**: PASS (machinery) — 14/30 clusters non-empty; title-filtered
   - `06490f90ed01` → [3867] paths=['core/frontend/src/components/vehiclesetup/configuration/accelerometer/ArdupilotAccelerometerSetup.vue', 'core/frontend/src/components/vehiclesetup/configuration/compass/LevelHorizonCalibration.vue']
   - `11088c7b8afe` → [3473, 3867, 3875] paths=['core/frontend/public/img/icons/motordetection.svg', 'core/frontend/src/components/common/StatusTextWatcher.vue', 'core/frontend/src/components/utils/themedSVG.vue', 'core/frontend/src/components/vehiclesetup/MotorDetection.vue', 'core/frontend/src/components/vehiclesetup/PwmSetup.vue', 'core/frontend/src/libs/firmware/ardupilot/ardusub.ts', 'core/frontend/src/utils/ardupilot_mavlink.ts']
   - `22025de9a5d7` → [3867] paths=['core/frontend/src/components/parameter-editor/InlineParameterEditor.vue', 'core/frontend/src/components/vehiclesetup/Configure.vue', 'core/frontend/src/components/vehiclesetup/overview/GyroCalib.vue']
   - `2f8d657c6cd4` → [3867] paths=['core/frontend/src/components/vehiclesetup/configuration/compass/ArdupilotMavlinkCompassSetup.vue', 'core/frontend/src/components/vehiclesetup/configuration/compass/AutoCoordinateDetector.vue', 'core/frontend/src/components/vehiclesetup/configuration/compass/CalibrationQualityIndicator.vue', 'core/frontend/src/components/vehiclesetup/configuration/compass/CompassMaskPicker.vue', 'core/frontend/src/components/vehiclesetup/configuration/compass/FullCompassCalibrator.vue']
   - `40c0bac44abd` → [3867, 3895] paths=['core/frontend/src/ArduPilot-Parameter-Repository', 'core/frontend/src/components/parameter-editor/Parameter.ts', 'core/frontend/src/components/parameter-editor/ParameterEditor.vue', 'core/frontend/src/components/parameter-editor/ParameterTable.ts', 'core/frontend/src/libs/MAVLink2Rest/index.ts', 'core/frontend/src/store/autopilot.ts', 'core/frontend/src/types/autopilot/parameter-fetcher.ts', 'core/frontend/src/types/autopilot/parameter-table.ts', 'core/frontend/src/types/autopilot/parameter.ts', 'core/frontend/src/views/ParameterEditorView.vue']
   - `5de1feb47221` → [3434] paths=['core/services/ping/bridges.py', 'core/services/ping/main.py', 'core/services/ping/ping1d_driver.py', 'core/services/ping/ping360_driver.py', 'core/services/ping/pingdriver.py', 'core/services/ping/pingmanager.py', 'core/services/ping/pingprober.py', 'core/services/ping/pingutils.py', 'core/services/ping/portwatcher.py', 'core/services/ping/serialhelper.py', 'core/services/ping/setup.py']
   - `635536507e69` → [3466, 3505, 3616, 3894] paths=['core/services/cable_guy/README.md', 'core/services/cable_guy/api/ethernet.py', 'core/services/cable_guy/api/manager.py', 'core/services/cable_guy/api/settings.py', 'core/services/cable_guy/html/index.html', 'core/services/cable_guy/main.py', 'core/services/cable_guy/setup.py', 'core/services/cable_guy/swagger/cable-guy.yaml']
   - `7c62889397a3` → [2534, 3495] paths=['core/services/versionchooser/main.py', 'core/services/versionchooser/openapi/versionchooser.yaml', 'core/services/versionchooser/setup.py', 'core/services/versionchooser/static/index.html', 'core/services/versionchooser/static/style.css', 'core/services/versionchooser/test_versionchooser.py', 'core/services/versionchooser/utils/chooser.py', 'core/services/versionchooser/utils/dockerhub.py']
2. **Follow-ups**: PASS — 28/30 clusters, 723 PR links
3. **Squash fields**: PASS on all clusters
4. **files_changed**: PASS — 30/30 landing PRs have files
5. **PR bodies**: PASS — 29/30 landing PRs have body
6. **Issues+sources**: PASS — 27/30 clusters; kinds={'closing': 250, 'body': 18, 'timeline': 12, 'search': 1}
7. **Structure**: PASS — indexed commits/PRs/issues/clusters; journeys ref intro_commit only
   missing_landing: none

## Spot checks
### InspectZenohNetwork
- landing #3300 — Add initial frontend zenoh integration
- discovery_paths: ['core/frontend/src/components/zenoh-inspector/ZenohInspector.vue', 'core/frontend/src/views/ZenohInspectorView.vue']
- backports: []
- follow_ups (8): [3313, 3368, 3376, 3386, 3407, 3635, 3820, 3953]
- squash_merge=True intro_in_pr=False merge=083f5e2b0a03
- intro_files=1 pr_files=9 body_len=784
- issues: []

### ChangeUiThemeColor
- landing #3930 — Add BlueOS cutomization
- discovery_paths: ['core/frontend/src/components/customization/BrandingUploader.vue', 'core/frontend/src/components/customization/ThemeCustomization.vue', 'core/frontend/src/store/customization.ts', 'core/frontend/src/types/customization.ts', 'core/services/customization/main.py', 'core/services/customization/storage.py', 'core/services/customization/theme.py']
- backports: []
- follow_ups (0): []
- squash_merge=True intro_in_pr=False merge=f40daaa03043
- intro_files=4 pr_files=19 body_len=1471
- issues: [(3426, ['search'])]

### InspectDiskUsage
- landing #3669 — core: Add disk-usage service
- discovery_paths: ['core/frontend/src/store/disk.ts', 'core/frontend/src/types/disk.ts', 'core/frontend/src/views/Disk.vue', 'core/services/disk_usage/main.py']
- backports: []
- follow_ups (3): [3681, 3691, 3743]
- squash_merge=True intro_in_pr=False merge=24baaf7bf8ff
- intro_files=5 pr_files=11 body_len=812
- issues: [(2572, ['timeline'])]

### RunInternetSpeedTest
- landing #3602 — core: move internet speed test to pardal
- discovery_paths: ['core/frontend/src/store/pardal.ts', 'core/frontend/src/types/pardal.ts', 'core/services/pardal/main.py']
- backports: []
- follow_ups (1): [3686]
- squash_merge=True intro_in_pr=False merge=757ce3c2f3dd
- intro_files=8 pr_files=8 body_len=593
- issues: [(2146, ['closing', 'body'])]

### LevelHorizon
- landing #3826 — Frontend: create LevelHorizonCalibration
- discovery_paths: ['core/frontend/src/components/vehiclesetup/configuration/accelerometer/ArdupilotAccelerometerSetup.vue', 'core/frontend/src/components/vehiclesetup/configuration/compass/LevelHorizonCalibration.vue']
- backports: [3867]
- follow_ups (0): []
- squash_merge=True intro_in_pr=False merge=06490f90ed01
- intro_files=2 pr_files=2 body_len=530
- issues: []

