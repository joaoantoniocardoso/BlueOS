# Catalog functions: 1.4.3 since 1.4.2

## Scope

Functions are ActionCatalog Action requirements (kind==functional) from
catalog/requirements-baselines/1.5.0-beta.40.json (108 total).

A function is present on a tag when ANY verifying_journey is present on that tag.
Journey presence comes from live git verification in presence.json
(intro_commit ancestor and/or source_path on tag SHA).

Tags and SHAs (presence.json tag_shas):

| Tag | SHA |
|-----|-----|
| 1.4.2 | `0a0043f3f347c1663e11a32a61c723e9f79b675e` |
| 1.4.3 | `c38aa1eefe0f03936f7f42ea87a9f8d42a5bbf3f` |

Note: the catalog is incomplete. Git tree, commits, and shipped binaries are required for a full product inventory.

## Added in 1.4.3 since 1.4.2

None. Catalog function presence is unchanged between 1.4.2 and 1.4.3.

## Unchanged count

87 functions were already present on 1.4.2 and remain present on 1.4.3.

## Not in 1.4.3 (catalog-only / 1.5+)

These 20 catalog functions are absent on 1.4.3.
They are not part of the 1.4.3 function set.

### commander

- `reset_blueos_settings/54288da215af898a` (reset_blueos_settings, first_tag: 1.5.0-beta.30)

### customization

- `delete_model_override` (delete_3d_model_override, first_tag: 1.5.0-beta.38)
- `remove_branding_logo` (remove_custom_logo, first_tag: 1.5.0-beta.38)
- `remove_branding_vehicle_image` (remove_custom_vehicle_image, first_tag: 1.5.0-beta.38)
- `reset_theme_color` (reset_ui_theme_color, first_tag: 1.5.0-beta.38)
- `set_theme_color` (change_ui_theme_color, first_tag: 1.5.0-beta.38)
- `upload_branding_logo` (upload_custom_logo, first_tag: 1.5.0-beta.38)
- `upload_branding_vehicle_image` (upload_custom_vehicle_image, first_tag: 1.5.0-beta.38)
- `upload_model_override` (upload_3d_model_override, first_tag: 1.5.0-beta.38)

### disk_usage

- `delete_disk_paths` (free_disk_space, first_tag: 1.5.0-beta.23)
- `inspect_disk_usage/66077bb16c8f3165` (free_disk_space, inspect_disk_usage, first_tag: 1.5.0-beta.23)
- `navigate_disk_usage` (inspect_disk_usage, first_tag: 1.5.0-beta.23)
- `run_disk_speed_test` (run_single_disk_speed_test, first_tag: 1.5.0-beta.23)
- `run_multi_size_disk_speed_test/9638e1657f9e12b6` (run_multi_size_disk_speed_test, first_tag: 1.5.0-beta.23)

### recorder_extractor

- `browse_video_recordings/ab18b05457a69ba2` (browse_video_recordings, first_tag: 1.5.0-beta.22)
- `delete_video_recording/6e06c4b5bcae5a00` (delete_video_recording, first_tag: 1.5.0-beta.22)
- `download_video_recording/6e06c4b5bcae5a00` (download_video_recording, first_tag: 1.5.0-beta.22)

### versionchooser

- `delete_local_blueos_version/b6a758e879c27c5e` (delete_local_blueos_version, first_tag: 1.5.0-beta.9)
- `update_bootstrap_image/d7a8583bc58eea88` (update_bootstrap_image, first_tag: 1.5.0-beta.9)

### zenohd

- `inspect_zenoh_network/cbf29ce484222325` (inspect_zenoh_network, first_tag: 1.5.0-beta.2)
