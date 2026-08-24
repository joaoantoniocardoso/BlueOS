# P3a: journey preconditions -> assume-constraint members

Decision D4: no parallel RTM rows for deleted `CON/` requirement IDs. Preconditions
are `Requirement.assumptions` on functional and system requirements they gate.

## Before / after totals

| Tag / scope | Before total | Constraint rows | After total | Assumption members (filtered) |
|---|---:|---:|---:|---:|
| Unfiltered bootstrap | 527 | 42 | 485 | 213 |
| `1.4-dev` | 388 | 36 | 352 | 182 |
| `1.4.4-beta.21` | 388 | 36 | 352 | 182 |
| `1.5.0-beta.40` / `master` | 481 | 42 | 439 | 213 |

Unique journey precondition labels preserved: 42 unfiltered (36 on `1.4-dev` filter).

## Storage

`Requirement.assumptions: Vec<Assumption>` on `catalog/src/requirement.rs`.

```rust
pub struct Assumption {
    pub statement: String,
    pub source_use_case: String, // snake_case JourneyId
    pub kind: AssumptionKind,    // hardware | software | data | network | other
}
```

Attached after derivation: functional requirements from `requirement_verifications`
journeys; system requirements from functional children plus overlay `journey_ids`.

## Deleted requirement IDs (`kind=constraint`)

### `1.4-dev` and `1.4.4-beta.21` (36 each)

```
REQ/vehicle/autopilot/CON/Other::Cold start after power-on
REQ/vehicle/autopilot/CON/Other::BlueOS is newly installed and the configuration wizard is available
REQ/vehicle/autopilot/CON/Other::At least one flight controller board is connected or SITL simulation is available
REQ/vehicle/autopilot/CON/Other::Virtual SITL flight controller board is selected
REQ/vehicle/autopilot/CON/Other::A flight controller board is selected
REQ/vehicle/autopilot/CON/Other::An autopilot is available to stop
REQ/vehicle/autopilot/CON/Network::Online
REQ/vehicle/autopilot/CON/Other::A compatible flight controller board is connected
REQ/vehicle/autopilot/CON/Other::A flight controller board is connected
REQ/onboard_computer/files_kv/CON/Software::PirateMode
REQ/network/identity_discovery/CON/NetworkResource::WiredEthernetPresent
REQ/network/identity_discovery/CON/Other::BlueOS is connected via a wired connection so blueos.local is reachable
REQ/peripherals/serial_bridge/CON/Software::AdvancedMode
REQ/peripherals/serial_bridge/CON/Hardware::UsbSerialDevice
REQ/peripherals/serial_bridge/CON/Data::SerialBridgeConfigured
REQ/network/wired_network/CON/Other::The interface has at least one static IP address to use as the DHCP gateway
REQ/network/wired_network/CON/Data::OnboardDhcpServerActive
REQ/onboard_computer/host_control/CON/Software::ConfirmDangerousOp
REQ/vehicle/autopilot/CON/Other::vehicle is stationary while the gyro is calibrated
REQ/vehicle/autopilot/CON/Other::vehicle can be physically rotated through six orientations for full calibration
REQ/vehicle/autopilot/CON/Other::vehicle can be rotated about all axes for full onboard compass calibration
REQ/vehicle/autopilot/CON/Other::barometer calibration is performed at the start of each dive_flight
REQ/vehicle/autopilot/CON/Other::vehicle rests on a level surface in its normal operating orientation
REQ/vehicle/autopilot/CON/Other::vehicle is safe to arm and briefly spin its motors
REQ/peripherals/camera/CON/Hardware::UsbCamera
REQ/blueos_platform/extensions/CON/Data::ExtensionInstalled
REQ/peripherals/camera/CON/Other::At least one configured stream is listed on a device card
REQ/peripherals/gps_nmea/CON/Hardware::ExternalNmeaGps
REQ/peripherals/gps_nmea/CON/Data::NmeaSocketConfigured
REQ/peripherals/sonar/CON/HardwarePresent::Ping family sonar device
REQ/peripherals/sonar/CON/Hardware::Ping1d
REQ/blueos_platform/versioning/CON/Data::LocalBlueosVersionAvailable
REQ/network/wireless_network/CON/NetworkResource::KnownWifiNetwork
REQ/network/wireless_network/CON/Data::WifiCurrentlyConnected
REQ/network/wireless_network/CON/Data::WifiNetworkSaved
REQ/network/wireless_network/CON/NetworkResource::HotspotCapable
```

### `1.5.0-beta.40` only (6 additional vs 1.4-dev)

```
REQ/onboard_computer/storage/CON/Other::disktest binary is available on PATH
REQ/onboard_computer/storage/CON/Other::Onboard storage is low or the high disk usage warning is shown
REQ/presentation/branding_ui/CON/Other::A custom logo has been uploaded
REQ/presentation/branding_ui/CON/Other::A custom vehicle image has been uploaded
REQ/presentation/branding_ui/CON/Other::At least one model override has been uploaded
REQ/vehicle/recording/CON/Data::RecordingListed
```

## RTM

`requirements --rtm` no longer emits `constraint` kind rows. Assumptions appear on
parent requirement SRS output only; no standalone RTM citation per precondition.
