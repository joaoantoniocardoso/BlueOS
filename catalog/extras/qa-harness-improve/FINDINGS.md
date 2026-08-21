# Findings catalog — QA harness improvement H7 live

Append-only. Supersede with `H7-F-NNN.1` if a re-run changes the result. Never delete.

DUT: `192.168.0.177` (`bluerobotics/blueos-core:1.4-dev` digest `sha256:5b50dfafc3114651d459993c0c65639d7205634613fb62e0dea5ee04de8eebb1`)

| id | dut | journey / probe | kind | severity | expected vs got | restore | next |
|---|---|---|---|---|---|---|---|
| H7-F-001 | 177 | update_blueos_version POST /version/pull | CatalogWrongStatus | minor | catalog HTTP 200; live 308 Permanent Redirect on version-chooser | n/a | trailing-slash or nginx redirect; catalog status |
| H7-F-002 | 177 | pull_blueos_version_without_switch POST /version/pull | CatalogWrongStatus | minor | catalog HTTP 200; live 308 | n/a | same as H7-F-001 |
| H7-F-003 | 177 | docker_registry_login POST /docker/login | CatalogWrongStatus | minor | catalog HTTP 200; live 308 | n/a | same redirect pattern |
| H7-F-004 | 177 | switch_local_blueos_version POST /version/current | environment | major | core switch HTTP 200 or connection drop; live 412 Precondition Failed | core restore attempted | local-version fixture / digest mismatch on 1.4-dev pin |
| H7-F-005 | 177 | switch_local_blueos_version teardown | environment | major | core restore + delete local tag HTTP 200; restore 412, delete 404 | reboot followed | post-switch DUT state; report counts 39 fail after reboot window |
| H7-F-006 | 177 | ui pre-reboot | HarnessGap | note | board snapshot GET before Playwright; HTTP 0 during reboot window | n/a | defer UI until DUT stable after mutating-smoke reboot |
| H7-F-007 | 177 | level_horizon ui | ClientDesync | minor | UI expects calibration success; MAV_RESULT_TEMPORARILY_REJECTED | n/a | SITL busy/reject after prior cal steps |
| H7-F-008 | 177 | detect_motor_directions ui | ClientDesync | major | wait text "Motor direction detection is complete"; 120s timeout | n/a | vectored frame motor test did not finish in UI |
