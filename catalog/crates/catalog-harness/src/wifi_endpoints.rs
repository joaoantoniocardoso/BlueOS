use crate::runner::execute_curl;
use crate::wifi_rf::{
    connect_body_for_mode, dut_hotspot_off, dut_status_ssid, host_ap_ensure, host_ap_up,
    l3_assert_associated, mode_ssid, rf_teardown, wait_dut_lease, ApMode,
};
use catalog_kernel::id::journey::JourneyId;
use catalog_model::journey::HttpMethod;

fn call(
    base: &str,
    method: HttpMethod,
    path: &str,
    body: Option<&str>,
) -> Result<(u16, String), String> {
    execute_curl(
        &method,
        &format!("{}/wifi-manager/v1.0/{path}", base.trim_end_matches('/')),
        true,
        body,
        None,
    )
}

fn record(
    results: &mut Vec<(String, bool, String)>,
    name: &str,
    result: Result<(u16, String), String>,
    expected: impl Fn(u16) -> bool,
) {
    match result {
        Ok((status, _)) if expected(status) => {
            results.push((name.into(), true, format!("HTTP {status}")))
        }
        Ok((status, body)) => results.push((name.into(), false, format!("HTTP {status}: {body}"))),
        Err(err) => results.push((name.into(), false, err)),
    }
}

/// Exercises the documented wifi-manager HTTP surface while the host provides a WPA2 AP.
pub fn run_wifi_endpoints(base: &str) -> Result<Vec<(String, bool, String)>, String> {
    dut_hotspot_off(base);
    std::thread::sleep(std::time::Duration::from_secs(2));
    host_ap_ensure()?;
    host_ap_up("wpa2")?;
    let mut results = Vec::new();
    let snapshot = crate::wifi_rf::fetch_hotspot_credentials_json(base).ok();
    let smart_before = call(base, HttpMethod::Get, "smart_hotspot", None)
        .ok()
        .and_then(|(status, body)| (status == 200).then_some(body.trim().to_ascii_lowercase()))
        .filter(|body| body == "true" || body == "false");

    record(
        &mut results,
        "status",
        call(base, HttpMethod::Get, "status", None),
        |status| status == 200,
    );
    record(
        &mut results,
        "scan",
        call(base, HttpMethod::Get, "scan", None),
        |status| status == 200 || status == 425,
    );
    record(
        &mut results,
        "saved",
        call(base, HttpMethod::Get, "saved", None),
        |status| status == 200,
    );
    record(
        &mut results,
        "connect_right",
        call(
            base,
            HttpMethod::Post,
            "connect?hidden=false",
            Some(&connect_body_for_mode(ApMode::Wpa2)),
        ),
        |status| status == 200,
    );
    match l3_assert_associated(base, ApMode::Wpa2) {
        Ok(ip) => results.push(("connect_l3".into(), true, ip)),
        Err(err) => results.push(("connect_l3".into(), false, err)),
    }
    record(
        &mut results,
        "disconnect",
        call(base, HttpMethod::Get, "disconnect", None),
        |status| status == 200 || status == 500,
    );
    record(
        &mut results,
        "connect_wrong",
        call(
            base,
            HttpMethod::Post,
            "connect?hidden=false",
            Some(&crate::wifi_rf::wrong_password_body_for_mode(ApMode::Wpa2)),
        ),
        |status| status != 200,
    );
    let no_lease = wait_dut_lease(base, &mode_ssid(ApMode::Wpa2), 5).is_err()
        && dut_status_ssid(base).ok().flatten().as_deref()
            != Some(mode_ssid(ApMode::Wpa2).as_str());
    results.push((
        "connect_wrong_no_lease".into(),
        no_lease,
        if no_lease {
            "no WPA2 lease".into()
        } else {
            "DUT retained WPA2 lease".into()
        },
    ));
    record(
        &mut results,
        "remove_unknown",
        call(
            base,
            HttpMethod::Post,
            "remove?ssid=__catalog_unknown_wifi__",
            None,
        ),
        |status| status == 400,
    );
    record(
        &mut results,
        "hotspot_get",
        call(base, HttpMethod::Get, "hotspot", None),
        |status| status == 200,
    );
    record(
        &mut results,
        "hotspot_post",
        call(base, HttpMethod::Post, "hotspot?enable=false", None),
        |status| status == 200,
    );
    record(
        &mut results,
        "smart_hotspot_get",
        call(base, HttpMethod::Get, "smart_hotspot", None),
        |status| status == 200,
    );
    record(
        &mut results,
        "smart_hotspot_post",
        call(base, HttpMethod::Post, "smart_hotspot?enable=false", None),
        |status| status == 200,
    );
    record(
        &mut results,
        "credentials_get",
        call(base, HttpMethod::Get, "hotspot_credentials", None),
        |status| status == 200,
    );
    record(
        &mut results,
        "credentials_post",
        call(
            base,
            HttpMethod::Post,
            "hotspot_credentials",
            Some(crate::wifi_rf::hotspot_credentials_smoke_body()),
        ),
        |status| status == 200 || status == 0,
    );
    if let Some(snapshot) = snapshot.as_deref() {
        record(
            &mut results,
            "credentials_post_restore",
            call(
                base,
                HttpMethod::Post,
                "hotspot_credentials",
                Some(snapshot),
            ),
            |status| status == 200 || status == 0,
        );
    }
    // Restore prior smart-hotspot flag (do not force enable=true — that leaves soft-AP
    // up and breaks the next DUT's client RF). Always leave hotspot off for hygiene.
    let smart_restore = smart_before.as_deref().unwrap_or("false");
    record(
        &mut results,
        "smart_hotspot_restore",
        call(
            base,
            HttpMethod::Post,
            &format!("smart_hotspot?enable={smart_restore}"),
            None,
        ),
        |status| status == 200,
    );
    dut_hotspot_off(base);
    let _ = call(
        base,
        HttpMethod::Post,
        &format!("remove?ssid={}", mode_ssid(ApMode::Wpa2)),
        None,
    );

    let _ = rf_teardown(JourneyId::ConnectToWifiNetwork);
    Ok(results)
}
