use crate::coverage::{mapping, CoverageKind, TrackerMapping};
use catalog_kernel::id::journey::JourneyId;

pub const TRACKER_MAPPINGS: &[TrackerMapping] = &[
    // ArduRover
    mapping("ArduRover", "Complete the wizard and write parameters, load Lua scripts successfully", CoverageKind::Unmodeled, &[], "No end-to-end wizard journey"),
    // ArduRover
    mapping("ArduRover", "Check that BlueBoat parameters loaded successfully", CoverageKind::Unmodeled, &[], "Parameter preset verification not modeled"),
    // ArduRover
    mapping("ArduRover", "Check that the navlight script loaded and works as expected", CoverageKind::External, &[], "Lua script on-vehicle verification"),
    // ArduRover
    mapping("ArduRover", "View/adjust 3D model, verify it matches frame type", CoverageKind::Journey, &[JourneyId::Upload3dModelOverride], ""),
    // ArduRover
    mapping("ArduRover", "Change frame type, verify model and parameters change correctly", CoverageKind::Journey, &[JourneyId::ChangeBoard], ""),
    // ArduRover
    mapping("ArduRover", "Load default parameters (no change expect", CoverageKind::Unmodeled, &[], "Restore-default-parameters flow not modeled"),
    // ArduRover
    mapping("ArduRover", "Do motor test, verify direction change via parameter works", CoverageKind::Unmodeled, &[], "Motor test UI not modeled"),
    // ArduRover
    mapping("ArduRover", "Calibrate Gyroscope", CoverageKind::Journey, &[JourneyId::CalibrateGyroscope], ""),
    // ArduRover
    mapping("ArduRover", "Calibrate Accelerometer (Via all possible methods)", CoverageKind::Journey, &[JourneyId::CalibrateAccelerometer], ""),
    // ArduRover
    mapping("ArduRover", "Calibrate compass (  Via all possible methods)", CoverageKind::Journey, &[JourneyId::CalibrateCompass], ""),
    // ArduSub
    mapping("ArduSub", "Complete the wizard for SUB and write parameters", CoverageKind::Unmodeled, &[], "No end-to-end wizard journey"),
    // ArduSub
    mapping("ArduSub", "Check that Sub parameters loaded successfully", CoverageKind::Unmodeled, &[], "Parameter preset verification not modeled"),
    // ArduSub
    mapping("ArduSub", "Change frame type, verify model and parameters change correctly", CoverageKind::Journey, &[JourneyId::ChangeBoard], ""),
    // ArduSub
    mapping("ArduSub", "Calibrate Gyroscope", CoverageKind::Journey, &[JourneyId::CalibrateGyroscope], ""),
    // ArduSub
    mapping("ArduSub", "Calibrate Accelerometer (Via all possible methods)", CoverageKind::Journey, &[JourneyId::CalibrateAccelerometer], ""),
    // ArduSub
    mapping("ArduSub", "Calibrate compass (  Via all possible methods)", CoverageKind::Journey, &[JourneyId::CalibrateCompass], ""),
    // ArduSub
    mapping("ArduSub", "Do motor test, verify direction change via parameter works (drag sliders)", CoverageKind::Unmodeled, &[], "Motor test UI not modeled"),
    // ArduSub
    mapping("ArduSub", "Run automatic motor direction detection (press button)", CoverageKind::Journey, &[JourneyId::DetectMotorDirections], ""),
    // ArduSub
    mapping("ArduSub", "Accessories / Gripper Setup (TBD)", CoverageKind::Unmodeled, &[], "Gripper setup not modeled"),
    // ArduSub
    mapping("ArduSub", "View/adjust 3D model, verify it matches frame type", CoverageKind::Journey, &[JourneyId::Upload3dModelOverride], ""),
    // ArduSub
    mapping("ArduSub", "Load default parameters", CoverageKind::Unmodeled, &[], "Restore-default-parameters flow not modeled"),
    // ArduSub
    mapping("ArduSub", "Lights Setup", CoverageKind::Unmodeled, &[], "Lights setup not modeled"),
    // Autopilot
    mapping("Autopilot", "Autopilot information displays properly", CoverageKind::Journey, &[JourneyId::ViewSystemInformation], ""),
    // Autopilot
    mapping("Autopilot", "Program autopilot firmware", CoverageKind::Journey, &[JourneyId::UpdateFirmwareOnline, JourneyId::UploadCustomFirmware], ""),
    // Autopilot
    mapping("Autopilot", "Navigator", CoverageKind::Journey, &[JourneyId::ChangeBoard], "Board selection for Navigator"),
    // Autopilot
    mapping("Autopilot", "Plug in a USB GPS and set Serial3 to talk to the usb device instead of pi serial port ( by setting the autopilot serial port to ttyUSB)", CoverageKind::Journey, &[JourneyId::AddExternalNmeaGpsSocket], ""),
    // Autopilot
    mapping("Autopilot", "Pixhawk", CoverageKind::Journey, &[JourneyId::ChangeBoard], "Board selection for Pixhawk"),
    // Autopilot
    mapping("Autopilot", "Search and change a parameter", CoverageKind::Unmodeled, &[], "Inline parameter edit journey not modeled"),
    // Autopilot
    mapping("Autopilot", "Parameter descriptions are populated", CoverageKind::Unmodeled, &[], "Parameter metadata UI not modeled"),
    // BlueOS Core
    mapping("BlueOS Core", "Log Browser: View, Download, delete autopilot log", CoverageKind::Unmodeled, &[], "Log browser page has no journey"),
    // BlueOS Core
    mapping("BlueOS Core", "File Manager: View, create, delete, download file, verify lua script directory present", CoverageKind::Journey, &[JourneyId::ManageBlueosFiles], ""),
    // BlueOS Core
    mapping("BlueOS Core", "View system information", CoverageKind::Journey, &[JourneyId::ViewSystemInformation], ""),
    // BlueOS Core
    mapping("BlueOS Core", "Check top bar system status information — is everything displayed correctly?", CoverageKind::Unmodeled, &[], "Composite top-bar status not modeled"),
    // BlueOS Core
    mapping("BlueOS Core", "Check for limited / unknow / available internet status", CoverageKind::Journey, &[JourneyId::MonitorInternetConnectivity], ""),
    // BlueOS Core
    mapping("BlueOS Core", "Check for SD card full status on top bar", CoverageKind::Journey, &[JourneyId::InspectDiskUsage], ""),
    // BlueOS Core
    mapping("BlueOS Core", "Check for reaching BlueOs by wifi and change to wired if available", CoverageKind::Journey, &[JourneyId::SetNetworkInterfacePriority], ""),
    // BlueOS Core
    mapping("BlueOS Core", "Terminal loads correctly", CoverageKind::Journey, &[JourneyId::AccessWebTerminal], ""),
    // BlueOS Core
    mapping("BlueOS Core", "Nmea Injector (needs pre-testing for a decent workflow)", CoverageKind::Journey, &[JourneyId::ViewConfiguredNmeaSockets], ""),
    // BlueOS Core
    mapping("BlueOS Core", "Make sure the system did not reboot unecessarily", CoverageKind::Unmodeled, &[], "Stability monitoring not modeled"),
    // BlueOS Core
    mapping("BlueOS Core", "Boot from CI-generated images", CoverageKind::Journey, &[JourneyId::UpdateBootstrapImage, JourneyId::VehicleFirstBoot], ""),
    // Bridges
    mapping("Bridges", "Plug in a gps, disconnect from the autopilot, create a new bridge to 127.0.0.1:15000. On the pi’s terminal, run `nc -ul 15000` (double check these instructions) and receive data (possibly unreadable, that is fine)", CoverageKind::Journey, &[JourneyId::CreateSerialToUdpBridge], ""),
    // Camera
    mapping("Camera", "Camera Setup", CoverageKind::Journey, &[JourneyId::ConfigureCameraStream], ""),
    // Camera
    mapping("Camera", "Test reseting to default video settings", CoverageKind::Journey, &[JourneyId::ConfigureVideoStream], ""),
    // Camera
    mapping("Camera", "Test default video config on Cockpit (Enable stats for nerds)", CoverageKind::External, &[], "Verify stream in Cockpit GCS"),
    // Camera
    mapping("Camera", "Test UDP video (default) on QGC", CoverageKind::External, &[], "Verify stream in QGroundControl"),
    // Camera
    mapping("Camera", "Test handling of multiple cameras on QGC", CoverageKind::External, &[], "Verify multi-camera in QGroundControl"),
    // Camera
    mapping("Camera", "Test RTSP video on QGC", CoverageKind::External, &[], "Verify RTSP in QGroundControl"),
    // Camera
    mapping("Camera", "Test RTSP re-stream", CoverageKind::Journey, &[JourneyId::ConfigureVideoStream], "BlueOS stream config; GCS verify is external"),
    // Extensions
    mapping("Extensions", "Install an extension, view details during installation", CoverageKind::Journey, &[JourneyId::InstallExtension], ""),
    // Extensions
    mapping("Extensions", "Uninstall an extension", CoverageKind::Journey, &[JourneyId::UninstallExtension], ""),
    // Extensions
    mapping("Extensions", "Change an extension to a custom version (cockpit to master)", CoverageKind::Journey, &[JourneyId::EditExtensionDevVersion], ""),
    // Extensions
    mapping("Extensions", "Restart an extension", CoverageKind::Journey, &[JourneyId::ConfigureInstalledExtension], ""),
    // Extensions
    mapping("Extensions", "View extension logs", CoverageKind::Journey, &[JourneyId::ConfigureInstalledExtension], ""),
    // Extensions
    mapping("Extensions", "Download extension logs", CoverageKind::Unmodeled, &[], "Extension log download not modeled"),
    // MAVLink
    mapping("MAVLink", "Mavlink Endpoint creation UI", CoverageKind::Unmodeled, &[], "MAVLink endpoint UI not modeled"),
    // MAVLink
    mapping("MAVLink", "MAVLink-router QGC", CoverageKind::External, &[], "QGroundControl MAVLink routing"),
    // MAVLink
    mapping("MAVLink", "MAVLink-router Cockpit", CoverageKind::External, &[], "Cockpit MAVLink routing"),
    // MAVLink
    mapping("MAVLink", "MAVLink-Server QGC", CoverageKind::External, &[], "QGroundControl MAVLink server"),
    // MAVLink
    mapping("MAVLink", "MAVLink-Server Cockpit", CoverageKind::External, &[], "Cockpit MAVLink server"),
    // Networking
    mapping("Networking", "Test wired Ethernet connections", CoverageKind::Journey, &[JourneyId::ProbeInterfaceInternetConnectivity], ""),
    // Networking
    mapping("Networking", "Check persistency of 192.168.2.2 ip on multiple boots", CoverageKind::Journey, &[JourneyId::EnableOnboardDhcpServer], ""),
    // Networking
    mapping("Networking", "Change DHCP server ip address", CoverageKind::Unmodeled, &[], "DHCP server IP change not modeled"),
    // Networking
    mapping("Networking", "tests", CoverageKind::Ignore, &[], "Placeholder row"),
    // Networking
    mapping("Networking", "Default DHCP server works fine", CoverageKind::Journey, &[JourneyId::EnableOnboardDhcpServer], ""),
    // Networking
    mapping("Networking", "Check connectivity on usb otg", CoverageKind::Journey, &[JourneyId::ProbeInterfaceInternetConnectivity], "USB OTG network path"),
    // Networking
    mapping("Networking", "Test network interface priority", CoverageKind::Journey, &[JourneyId::SetNetworkInterfacePriority], ""),
    // Networking
    mapping("Networking", "Network Test", CoverageKind::Journey, &[JourneyId::RunInternetSpeedTest], ""),
    // Networking
    mapping("Networking", "Local connection", CoverageKind::Journey, &[JourneyId::ProbeInterfaceInternetConnectivity], ""),
    // Networking
    mapping("Networking", "Internet connection", CoverageKind::Journey, &[JourneyId::VerifyInternetConnectivity], ""),
    // Networking
    mapping("Networking", "Access BlueOS via Pi Hotspot", CoverageKind::Journey, &[JourneyId::ToggleHotspot, JourneyId::ConfigureHotspotCredentials, JourneyId::ToggleSmartHotspot, JourneyId::AccessBlueosWebInterface], ""),
    // Networking
    mapping("Networking", "Change mDNS and try to access Pi with it", CoverageKind::Journey, &[JourneyId::ChangeMdnsHostname, JourneyId::DiscoverBlueosOnNetwork], ""),
    // Networking
    mapping("Networking", "Connect to Wifi with internet via Pi Radio", CoverageKind::Journey, &[
        JourneyId::ConnectToWifiNetwork,
        JourneyId::ConnectToHiddenWifiNetwork,
        JourneyId::ForceWifiNetworkPassword,
        JourneyId::ReconnectToSavedWifiNetwork,
        JourneyId::RejectInvalidWifiCredentials,
        JourneyId::DisconnectFromWifiNetwork,
        JourneyId::ForgetSavedWifiNetwork,
        JourneyId::DetectWifiApLoss,
        JourneyId::AutoconnectToSavedWifiNetwork,
    ], ""),
    // Ping
    mapping("Ping", "HotPlug a Ping1D (USB)", CoverageKind::Journey, &[JourneyId::ViewDetectedSonarDevices], ""),
    // Ping
    mapping("Ping", "Check if it shows up in the UI", CoverageKind::Journey, &[JourneyId::ViewDetectedSonarDevices], ""),
    // Ping
    mapping("Ping", "Check if Mavlink messages work as expected", CoverageKind::Journey, &[JourneyId::EnablePing1dRangefinderMavlink], ""),
    // Ping
    mapping("Ping", "HotPlug in a Ping360 via (USB/Ethernet)", CoverageKind::Journey, &[JourneyId::ViewDetectedSonarDevices], ""),
    // Ping
    mapping("Ping", "Check if it is detected in either Sonarview/PingViewer/Pingviewer2", CoverageKind::External, &[], "PingViewer desktop app detection"),
    // ROV
    mapping("ROV", "Test ROV operation via QGC", CoverageKind::External, &[], "Full vehicle operation in QGroundControl"),
    // ROV
    mapping("ROV", "Lights, camera tilt, mode change, arm/disarm", CoverageKind::External, &[], "Vehicle control via GCS"),
    // Software Update
    mapping("Software Update", "BlueOS version utility can load and change versions", CoverageKind::Journey, &[JourneyId::UpdateBlueosVersion, JourneyId::SwitchLocalBlueosVersion], ""),
    // Software Update
    mapping("Software Update", "Pirate mode allows beta BlueOS install", CoverageKind::Journey, &[JourneyId::UpdateBlueosVersion], "Requires pirate mode precondition"),
    // Software Update
    mapping("Software Update", "Arbitrary git branch BlueOS fork installation", CoverageKind::Unmodeled, &[], "Custom BlueOS fork install not modeled"),
    // Software Update
    mapping("Software Update", "Delete unused versions", CoverageKind::Journey, &[JourneyId::DeleteLocalBlueosVersion], ""),
    // System Settings
    mapping("System Settings", "BlueOS Log Download valid", CoverageKind::Unmodeled, &[], "System log download not modeled"),
    // System Settings
    mapping("System Settings", "Restart Autopilot works", CoverageKind::Journey, &[JourneyId::RestartAutopilot], ""),
    // System Settings
    mapping("System Settings", "Restart Core Container works", CoverageKind::Unmodeled, &[], "Core container restart not modeled"),
    // System Settings
    mapping("System Settings", "Restart system (pi) works", CoverageKind::Journey, &[JourneyId::RebootOnboardComputer], ""),
    // System Settings
    mapping("System Settings", "Enter special dev mode works", CoverageKind::Unmodeled, &[], "Dev mode toggle not modeled"),
    // System Settings
    mapping("System Settings", "Correct autopilot version / BlueOS version listed, updates with changes to both as expected", CoverageKind::Journey, &[JourneyId::ViewSystemInformation], ""),
    // System Settings
    mapping("System Settings", "Reset settings and check for errors after restarting (how?)", CoverageKind::Journey, &[JourneyId::ResetBlueosSettings], ""),
    // System Settings
    mapping("System Settings", "Download/delete System logs", CoverageKind::Unmodeled, &[], "Service log download/delete not modeled"),
    // System Settings
    mapping("System Settings", "Download/remote Mavlink logs", CoverageKind::Unmodeled, &[], "MAVLink log remote download not modeled"),
    // UI
    mapping("UI", "Test functionality of all top bar utilities", CoverageKind::Unmodeled, &[], "Top bar composite check not modeled"),
    // UI
    mapping("UI", "Test Wi-Fi manager functions and hotspot toggle", CoverageKind::Journey, &[
        JourneyId::ConnectToWifiNetwork,
        JourneyId::ConnectToHiddenWifiNetwork,
        JourneyId::DisconnectFromWifiNetwork,
        JourneyId::ForgetSavedWifiNetwork,
        JourneyId::ForceWifiNetworkPassword,
        JourneyId::ReconnectToSavedWifiNetwork,
        JourneyId::RejectInvalidWifiCredentials,
        JourneyId::DetectWifiApLoss,
        JourneyId::AutoconnectToSavedWifiNetwork,
        JourneyId::ToggleHotspot,
        JourneyId::ConfigureHotspotCredentials,
        JourneyId::ToggleSmartHotspot,
    ], ""),
    // UI
    mapping("UI", "Toggle pirate mode", CoverageKind::Unmodeled, &[], "Pirate mode toggle not modeled"),
    // UI
    mapping("UI", "Toggle dark mode", CoverageKind::Journey, &[JourneyId::ChangeUiThemeColor], ""),
    // UI
    mapping("UI", "Check all UI interfaces on both Pirate and non-pirate mode for obvious issues", CoverageKind::Unmodeled, &[], "Full UI regression not modeled"),
    // Update
    mapping("Update", "Update from current stable to stable candidate", CoverageKind::Journey, &[JourneyId::UpdateBlueosVersion], ""),
    // Wizard
    mapping("Wizard", "Check that both BlueROV2 and BlueBoat parameters options load successfully", CoverageKind::Unmodeled, &[], "Wizard parameter presets not modeled"),
    // Wizard
    mapping("Wizard", "Complete the wizard and write parameters, load Lua scripts successfully", CoverageKind::Unmodeled, &[], "No end-to-end wizard journey"),
    // Wizard
    mapping("Wizard", "Run through the wizard on first boot", CoverageKind::Journey, &[JourneyId::VehicleFirstBoot], "First-boot wizard partially modeled"),
    // Physical ROV
    mapping("Physical ROV", "Flash 128gb SD card with Etcher", CoverageKind::External, &[], "Physical SD imaging"),
    // Physical ROV
    mapping("Physical ROV", "Record First Boot time <2min", CoverageKind::External, &[], "Boot timing measurement"),
    // Physical ROV
    mapping("Physical ROV", "Connect to 192.168.2.2 after first boot", CoverageKind::Journey, &[JourneyId::AccessBlueosWebInterface], "Reach BlueOS over default IP"),
    // Physical ROV
    mapping("Physical ROV", "Connect to blueos.local after first boot", CoverageKind::Journey, &[JourneyId::DiscoverBlueosOnNetwork], "Reach BlueOS via mDNS"),
    // Physical ROV
    mapping("Physical ROV", "Record Second Boot time <2min", CoverageKind::External, &[], "Boot timing measurement"),
    // Physical ROV
    mapping("Physical ROV", "Connect to 192.168.2.2 after second boot", CoverageKind::Journey, &[JourneyId::AccessBlueosWebInterface], ""),
    // Physical ROV
    mapping("Physical ROV", "Connect to blueos.local after second boot", CoverageKind::Journey, &[JourneyId::DiscoverBlueosOnNetwork], ""),
    // Physical ROV
    mapping("Physical ROV", "Go through setup wizard to ROV Standard", CoverageKind::Unmodeled, &[], "Wizard end-to-end not modeled"),
    // Physical ROV
    mapping("Physical ROV", "Connects to wifi", CoverageKind::Journey, &[
        JourneyId::ConnectToWifiNetwork,
        JourneyId::DisconnectFromWifiNetwork,
        JourneyId::ForgetSavedWifiNetwork,
    ], ""),
    // Physical ROV
    mapping("Physical ROV", "Confirm parameters are correct (output 1-6 are motor, Output 13 & 14 are lights, output 16 is Mount1Pitch)", CoverageKind::External, &[], "On-vehicle parameter verification"),
    // Physical ROV
    mapping("Physical ROV", "Confirm FAILSSAFES\nHeatbeat Loss -> disable\nPilot Input Loss -> 3s -> disarm\nLeak Detection -> warn only\nExcess internal pressure -> 105000 Pa -> Disabled\nExcess internal Temperature -> 62 C -> disabled\nLow Battery -> action when battery is low -> none -> action when battery is critical -> none\nSensor Fusion Uncertainty -> Disabled\nCrash Detection -> Disabled", CoverageKind::External, &[], "On-vehicle failsafe verification"),
    // Physical ROV
    mapping("Physical ROV", "Camera Gimble is ON be default", CoverageKind::External, &[], "On-vehicle camera gimbal check"),
    // Physical ROV
    mapping("Physical ROV", "Image in Overview and PWM Outputs is corrent (2 vertical thrusters)", CoverageKind::External, &[], "On-vehicle UI image check"),
    // Physical ROV
    mapping("Physical ROV", "Under Configure, update to Heavy Parameters and confirm output 7 & 8 are motor and value is at 1500, other stuff is backend)", CoverageKind::External, &[], "On-vehicle parameter profile switch"),
    // Physical ROV
    mapping("Physical ROV", "Go back to Standard Parameters (output 7 & 8 are disabled)", CoverageKind::External, &[], "On-vehicle parameter profile switch"),
    // Physical ROV
    mapping("Physical ROV", "Run Wizard again to Standard and confirm outputs are correct (output 1-6 are motor, Output 13 & 14 is lights, output 16 is Mount1Pitch, everything else is Disabled)", CoverageKind::External, &[], "On-vehicle wizard re-run"),
    // Physical ROV
    mapping("Physical ROV", "Go through setup wizard to ROV Heavy", CoverageKind::Unmodeled, &[], "Wizard end-to-end not modeled"),
    // Physical ROV
    mapping("Physical ROV", "Confirm parameters are correct (output 1-8 are motor with values at 1500, Output 13 & 14 is lights, output 16 is Mount1Pitch)", CoverageKind::External, &[], "On-vehicle parameter verification"),
    // Physical ROV
    mapping("Physical ROV", "Image in Overview and PWM Outputs is corrent (4 vertical thrusters)", CoverageKind::External, &[], "On-vehicle UI image check"),
    // Physical ROV
    mapping("Physical ROV", "Under Configure, update to Standard Parameters and confirm output 7 & 8 are disabled, other stuff is backend", CoverageKind::External, &[], "On-vehicle parameter profile switch"),
    // Physical ROV
    mapping("Physical ROV", "Go back to Heavy Parameters and confirm output 7 & 8 are motor with values at 1500", CoverageKind::External, &[], "On-vehicle parameter profile switch"),
    // Physical ROV
    mapping("Physical ROV", "Run Wizard again to Heavy and confirm outputs are correct (output 1-6 are motor, Output 13 & 14 is lights, output 16 is Mount1Pitch, everything else is Disabled)", CoverageKind::External, &[], "On-vehicle wizard re-run"),
    // Physical ROV
    mapping("Physical ROV", "Has newest stable ArduSub", CoverageKind::Journey, &[JourneyId::UpdateFirmwareOnline], "Firmware version check"),
    // Physical ROV
    mapping("Physical ROV", "Calibrate Gyroscope", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical ROV
    mapping("Physical ROV", "Calibrate accelerometer Quick", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical ROV
    mapping("Physical ROV", "Calibrate accelerometer Full", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical ROV
    mapping("Physical ROV", "Calibrate compass - large vehicle", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical ROV
    mapping("Physical ROV", "Calibrate compass - Full", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical ROV
    mapping("Physical ROV", "USB stream visible in Cockpit", CoverageKind::External, &[], "Cockpit video verify"),
    // Physical ROV
    mapping("Physical ROV", "Camera tilt", CoverageKind::External, &[], "On-vehicle camera control"),
    // Physical ROV
    mapping("Physical ROV", "Camera stabalization", CoverageKind::External, &[], "On-vehicle camera stabilization"),
    // Physical ROV
    mapping("Physical ROV", "lights turn on", CoverageKind::External, &[], "On-vehicle lights"),
    // Physical ROV
    mapping("Physical ROV", "swaping direction of thrusters works", CoverageKind::External, &[], "On-vehicle thruster direction"),
    // Physical ROV
    mapping("Physical ROV", "fast thruster response to controller input. There is no weird lag or unexpected spinning of thrusters)", CoverageKind::External, &[], "On-vehicle thruster response"),
    // Physical ROV
    mapping("Physical ROV", "reading a correct voltage and current/W (10-16V and 0.6-1A)", CoverageKind::External, &[], "On-vehicle power telemetry"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Flash 128gb SD card with Etcher", CoverageKind::External, &[], "Physical SD imaging"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Record First Boot time <120s", CoverageKind::External, &[], "Boot timing measurement"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Connect to 192.168.2.2 after first boot", CoverageKind::Journey, &[JourneyId::AccessBlueosWebInterface], ""),
    // Physical Tests BB
    mapping("Physical Tests BB", "Connect to blueos.local after first boot", CoverageKind::Journey, &[JourneyId::DiscoverBlueosOnNetwork], ""),
    // Physical Tests BB
    mapping("Physical Tests BB", "Record Second Boot time <120s", CoverageKind::External, &[], "Boot timing measurement"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Connect to 192.168.2.2 after second boot", CoverageKind::Journey, &[JourneyId::AccessBlueosWebInterface], ""),
    // Physical Tests BB
    mapping("Physical Tests BB", "Connect to blueos.local after second boot", CoverageKind::Journey, &[JourneyId::DiscoverBlueosOnNetwork], ""),
    // Physical Tests BB
    mapping("Physical Tests BB", "Connects to wifi", CoverageKind::Journey, &[
        JourneyId::ConnectToWifiNetwork,
        JourneyId::DisconnectFromWifiNetwork,
        JourneyId::ForgetSavedWifiNetwork,
    ], ""),
    // Physical Tests BB
    mapping("Physical Tests BB", "Confirm motor and NavLight functionality", CoverageKind::External, &[], "On-vehicle motor/navlight"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Has newest stable ArduRover", CoverageKind::Journey, &[JourneyId::UpdateFirmwareOnline], "Firmware version check"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Confirm FAILSSAFES\nHeatbeat Loss -> 5 seconds\nLow Battery -> action when battery is low -> none -> action when battery is critical -> none\nSensor Fusion Uncertainty -> Disabled\nCrash Detection -> Disabled", CoverageKind::External, &[], "On-vehicle failsafe verification"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Confirm GPS is functional", CoverageKind::External, &[], "On-vehicle GPS"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Image in Overview and PWM Outputs is corrent (2 motors)", CoverageKind::External, &[], "On-vehicle UI image check"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Calibrate Gyroscope", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Calibrate accelerometer Quick", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Calibrate accelerometer Full", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Calibrate compass - large vehicle", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Calibrate compass - Full", CoverageKind::External, &[], "On-vehicle calibration verify"),
    // Physical Tests BB
    mapping("Physical Tests BB", "Cockpit opens and is functional", CoverageKind::External, &[], "Cockpit GCS smoke test"),
    // Physical Tests BB
    mapping("Physical Tests BB", "swaping direction of thrusters works", CoverageKind::External, &[], "On-vehicle thruster direction"),
    // Physical Tests BB
    mapping("Physical Tests BB", "fast thruster response to controller input. There is no weird lag or unexpected spinning of thrusters)", CoverageKind::External, &[], "On-vehicle thruster response"),
    // Physical Tests BB
    mapping("Physical Tests BB", "reading a correct voltage and current/W (10-14V and 0.6-1A)", CoverageKind::External, &[], "On-vehicle power telemetry"),
    // 
    mapping("", "RUN THE TOUR!", CoverageKind::Ignore, &[], "Non-test footer row"),
];
