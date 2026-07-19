# Polish QA — post narrow-widen re-enrich

## InspectZenohNetwork
- merge_method=`rebase` squash_merge=`False`
- landing=[3300]
- backports=[]
- follow_ups=[3313, 3368, 3376, 3381, 3386, 3387, 3391, 3392, 3393, 3407, 3635, 3820, 3953]
- issues=[]
- discovery_paths=['core/frontend/src/components/zenoh-inspector', 'core/frontend/src/components/zenoh-inspector/ZenohInspector.vue', 'core/frontend/src/views/ZenohInspectorView.vue']

## ChangeUiThemeColor
- merge_method=`rebase` squash_merge=`False`
- landing=[3930]
- backports=[]
- follow_ups=[]
- issues=[]
- discovery_paths=['core/frontend/src/components/customization', 'core/frontend/src/components/customization/BrandingUploader.vue', 'core/frontend/src/components/customization/ThemeCustomization.vue', 'core/frontend/src/store/customization.ts', 'core/frontend/src/types/customization.ts', 'core/services/customization', 'core/services/customization/main.py', 'core/services/customization/storage.py', 'core/services/customization/theme.py']

## InspectDiskUsage
- merge_method=`rebase` squash_merge=`False`
- landing=[3669]
- backports=[]
- follow_ups=[3681, 3691, 3743]
- issues=[2572]
- discovery_paths=['core/frontend/src/store/disk.ts', 'core/frontend/src/types/disk.ts', 'core/frontend/src/views/Disk.vue', 'core/services/disk_usage', 'core/services/disk_usage/main.py']

## RunInternetSpeedTest
- merge_method=`rebase` squash_merge=`False`
- landing=[3602]
- backports=[]
- follow_ups=[3758]
- issues=[2146]
- discovery_paths=['core/frontend/src/store/pardal.ts', 'core/frontend/src/types/pardal.ts', 'core/services/pardal', 'core/services/pardal/main.py']

## LevelHorizon
- merge_method=`rebase` squash_merge=`False`
- landing=[3826]
- backports=[3867]
- follow_ups=[3874, 3962]
- issues=[3873, 3958]
- discovery_paths=['core/frontend/src/components/vehiclesetup/configuration/accelerometer', 'core/frontend/src/components/vehiclesetup/configuration/accelerometer/ArdupilotAccelerometerSetup.vue', 'core/frontend/src/components/vehiclesetup/configuration/compass', 'core/frontend/src/components/vehiclesetup/configuration/compass/LevelHorizonCalibration.vue']

Cluster merge_method counts: `{'rebase': 27, 'merge_commit': 3}` (squash_true=0)
