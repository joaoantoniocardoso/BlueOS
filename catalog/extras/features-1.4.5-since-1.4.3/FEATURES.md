# Catalog features: 1.4.5 since 1.4.3

## Scope

Tags and SHAs (`presence.json` `tag_shas`):

| Tag | SHA |
|-----|-----|
| 1.4.3 | `c38aa1eefe0f03936f7f42ea87a9f8d42a5bbf3f` |
| 1.4.4 | `168e82a7a71212b674a7576d854a88b363cd7879` |
| 1.4.4-beta.23 | `168e82a7a71212b674a7576d854a88b363cd7879` |
| 1.4.5 | `14722557d0b8af7907045eee43dfb9e9392c7728` |

Note: 1.4.4 and 1.4.4-beta.23 share the same SHA.

Method: catalog journeys. A journey is present on a tag when `intro_commit` is an ancestor of the tag commit, or the journey `source_path` exists on that tag.

Inventory: 100 catalog journeys total.

## Added in 1.4.5 since 1.4.3

- **Operator levels the horizon by placing the vehicle on a level surface and running board-level calibration** (`level_horizon`, frontend_calibration, 06490f90ed01, path / absent)

## Unchanged count

80 journeys were already present on 1.4.3 and remain present on 1.4.5.

## Not in 1.4.5 (catalog-only / 1.5+)

These 19 inventory journeys are absent on 1.4.5. They are not part of the 1.4.5 feature set.

### commander

- `reset_blueos_settings` (first_tag: 1.5.0-beta.30)

### customization

- `change_ui_theme_color` (first_tag: 1.5.0-beta.38)
- `delete_3d_model_override` (first_tag: 1.5.0-beta.38)
- `remove_custom_logo` (first_tag: 1.5.0-beta.38)
- `remove_custom_vehicle_image` (first_tag: 1.5.0-beta.38)
- `reset_ui_theme_color` (first_tag: 1.5.0-beta.38)
- `upload_3d_model_override` (first_tag: 1.5.0-beta.38)
- `upload_custom_logo` (first_tag: 1.5.0-beta.38)
- `upload_custom_vehicle_image` (first_tag: 1.5.0-beta.38)

### disk_usage

- `free_disk_space` (first_tag: 1.5.0-beta.23)
- `inspect_disk_usage` (first_tag: 1.5.0-beta.23)
- `run_multi_size_disk_speed_test` (first_tag: 1.5.0-beta.23)
- `run_single_disk_speed_test` (first_tag: 1.5.0-beta.23)

### recorder_extractor

- `browse_video_recordings` (first_tag: 1.5.0-beta.22)
- `delete_video_recording` (first_tag: 1.5.0-beta.22)
- `download_video_recording` (first_tag: 1.5.0-beta.22)

### versionchooser

- `delete_local_blueos_version` (first_tag: 1.5.0-beta.9)
- `update_bootstrap_image` (first_tag: 1.5.0-beta.9)

### zenohd

- `inspect_zenoh_network` (first_tag: 1.5.0-beta.2)

## Cross-check

Added set is equal across 1.4.4-beta.23, 1.4.4, and 1.4.5 (`sets_equal`: true). Removed journeys: none.
