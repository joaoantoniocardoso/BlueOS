use std::f32::consts::PI;
use std::thread;
use std::time::{Duration, Instant};

use crate::runner::{execute_curl, join_url};
use catalog_kernel::id::journey::JourneyId;
use catalog_model::journey::HttpMethod;

pub const SITL_FRAME_CALIBRATION: &str = "calibration";
pub const SITL_FRAME_VECTORED: &str = "vectored";

const PWM_STOP: u16 = 1050;
const PWM_ATTITUDE_MODE: u16 = 1150;
const PWM_MAG_DANCE: u16 = 1250;
const RC_HOLD: u16 = 1500;

const SITL_BOARD_FALLBACK: &str = r#"{"name":"SITL","manufacturer":"ArduPilot Team","platform":"SITL_arm_linux_gnueabihf","path":null,"flags":[]}"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SitlRc {
    pub chan5: u16,
    pub chan6: u16,
    pub chan7: u16,
    pub chan8: u16,
}

pub struct BoardRestore {
    base: String,
    snapshot: String,
}

impl SitlRc {
    pub fn stop() -> Self {
        Self {
            chan5: PWM_STOP,
            chan6: RC_HOLD,
            chan7: RC_HOLD,
            chan8: RC_HOLD,
        }
    }

    pub fn mag_dance() -> Self {
        Self {
            chan5: PWM_MAG_DANCE,
            chan6: RC_HOLD,
            chan7: RC_HOLD,
            chan8: RC_HOLD,
        }
    }

    pub fn attitude_deg(roll_deg: i16, pitch_deg: i16, yaw_deg: i16) -> Self {
        Self {
            chan5: PWM_ATTITUDE_MODE,
            chan6: angle_pwm(roll_deg),
            chan7: angle_pwm(pitch_deg),
            chan8: angle_pwm(yaw_deg),
        }
    }
}

impl BoardRestore {
    pub fn snapshot(base: &str) -> Result<Self, String> {
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            snapshot: get_board(base)?,
        })
    }
}

impl Drop for BoardRestore {
    fn drop(&mut self) {
        if let Err(err) = set_board(&self.base, &self.snapshot, None) {
            eprintln!("sitl_cal: restore board failed: {err}; stopping then retrying");
        } else {
            return;
        }
        let stop = join_url(&self.base, "/ardupilot-manager/v1.0/stop");
        match execute_curl(&HttpMethod::Post, &stop, true, None, None) {
            Ok((status, body)) if status != 200 => {
                eprintln!("sitl_cal: POST /stop HTTP {status}: {body}");
            }
            Err(err) => eprintln!("sitl_cal: POST /stop: {err}"),
            Ok(_) => thread::sleep(Duration::from_secs(2)),
        }
        if let Err(err) = set_board(&self.base, &self.snapshot, None) {
            eprintln!("sitl_cal: restore board retry failed: {err}");
            return;
        }
        let start = join_url(&self.base, "/ardupilot-manager/v1.0/start");
        if let Err(err) = execute_curl(&HttpMethod::Post, &start, true, None, None) {
            eprintln!("sitl_cal: POST /start after restore: {err}");
        }
    }
}

pub fn needs_calibration_frame(id: JourneyId) -> bool {
    matches!(
        id,
        JourneyId::CalibrateGyroscope
            | JourneyId::CalibrateBarometer
            | JourneyId::LevelHorizon
            | JourneyId::CalibrateAccelerometer
            | JourneyId::CalibrateCompass
    )
}

pub fn needs_vectored_frame(id: JourneyId) -> bool {
    matches!(id, JourneyId::DetectMotorDirections)
}

pub fn get_board(base: &str) -> Result<String, String> {
    let (status, body) = execute_curl(
        &HttpMethod::Get,
        &join_url(base, "/ardupilot-manager/v1.0/board"),
        false,
        None,
        None,
    )?;
    if status != 200 {
        return Err(format!("GET /board HTTP {status}: {body}"));
    }
    Ok(body)
}

pub fn sitl_board_json(base: &str) -> Result<String, String> {
    let (status, body) = execute_curl(
        &HttpMethod::Get,
        &join_url(base, "/ardupilot-manager/v1.0/available_boards"),
        false,
        None,
        None,
    )?;
    if status != 200 {
        return Err(format!("GET /available_boards HTTP {status}: {body}"));
    }
    let boards: serde_json::Value =
        serde_json::from_str(&body).map_err(|err| format!("available_boards JSON: {err}"))?;
    let Some(list) = boards.as_array() else {
        return Ok(SITL_BOARD_FALLBACK.to_string());
    };
    for board in list {
        let platform = board.get("platform").and_then(|v| v.as_str()).unwrap_or("");
        let name = board.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if platform.contains("SITL") || name == "SITL" {
            return Ok(board.to_string());
        }
    }
    Ok(SITL_BOARD_FALLBACK.to_string())
}

pub fn set_board(base: &str, board_json: &str, sitl_frame: Option<&str>) -> Result<(), String> {
    // start_sitl() loads the frame from settings, not the in-memory setter on POST /board.
    if let Some(frame) = sitl_frame {
        let (status, body) = execute_curl(
            &HttpMethod::Post,
            &join_url(
                base,
                &format!("/ardupilot-manager/v1.0/sitl_frame?frame={frame}"),
            ),
            true,
            None,
            None,
        )?;
        if status != 200 {
            return Err(format!("POST /sitl_frame HTTP {status}: {body}"));
        }
    }
    let mut path = "/ardupilot-manager/v1.0/board".to_string();
    if let Some(frame) = sitl_frame {
        path.push_str("?sitl_frame=");
        path.push_str(frame);
    }
    let (status, body) = execute_curl(
        &HttpMethod::Post,
        &join_url(base, &path),
        true,
        Some(board_json),
        None,
    )?;
    if status != 200 {
        return Err(format!("POST /board HTTP {status}: {body}"));
    }
    Ok(())
}

pub fn wait_heartbeat(base: &str, timeout: Duration) -> Result<(), String> {
    // Short path /v1/mavlink/HEARTBEAT returns "None" on 1.4-dev; the FC lives here.
    let url = join_url(
        base,
        "/mavlink2rest/v1/mavlink/vehicles/1/components/1/messages/HEARTBEAT",
    );
    let deadline = Instant::now() + timeout;
    let mut last = String::from("no attempts");
    let mut seen_update: Option<String> = None;
    while Instant::now() < deadline {
        match execute_curl(&HttpMethod::Get, &url, false, None, None) {
            Ok((200, body)) => {
                if let Some(update) = heartbeat_last_update(&body) {
                    if seen_update.as_ref() == Some(&update) {
                        last = format!("stale last_update={update}");
                    } else if seen_update.is_some() {
                        return Ok(());
                    } else {
                        last = format!("first last_update={update}");
                        seen_update = Some(update);
                    }
                } else {
                    last = format!("HTTP 200: {body}");
                }
            }
            Ok((status, body)) => last = format!("HTTP {status}: {body}"),
            Err(err) => last = err,
        }
        thread::sleep(Duration::from_secs(2));
    }
    Err(format!(
        "no live HEARTBEAT within {}s ({last})",
        timeout.as_secs()
    ))
}

fn heartbeat_last_update(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value
        .get("status")?
        .get("time")?
        .get("last_update")?
        .as_str()
        .map(str::to_string)
}

pub fn post_rc_override(base: &str, rc: SitlRc) -> Result<(), String> {
    let (status, body) = execute_curl(
        &HttpMethod::Post,
        &join_url(base, "/mavlink2rest/mavlink"),
        true,
        Some(&rc_override_json(rc)),
        None,
    )?;
    if status != 200 {
        return Err(format!("POST /mavlink2rest/mavlink HTTP {status}: {body}"));
    }
    Ok(())
}

pub fn release_calibration_servos(base: &str) -> Result<(), String> {
    for channel in 5u8..=8 {
        post_param_set(base, &format!("SERVO{channel}_FUNCTION"), 0.0)?;
    }
    thread::sleep(Duration::from_millis(500));
    post_do_set_servo(base, 5, PWM_STOP)?;
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut last = String::from("no SERVO_OUTPUT_RAW");
    while Instant::now() < deadline {
        match servo5_raw(base) {
            Ok(pwm) if (1000..1100).contains(&pwm) => return Ok(()),
            Ok(pwm) => last = format!("servo5_raw={pwm}"),
            Err(err) => last = err,
        }
        thread::sleep(Duration::from_millis(250));
        let _ = post_do_set_servo(base, 5, PWM_STOP);
    }
    Err(format!(
        "SERVO5 still not in stop band after disabling functions ({last})"
    ))
}

pub fn wait_level_attitude(base: &str, timeout: Duration) -> Result<(), String> {
    let rc = SitlRc::attitude_deg(0, 0, 0);
    post_sitl_servos(base, rc)?;
    let deadline = Instant::now() + timeout;
    let mut last = String::from("no ATTITUDE");
    while Instant::now() < deadline {
        let _ = post_sitl_servos(base, rc);
        match attitude_roll_pitch(base) {
            Ok((roll, pitch)) if roll.abs() < 0.09 && pitch.abs() < 0.09 => return Ok(()),
            Ok((roll, pitch)) => last = format!("roll={roll:.3} pitch={pitch:.3} rad"),
            Err(err) => last = err,
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err(format!(
        "vehicle not level within {}s ({last})",
        timeout.as_secs()
    ))
}

pub fn rc_override_json(rc: SitlRc) -> String {
    format!(
        r#"{{"header":{{"system_id":255,"component_id":0,"sequence":0}},"message":{{"type":"RC_CHANNELS_OVERRIDE","target_system":1,"target_component":1,"chan1_raw":{hold},"chan2_raw":{hold},"chan3_raw":{hold},"chan4_raw":{hold},"chan5_raw":{},"chan6_raw":{},"chan7_raw":{},"chan8_raw":{}}}}}"#,
        rc.chan5,
        rc.chan6,
        rc.chan7,
        rc.chan8,
        hold = RC_HOLD,
    )
}

fn post_param_set(base: &str, name: &str, value: f32) -> Result<(), String> {
    let (status, body) = execute_curl(
        &HttpMethod::Post,
        &join_url(base, "/mavlink2rest/mavlink"),
        true,
        Some(&param_set_json(name, value)),
        None,
    )?;
    if status != 200 {
        return Err(format!("PARAM_SET {name} HTTP {status}: {body}"));
    }
    Ok(())
}

fn post_do_set_servo(base: &str, servo: u8, pwm: u16) -> Result<(), String> {
    let (status, body) = execute_curl(
        &HttpMethod::Post,
        &join_url(base, "/mavlink2rest/mavlink"),
        true,
        Some(&do_set_servo_json(servo, pwm)),
        None,
    )?;
    if status != 200 {
        return Err(format!("DO_SET_SERVO {servo} HTTP {status}: {body}"));
    }
    Ok(())
}

fn post_sitl_servos(base: &str, rc: SitlRc) -> Result<(), String> {
    post_do_set_servo(base, 5, rc.chan5)?;
    post_do_set_servo(base, 6, rc.chan6)?;
    post_do_set_servo(base, 7, rc.chan7)?;
    post_do_set_servo(base, 8, rc.chan8)
}

fn attitude_roll_pitch(base: &str) -> Result<(f32, f32), String> {
    let (status, body) = execute_curl(
        &HttpMethod::Get,
        &join_url(
            base,
            "/mavlink2rest/v1/mavlink/vehicles/1/components/1/messages/ATTITUDE",
        ),
        false,
        None,
        None,
    )?;
    if status != 200 {
        return Err(format!("GET ATTITUDE HTTP {status}: {body}"));
    }
    let value: serde_json::Value =
        serde_json::from_str(&body).map_err(|err| format!("ATTITUDE JSON: {err}"))?;
    let message = value
        .get("message")
        .ok_or_else(|| format!("no ATTITUDE message in {body}"))?;
    let roll = message
        .get("roll")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| format!("no roll in {body}"))? as f32;
    let pitch = message
        .get("pitch")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| format!("no pitch in {body}"))? as f32;
    Ok((roll, pitch))
}

fn servo5_raw(base: &str) -> Result<u16, String> {
    let (status, body) = execute_curl(
        &HttpMethod::Get,
        &join_url(
            base,
            "/mavlink2rest/v1/mavlink/vehicles/1/components/1/messages/SERVO_OUTPUT_RAW",
        ),
        false,
        None,
        None,
    )?;
    if status != 200 {
        return Err(format!("GET SERVO_OUTPUT_RAW HTTP {status}: {body}"));
    }
    let value: serde_json::Value =
        serde_json::from_str(&body).map_err(|err| format!("SERVO_OUTPUT_RAW JSON: {err}"))?;
    value
        .get("message")
        .and_then(|m| m.get("servo5_raw"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .ok_or_else(|| format!("no servo5_raw in {body}"))
}

fn param_set_json(name: &str, value: f32) -> String {
    format!(
        r#"{{"header":{{"system_id":255,"component_id":0,"sequence":0}},"message":{{"type":"PARAM_SET","param_value":{value},"target_system":1,"target_component":1,"param_id":{},"param_type":{{"type":"MAV_PARAM_TYPE_INT16"}}}}}}"#,
        param_id_json(name)
    )
}

fn do_set_servo_json(servo: u8, pwm: u16) -> String {
    format!(
        r#"{{"header":{{"system_id":255,"component_id":1,"sequence":0}},"message":{{"type":"COMMAND_LONG","param1":{servo},"param2":{pwm},"param3":0,"param4":0,"param5":0,"param6":0,"param7":0,"command":{{"type":"MAV_CMD_DO_SET_SERVO"}},"target_system":1,"target_component":1,"confirmation":0}}}}"#
    )
}

fn param_id_json(name: &str) -> String {
    let mut chars: Vec<String> = name.chars().map(|c| format!("\"{c}\"")).collect();
    while chars.len() < 16 {
        chars.push("\"\\u0000\"".into());
    }
    format!("[{}]", chars.join(","))
}

fn angle_pwm(deg: i16) -> u16 {
    let theta = (deg as f32).clamp(-180.0, 180.0) * PI / 180.0;
    (1500.0 + 500.0 * (theta / PI))
        .round()
        .clamp(1000.0, 2000.0) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_pwm_cardinals() {
        assert_eq!(angle_pwm(0), 1500);
        assert_eq!(angle_pwm(90), 1750);
        assert_eq!(angle_pwm(-90), 1250);
        assert_eq!(angle_pwm(180), 2000);
        assert_eq!(angle_pwm(-180), 1000);
    }

    #[test]
    fn stop_uses_ch5_stop_band() {
        assert!(SitlRc::stop().chan5 >= 1000 && SitlRc::stop().chan5 < 1100);
        let attitude = SitlRc::attitude_deg(0, 0, 0);
        assert!(attitude.chan5 >= 1100 && attitude.chan5 < 1200);
        assert!(SitlRc::mag_dance().chan5 >= 1200 && SitlRc::mag_dance().chan5 < 1300);
    }

    #[test]
    fn rc_json_contains_override_type() {
        let json = rc_override_json(SitlRc::stop());
        assert!(json.contains("RC_CHANNELS_OVERRIDE"));
        assert!(json.contains("chan5_raw"));
    }

    #[test]
    fn param_set_json_pads_id() {
        let json = param_set_json("SERVO5_FUNCTION", 0.0);
        assert!(json.contains("PARAM_SET"));
        assert!(json.contains("\"S\",\"E\",\"R\",\"V\",\"O\",\"5\""));
        assert!(json.contains("MAV_PARAM_TYPE_INT16"));
    }

    #[test]
    fn heartbeat_last_update_reads_nested_status() {
        let body = r#"{"message":{"type":"HEARTBEAT"},"status":{"time":{"last_update":"2026-08-14T01:42:50Z","counter":3}}}"#;
        assert_eq!(
            heartbeat_last_update(body).as_deref(),
            Some("2026-08-14T01:42:50Z")
        );
        assert_eq!(heartbeat_last_update("None"), None);
    }
}
