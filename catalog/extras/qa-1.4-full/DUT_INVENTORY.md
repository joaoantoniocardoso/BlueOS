# DUT inventory — live 2026-08-13 (W0)

Core (HTTP `/version-chooser/v1.0/version/current` on all four):

`bluerobotics/blueos-core:1.4-dev @ sha256:f615d7caef4d3e99f1c068e082350c1af43d5fcc0f45379ed97ea27c6dc89805`

| DUT | Pi | board | wlan | cameras | ping | pirate | disk free | blockers |
|---|---|---|---|---|---|---|---|---|
| 192.168.0.177 | Pi4 B 1.5 | Navigator | present, disconnected | 1 USB, 0 MCM streams | ABSENT | pirate=true | 106G | disk-usage :9151 down; FC heartbeat empty |
| 192.168.0.87 | Pi4 | **Pixhawk1** `/dev/ttyACM0` ArduPilot 4.7.0 | present, DOWN | USB H264 + MCM stream `udp://192.168.2.1:5600` | ABSENT | bag pirate ABSENT | 7.9G | `/serials` 500; mavlink2rest nginx 404; wlan inactive |
| 192.168.2.2 | Pi4 B 1.2 | Navigator ArduSub 4.7.0 | present, DOWN | no local V4L2; RadCam ONVIF idle | ABSENT | bag settings 404 | 106G | USB mgmt eth0; **do not strand**; FC heartbeat empty; throttled history 0x50000 |
| 192.168.0.124 | **Pi5** | Navigator 4.7.0 STABLE | wlan0+uap0 DOWN; hotspot off | ABSENT | ABSENT | pirate on | 96G | disk-usage :9151 down; mavlink2rest nginx 404; FC heartbeat OK |

Per-DUT dumps: `inventory/<ip>.md`. SSH `pi`/`raspberry`.
