use crate::runner::{execute_curl, join_url};
use catalog_model::journey::HttpMethod;

/// Disposable Redirect stream for mutating-smoke `POST /streams` (does not touch `/dev/video2`).
pub const SMOKE_CATALOG_STREAM_JSON: &str = r##"{"name":"__smoke_catalog__","source":"Redirect","stream_information":{"endpoints":["udp://127.0.0.1:5599"],"configuration":{"type":"redirect"},"extended_configuration":{"thermal":false,"disable_lazy":false,"disable_mavlink":false,"disable_thumbnails":true,"disable_zenoh":true}}}"##;

pub struct McmStreamRestore {
    base: String,
    snapshot: serde_json::Value,
}

impl McmStreamRestore {
    pub fn snapshot(base: &str) -> Result<Self, String> {
        let base = base.trim_end_matches('/').to_string();
        let streams = get_streams(&base)?;
        Ok(Self {
            base,
            snapshot: serde_json::Value::Array(streams),
        })
    }
}

impl Drop for McmStreamRestore {
    fn drop(&mut self) {
        if let Err(err) = restore_streams(&self.base, &self.snapshot) {
            eprintln!("mcm_restore: restore failed: {err}");
        }
    }
}

fn get_streams(base: &str) -> Result<Vec<serde_json::Value>, String> {
    let (status, body) = execute_curl(
        &HttpMethod::Get,
        &join_url(base, "/mavlink-camera-manager/streams"),
        false,
        None,
        None,
    )?;
    if status != 200 {
        return Err(format!("GET /streams HTTP {status}: {body}"));
    }
    let streams: serde_json::Value =
        serde_json::from_str(&body).map_err(|err| format!("streams JSON: {err}"))?;
    let Some(array) = streams.as_array() else {
        return Err("streams response is not a JSON array".into());
    };
    Ok(array.clone())
}

fn restore_streams(base: &str, snapshot: &serde_json::Value) -> Result<(), String> {
    let current = get_streams(base)?;
    for stream in &current {
        let Some(name) = stream_name(stream) else {
            continue;
        };
        if let Err(err) = delete_stream_by_name(base, &name) {
            eprintln!("mcm_restore: delete {name}: {err}");
        }
    }
    let Some(entries) = snapshot.as_array() else {
        return Ok(());
    };
    for stream in entries {
        match stream_status_to_created_body(stream) {
            Some(body) => {
                if let Err(err) = recreate_stream(base, &body) {
                    eprintln!("mcm_restore: recreate stream: {err}");
                }
            }
            None => eprintln!("mcm_restore: skip unmapped stream entry"),
        }
    }
    Ok(())
}

fn stream_name(stream: &serde_json::Value) -> Option<String> {
    stream
        .get("video_and_stream")?
        .get("name")?
        .as_str()
        .map(str::to_string)
}

fn delete_stream_by_name(base: &str, name: &str) -> Result<(), String> {
    let url = format!(
        "{}/mavlink-camera-manager/delete_stream?name={}",
        base.trim_end_matches('/'),
        url_encode_component(name)
    );
    let (status, body) = execute_curl(&HttpMethod::Delete, &url, true, None, None)?;
    if status != 200 {
        return Err(format!("DELETE /delete_stream HTTP {status}: {body}"));
    }
    Ok(())
}

fn recreate_stream(base: &str, body: &serde_json::Value) -> Result<(), String> {
    let json = serde_json::to_string(body).map_err(|err| err.to_string())?;
    let (status, resp) = execute_curl(
        &HttpMethod::Post,
        &join_url(base, "/mavlink-camera-manager/streams"),
        true,
        Some(&json),
        None,
    )?;
    if status != 200 {
        return Err(format!("POST /streams HTTP {status}: {resp}"));
    }
    Ok(())
}

fn url_encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn video_source_to_post_source(video_source: &serde_json::Value) -> Option<String> {
    if let Some(local) = video_source.get("Local") {
        return local.get("device_path")?.as_str().map(str::to_string);
    }
    if let Some(gst) = video_source.get("Gst") {
        return gst.get("source")?.get("Fake")?.as_str().map(str::to_string);
    }
    if let Some(redirect) = video_source.get("Redirect") {
        return redirect
            .get("source")?
            .get("Redirect")?
            .as_str()
            .map(str::to_string);
    }
    if let Some(onvif) = video_source.get("Onvif") {
        return onvif
            .get("source")?
            .get("Onvif")?
            .as_str()
            .map(str::to_string);
    }
    None
}

struct V4lControlSnapshot {
    v4l_id: i64,
    value: i64,
}

pub struct McmV4lRestore {
    base: String,
    device: String,
    controls: Vec<V4lControlSnapshot>,
}

impl McmV4lRestore {
    pub fn snapshot(base: &str) -> Result<Self, String> {
        let base = base.trim_end_matches('/').to_string();
        let devices = get_v4l_devices(&base)?;
        let device = devices
            .iter()
            .find(|entry| {
                entry
                    .get("name")
                    .and_then(|v| v.as_str())
                    .is_some_and(|n| n.contains("USB"))
            })
            .or_else(|| {
                devices.iter().find(|entry| {
                    entry
                        .get("controls")
                        .map(|c| c.as_array().is_some_and(|a| !a.is_empty()))
                        .unwrap_or(false)
                })
            })
            .cloned()
            .ok_or_else(|| "no UVC device with controls in GET /v4l".to_string())?;
        let device_path = device
            .get("source")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| "UVC device missing source path".to_string())?;
        let controls = v4l_controls_from_device(&device)?;
        Ok(Self {
            base,
            device: device_path,
            controls,
        })
    }

    pub fn device_path(&self) -> &str {
        &self.device
    }

    pub fn mutate_brightness_body(&self) -> Result<String, String> {
        let control = self
            .controls
            .iter()
            .find(|c| c.v4l_id == 9963776)
            .ok_or_else(|| "Brightness control not found on USB camera".to_string())?;
        let value = if control.value < 64 {
            control.value + 1
        } else {
            control.value - 1
        };
        serde_json::to_string(&serde_json::json!({
            "device": self.device,
            "v4l_id": control.v4l_id,
            "value": value,
        }))
        .map_err(|err| err.to_string())
    }
}

impl Drop for McmV4lRestore {
    fn drop(&mut self) {
        for control in &self.controls {
            let body = serde_json::json!({
                "device": self.device,
                "v4l_id": control.v4l_id,
                "value": control.value,
            });
            let json = serde_json::to_string(&body).unwrap_or_default();
            if let Err(err) = post_v4l_control(&self.base, &json) {
                eprintln!("mcm_restore: restore v4l {}: {err}", control.v4l_id);
            }
        }
    }
}

fn get_v4l_devices(base: &str) -> Result<Vec<serde_json::Value>, String> {
    let (status, body) = execute_curl(
        &HttpMethod::Get,
        &join_url(base, "/mavlink-camera-manager/v4l"),
        false,
        None,
        None,
    )?;
    if status != 200 {
        return Err(format!("GET /v4l HTTP {status}: {body}"));
    }
    let devices: serde_json::Value =
        serde_json::from_str(&body).map_err(|err| format!("v4l JSON: {err}"))?;
    let Some(array) = devices.as_array() else {
        return Err("v4l response is not a JSON array".into());
    };
    Ok(array.clone())
}

fn v4l_controls_from_device(device: &serde_json::Value) -> Result<Vec<V4lControlSnapshot>, String> {
    let controls = device
        .get("controls")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "device has no controls array".to_string())?;
    let mut out = Vec::new();
    for control in controls {
        let v4l_id = control
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| "control missing id".to_string())?;
        let config = control
            .get("configuration")
            .ok_or_else(|| "control missing configuration".to_string())?;
        if control
            .get("state")
            .and_then(|s| s.get("is_inactive"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            continue;
        }
        let value = if let Some(slider) = config.get("Slider") {
            slider.get("value").and_then(|v| v.as_i64()).unwrap_or(0)
        } else if let Some(menu) = config.get("Menu") {
            menu.get("value").and_then(|v| v.as_i64()).unwrap_or(0)
        } else if let Some(boolean) = config.get("Bool") {
            boolean
                .get("value")
                .and_then(|v| v.as_bool())
                .map(|b| if b { 1 } else { 0 })
                .unwrap_or(0)
        } else {
            continue;
        };
        out.push(V4lControlSnapshot { v4l_id, value });
    }
    if out.is_empty() {
        return Err("no mutable UVC controls".into());
    }
    Ok(out)
}

fn post_v4l_control(base: &str, json: &str) -> Result<(), String> {
    let (status, body) = execute_curl(
        &HttpMethod::Post,
        &join_url(base, "/mavlink-camera-manager/v4l"),
        true,
        Some(json),
        None,
    )?;
    if status != 200 {
        return Err(format!("POST /v4l HTTP {status}: {body}"));
    }
    Ok(())
}

fn stream_status_to_created_body(stream: &serde_json::Value) -> Option<serde_json::Value> {
    let vas = stream.get("video_and_stream")?;
    let name = vas.get("name")?.as_str()?;
    let stream_information = vas.get("stream_information")?.clone();
    let source = video_source_to_post_source(vas.get("video_source")?)?;
    Some(serde_json::json!({
        "name": name,
        "source": source,
        "stream_information": stream_information,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL_STREAM_FIXTURE: &str = r#"{
      "id": "0",
      "state": "running",
      "video_and_stream": {
        "name": "UDP Stream 0",
        "stream_information": {
          "endpoints": ["udp://192.168.2.1:5600"],
          "configuration": {
            "type": "video",
            "encode": "H264",
            "width": 1920,
            "height": 1080,
            "frame_interval": { "numerator": 1, "denominator": 30 }
          },
          "extended_configuration": {
            "thermal": false,
            "disable_lazy": false,
            "disable_mavlink": false,
            "disable_thumbnails": false,
            "disable_zenoh": false
          }
        },
        "video_source": {
          "Local": {
            "name": "H264 USB Camera: USB Camera",
            "device_path": "/dev/video2",
            "type": { "Usb": "usb" }
          }
        }
      }
    }"#;

    #[test]
    fn local_stream_maps_to_created_body() {
        let stream: serde_json::Value = serde_json::from_str(LOCAL_STREAM_FIXTURE).unwrap();
        let body = stream_status_to_created_body(&stream).expect("mapped");
        assert_eq!(body["name"], "UDP Stream 0");
        assert_eq!(body["source"], "/dev/video2");
        assert_eq!(
            body["stream_information"]["endpoints"],
            serde_json::json!(["udp://192.168.2.1:5600"])
        );
        assert_eq!(
            body["stream_information"]["configuration"]["encode"],
            "H264"
        );
    }
}
