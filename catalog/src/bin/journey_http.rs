// Live HTTP journey runner: `journey_http --base http://<pi> [--fixtures internet,pirate,advanced]`.
// Tier-1 smoke (GET + known status only): `journey_http --base http://<pi> --smoke`.
// Tier-2 mutating smoke (allowlisted reversible journeys): `journey_http --base http://<pi> --mutating-smoke`.
// Offline plan: `journey_http --dry-run` (no --base).
use std::process;

use blueos_catalog::wifi_rf::{self, ApMode};
use blueos_catalog::{
    evaluate_journey, execute_curl, fetch_dut_version, format_dry_run, format_http_fail,
    http_journeys, http_mutating_smoke_steps, http_smoke_steps, http_steps,
    is_mutating_smoke_journey, join_url, journey_availability_skip, journey_fixtures_ready,
    journey_http_mode_conflict, journey_http_requires_base, journey_mutating_smoke_ready,
    mutating_smoke_setup_calls, mutating_smoke_skip_reason, mutating_smoke_teardown_calls,
    parse_fixture_list, resolve_http_path, run_core_image_switch, run_http_step,
    run_smoke_http_call, summarize_journey, utc_rfc3339_now, wait_for_blueos,
    write_journey_http_report, Catalog, DutVersion, FixtureInventory, HttpMethod,
    JourneyHttpReport, JourneyId, JourneyReportEntry, JourneyResult, PreconditionStatus, ReportDut,
    RunCounts, StepResult, SuiteKind, MUTATING_SMOKE_DEFAULT_FIXTURES, SCHEMA_VERSION,
    SMOKE_CORE_MASTER_JSON, SMOKE_CORE_MASTER_TAG, SMOKE_CORE_SWITCH_JSON, SMOKE_CORE_SWITCH_TAG,
    SMOKE_DEFAULT_FIXTURES, TIER2_SMOKE_DUT_CORE_DIGEST,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut base: Option<String> = None;
    let mut fixtures_spec: Option<String> = None;
    let mut dry_run = false;
    let mut allow_mutating = false;
    let mut smoke = false;
    let mut mutating_smoke = false;
    let mut journey_filter: Option<JourneyId> = None;
    let mut report_path: Option<String> = None;
    let mut wifi_modes_spec: Option<String> = None;
    let mut wifi_endpoints = false;

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--base" => {
                index += 1;
                base = Some(
                    args.get(index)
                        .cloned()
                        .unwrap_or_else(|| usage_and_exit("--base requires a URL")),
                );
            }
            "--fixtures" => {
                index += 1;
                fixtures_spec = Some(
                    args.get(index)
                        .cloned()
                        .unwrap_or_else(|| usage_and_exit("--fixtures requires a spec")),
                );
            }
            "--dry-run" => dry_run = true,
            "--allow-mutating" => allow_mutating = true,
            "--smoke" => smoke = true,
            "--mutating-smoke" => mutating_smoke = true,
            "--wifi-modes" => {
                index += 1;
                wifi_modes_spec = Some(
                    args.get(index)
                        .cloned()
                        .unwrap_or_else(|| usage_and_exit("--wifi-modes requires a csv")),
                );
            }
            "--wifi-endpoints" => wifi_endpoints = true,
            "--journey" => {
                index += 1;
                let id = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| usage_and_exit("--journey requires an id"));
                journey_filter = Some(parse_journey_id(&id));
            }
            "--report" => {
                index += 1;
                report_path = Some(
                    args.get(index)
                        .cloned()
                        .unwrap_or_else(|| usage_and_exit("--report requires a path")),
                );
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => usage_and_exit(&format!("unknown argument: {other}")),
        }
        index += 1;
    }

    if let Err(message) = journey_http_mode_conflict(smoke, mutating_smoke) {
        usage_and_exit(message);
    }
    if wifi_endpoints && (smoke || mutating_smoke) {
        usage_and_exit("--wifi-endpoints is mutually exclusive with --smoke and --mutating-smoke");
    }
    if wifi_endpoints && (dry_run || base.is_none()) {
        usage_and_exit("--wifi-endpoints requires --base and cannot use --dry-run");
    }

    if let Err(message) =
        journey_http_requires_base(dry_run, smoke, mutating_smoke, base.as_deref())
    {
        usage_and_exit(message);
    }

    if smoke {
        allow_mutating = false;
    } else if mutating_smoke {
        allow_mutating = true;
    }

    if wifi_endpoints {
        let base = base.as_deref().expect("base checked above");
        match blueos_catalog::run_wifi_endpoints(base) {
            Ok(results) => {
                let failed = results.iter().filter(|(_, ok, _)| !ok).count();
                for (name, ok, detail) in results {
                    println!("{} {name}: {detail}", if ok { "PASS" } else { "FAIL" });
                }
                if failed > 0 {
                    process::exit(1);
                }
                return;
            }
            Err(err) => {
                eprintln!("journey_http: wifi endpoint RF setup: {err}");
                process::exit(1);
            }
        }
    }

    let wifi_modes = match wifi_modes_spec.as_deref() {
        Some(spec) => ApMode::parse_csv(spec).unwrap_or_else(|err| usage_and_exit(&err)),
        None => vec![ApMode::Wpa2],
    };

    let fixtures_label = if let Some(spec) = &fixtures_spec {
        spec.clone()
    } else if smoke {
        SMOKE_DEFAULT_FIXTURES.to_string()
    } else if mutating_smoke {
        MUTATING_SMOKE_DEFAULT_FIXTURES.to_string()
    } else {
        String::new()
    };

    let fixtures = match fixtures_spec {
        Some(spec) => match parse_fixture_list(&spec) {
            Ok(fixtures) => fixtures,
            Err(err) => {
                eprintln!("journey_http: fixtures: {err}");
                process::exit(2);
            }
        },
        None if smoke => match parse_fixture_list(SMOKE_DEFAULT_FIXTURES) {
            Ok(fixtures) => fixtures,
            Err(err) => {
                eprintln!("journey_http: fixtures: {err}");
                process::exit(2);
            }
        },
        None if mutating_smoke => match parse_fixture_list(MUTATING_SMOKE_DEFAULT_FIXTURES) {
            Ok(fixtures) => fixtures,
            Err(err) => {
                eprintln!("journey_http: fixtures: {err}");
                process::exit(2);
            }
        },
        None => FixtureInventory::default(),
    };

    let catalog = Catalog::bootstrap();
    let mut journeys: Vec<_> = http_journeys(&catalog);
    if mutating_smoke {
        journeys.retain(|journey| is_mutating_smoke_journey(journey.id));
        // Forget needs healthy wpa before late hotspot churn; core switch restarts mid-suite;
        // reboot must stay last.
        journeys.sort_by_key(|journey| match journey.id {
            JourneyId::ForgetSavedWifiNetwork => 0,
            JourneyId::SwitchLocalBlueosVersion => 2,
            JourneyId::RebootOnboardComputer => 3,
            _ => 1,
        });
    }
    if let Some(filter) = journey_filter {
        journeys.retain(|journey| journey.id == filter);
        if journeys.is_empty() {
            eprintln!("journey_http: no Http journey with id {filter}");
            process::exit(2);
        }
    }

    let mut totals = RunCounts::default();
    let mut journey_lines: Vec<String> = Vec::new();
    let mut report_journeys: Vec<JourneyReportEntry> = Vec::new();
    let mut any_fail = false;
    let emit_report = report_path.is_some() && !dry_run;
    let started_at = if emit_report {
        Some(utc_rfc3339_now())
    } else {
        None
    };

    if smoke {
        println!(
            "journey_http: smoke — {} Http journeys (fixtures={fixtures_label})",
            journeys.len()
        );
    } else if mutating_smoke {
        println!(
            "journey_http: mutating-smoke — {} allowlisted journeys (fixtures={fixtures_label})",
            journeys.len()
        );
    } else {
        println!(
            "journey_http: {} Http journeys (dry_run={dry_run}, allow_mutating={allow_mutating})",
            journeys.len()
        );
    }
    if let Some(base) = &base {
        println!("base: {base}");
    }

    let dut_version: Option<DutVersion> = if dry_run {
        None
    } else if let Some(base_url) = base.as_deref() {
        match fetch_dut_version(base_url) {
            Ok(dut) => {
                if smoke || mutating_smoke {
                    let digest = dut.digest.as_deref().unwrap_or("(none)");
                    println!("dut: tag={} digest={digest}", dut.tag);
                }
                Some(dut)
            }
            Err(err) => {
                if smoke || mutating_smoke {
                    eprintln!(
                        "journey_http: warning: could not probe DUT version ({err}); availability skips disabled"
                    );
                }
                None
            }
        }
    } else {
        None
    };
    println!();

    for journey in journeys {
        let journey_id = journey.id;
        if mutating_smoke {
            if let Some(reason) = mutating_smoke_skip_reason(journey_id, &fixtures) {
                let step_count = http_mutating_smoke_steps(journey).len().max(1);
                totals.skipped += step_count;
                journey_lines.push(format!("SKIP {journey_id}: {reason}"));
                if emit_report {
                    report_journeys.push(JourneyReportEntry::skipped(
                        journey_id,
                        &journey.availability,
                        reason,
                        step_count,
                    ));
                }
                continue;
            }
        }
        let fixtures_ready = if mutating_smoke {
            journey_mutating_smoke_ready(journey, &fixtures)
        } else {
            journey_fixtures_ready(journey, &fixtures)
        };

        if let Some(dut) = dut_version.as_ref() {
            if let Some(reason) = journey_availability_skip(journey, dut) {
                let step_count = if smoke {
                    http_smoke_steps(journey).len().max(1)
                } else if mutating_smoke {
                    http_mutating_smoke_steps(journey).len().max(1)
                } else {
                    http_steps(journey).len().max(1)
                };
                totals.skipped += step_count;
                journey_lines.push(format!("SKIP {journey_id}: {reason}"));
                if emit_report {
                    report_journeys.push(JourneyReportEntry::skipped(
                        journey_id,
                        &journey.availability,
                        reason,
                        step_count,
                    ));
                }
                continue;
            }
        }

        if !fixtures_ready {
            let reasons = skip_reasons(journey, &fixtures);
            let step_count = if smoke {
                http_smoke_steps(journey).len().max(1)
            } else if mutating_smoke {
                http_mutating_smoke_steps(journey).len().max(1)
            } else {
                http_steps(journey).len().max(1)
            };
            totals.skipped += step_count;
            let reason = reasons.join("; ");
            journey_lines.push(format!("SKIP {journey_id}: {reason}"));
            if emit_report {
                report_journeys.push(JourneyReportEntry::skipped(
                    journey_id,
                    &journey.availability,
                    reason,
                    step_count,
                ));
            }
            continue;
        }

        let steps = if smoke {
            http_smoke_steps(journey)
        } else if mutating_smoke {
            http_mutating_smoke_steps(journey)
        } else {
            http_steps(journey)
        };
        if steps.is_empty() && !(mutating_smoke && wifi_rf::is_rf_status_journey(journey_id)) {
            let reason = if smoke {
                "no smoke-eligible GET steps with expected_status"
            } else if mutating_smoke {
                "no mutating-smoke-eligible steps with expected_status"
            } else {
                "no runnable HTTP steps"
            };
            journey_lines.push(format!("SKIP {journey_id}: {reason}"));
            totals.skipped += 1;
            if emit_report {
                report_journeys.push(JourneyReportEntry::skipped(
                    journey_id,
                    &journey.availability,
                    reason,
                    1,
                ));
            }
            continue;
        }

        if dry_run {
            for step in &steps {
                let resolved = resolve_http_path(&catalog, &step.route)
                    .unwrap_or_else(|| "(unresolved)".to_string());
                println!("{}", format_dry_run(journey_id, step, &resolved));
                totals.skipped += 1;
            }
            journey_lines.push(format!("DRY-RUN {journey_id}: {} step(s)", steps.len()));
            continue;
        }

        let base = base.as_deref().expect("base checked above");
        let mut step_results = Vec::new();
        let mut hotspot_creds_snapshot: Option<String> = None;

        if mutating_smoke {
            if journey_id == JourneyId::ConfigureHotspotCredentials {
                match wifi_rf::fetch_hotspot_credentials_json(base) {
                    Ok(body) => hotspot_creds_snapshot = Some(body),
                    Err(err) => {
                        eprintln!("FAIL {journey_id} hotspot credentials snapshot — {err}");
                        totals.failed += 1;
                        step_results.push(StepResult::Fail(err));
                        any_fail = true;
                        journey_lines
                            .push(format!("Fail {journey_id}: hotspot credentials snapshot"));
                        if emit_report {
                            report_journeys.push(JourneyReportEntry::from_run(
                                journey_id,
                                &journey.availability,
                                JourneyResult::Fail,
                                &step_results,
                            ));
                        }
                        continue;
                    }
                }
            }
            let rf_setup = if journey_id == JourneyId::ConnectToWifiNetwork {
                wifi_rf::dut_hotspot_off(base);
                wifi_rf::host_ap_ensure()
            } else {
                wifi_rf::rf_setup_for_base(journey_id, base)
            };
            if let Err(err) = rf_setup {
                eprintln!("FAIL {journey_id} wifi RF setup — {err}");
                totals.failed += 1;
                step_results.push(StepResult::Fail(format!("wifi RF setup — {err}")));
                any_fail = true;
                journey_lines.push(format!("Fail {journey_id}: wifi RF setup"));
                if emit_report {
                    report_journeys.push(JourneyReportEntry::from_run(
                        journey_id,
                        &journey.availability,
                        JourneyResult::Fail,
                        &step_results,
                    ));
                }
                let _ = wifi_rf::rf_teardown(journey_id);
                continue;
            }
            for call in mutating_smoke_setup_calls(journey_id) {
                let result = run_smoke_http_call(&catalog, base, call, allow_mutating);
                if let StepResult::Fail(msg) = &result {
                    eprintln!(
                        "FAIL {journey_id} setup {:?} {} — {msg}",
                        call.route.method, call.route.path
                    );
                }
                totals.record(&result);
                step_results.push(result);
            }
        }

        if mutating_smoke && journey_id == JourneyId::ConnectToWifiNetwork {
            for mode in &wifi_modes {
                let ssid = wifi_rf::mode_ssid(*mode);
                let result = run_wifi_mode(base, *mode);
                match result {
                    Ok(ip) => {
                        eprintln!("journey_http: WiFi {mode} connect+L3 ok ip={ip}");
                        totals.passed += 1;
                        step_results.push(StepResult::Pass);
                    }
                    Err(err) => {
                        eprintln!("FAIL {journey_id} WiFi {mode} — {err}");
                        totals.failed += 1;
                        step_results.push(StepResult::Fail(err));
                    }
                }
                wifi_rf::disconnect_and_remove(base, &ssid);
            }
        } else if mutating_smoke && wifi_rf::is_rf_status_journey(journey_id) {
            let rf_result = match journey_id {
                JourneyId::DetectWifiApLoss => wifi_rf::run_detect_ap_loss(base),
                JourneyId::AutoconnectToSavedWifiNetwork => wifi_rf::run_autoconnect(base),
                _ => unreachable!("is_rf_status_journey"),
            };
            match rf_result {
                Ok(()) => {
                    eprintln!("journey_http: wifi RF mid-journey ok for {journey_id}");
                    totals.passed += 1;
                    step_results.push(StepResult::Pass);
                    if journey_id == JourneyId::AutoconnectToSavedWifiNetwork {
                        match wifi_rf::l3_assert_client_lease(base) {
                            Ok(ip) => {
                                eprintln!("journey_http: L3 client lease+ping ok ip={ip}");
                                totals.passed += 1;
                                step_results.push(StepResult::Pass);
                            }
                            Err(err) => {
                                eprintln!("FAIL {journey_id} L3 client — {err}");
                                totals.failed += 1;
                                step_results.push(StepResult::Fail(err));
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("FAIL {journey_id} wifi RF mid-journey — {err}");
                    totals.failed += 1;
                    step_results.push(StepResult::Fail(err));
                }
            }
        } else {
            for step in &steps {
                let resolved_url = resolve_http_path(&catalog, &step.route)
                    .map(|path| join_url(base, &path))
                    .unwrap_or_else(|| "(unresolved)".to_string());
                let result = if mutating_smoke
                    && journey_id == JourneyId::SwitchLocalBlueosVersion
                    && step.route.path == "/version/current"
                {
                    run_core_image_switch(
                        &catalog,
                        base,
                        SMOKE_CORE_SWITCH_JSON,
                        SMOKE_CORE_SWITCH_TAG,
                        allow_mutating,
                    )
                } else {
                    run_http_step(&catalog, base, step, allow_mutating)
                };
                if let StepResult::Fail(msg) = &result {
                    eprintln!("{}", format_http_fail(journey_id, step, &resolved_url, msg));
                }
                totals.record(&result);
                step_results.push(result);
                if mutating_smoke
                    && journey_id == JourneyId::ConfigureHotspotCredentials
                    && step.route.path == "/hotspot_credentials"
                    && matches!(step_results.last(), Some(StepResult::Pass))
                {
                    match wifi_rf::fetch_hotspot_credentials_json(base) {
                        Ok(body) if body.contains(wifi_rf::SMOKE_HOTSPOT_SSID) => {
                            eprintln!("journey_http: hotspot credentials mutate verified");
                            totals.passed += 1;
                            step_results.push(StepResult::Pass);
                        }
                        Ok(body) => {
                            let err = format!(
                                "expected ssid {} in credentials after mutate, got {body}",
                                wifi_rf::SMOKE_HOTSPOT_SSID
                            );
                            eprintln!("FAIL {journey_id} credentials verify — {err}");
                            totals.failed += 1;
                            step_results.push(StepResult::Fail(err));
                        }
                        Err(err) => {
                            eprintln!("FAIL {journey_id} credentials verify — {err}");
                            totals.failed += 1;
                            step_results.push(StepResult::Fail(err));
                        }
                    }
                }
                if mutating_smoke
                    && wifi_rf::wants_client_l3(journey_id)
                    && step.route.path == "/connect"
                    && matches!(step_results.last(), Some(StepResult::Pass))
                {
                    match wifi_rf::l3_assert_client_lease(base) {
                        Ok(ip) => {
                            eprintln!("journey_http: L3 client lease+ping ok ip={ip}");
                            totals.passed += 1;
                            step_results.push(StepResult::Pass);
                        }
                        Err(err) => {
                            eprintln!("FAIL {journey_id} L3 client — {err}");
                            totals.failed += 1;
                            step_results.push(StepResult::Fail(err));
                        }
                    }
                }
                if mutating_smoke
                    && journey_id == JourneyId::DisconnectFromWifiNetwork
                    && step.route.path == "/disconnect"
                    && matches!(step_results.last(), Some(StepResult::Pass))
                {
                    match wifi_rf::run_assert_disconnected(base) {
                        Ok(()) => {
                            eprintln!("journey_http: disconnect confirmed via /status");
                            totals.passed += 1;
                            step_results.push(StepResult::Pass);
                        }
                        Err(err) => {
                            eprintln!("FAIL {journey_id} disconnect status — {err}");
                            totals.failed += 1;
                            step_results.push(StepResult::Fail(err));
                        }
                    }
                }
                if mutating_smoke
                    && journey_id == JourneyId::ToggleHotspot
                    && step.route.path == "/hotspot"
                    && matches!(step_results.last(), Some(StepResult::Pass))
                {
                    match wifi_rf::rf_verify_hotspot_join() {
                        Ok(lease) => {
                            eprintln!("journey_http: host joined BlueOS hotspot lease={lease}");
                            totals.passed += 1;
                            step_results.push(StepResult::Pass);
                            match wifi_rf::l3_assert_hotspot_gateway() {
                                Ok(()) => {
                                    eprintln!(
                                        "journey_http: L3 hotspot ping {} ok",
                                        wifi_rf::SMOKE_HOTSPOT_GATEWAY
                                    );
                                    totals.passed += 1;
                                    step_results.push(StepResult::Pass);
                                }
                                Err(err) => {
                                    eprintln!("FAIL {journey_id} L3 hotspot — {err}");
                                    totals.failed += 1;
                                    step_results.push(StepResult::Fail(err));
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("FAIL {journey_id} hotspot RF join — {err}");
                            totals.failed += 1;
                            step_results.push(StepResult::Fail(err));
                        }
                    }
                }
                if mutating_smoke
                    && journey_id == JourneyId::RebootOnboardComputer
                    && matches!(step_results.last(), Some(StepResult::Pass))
                {
                    eprintln!("journey_http: waiting for BlueOS after reboot…");
                    if let Err(err) = wait_for_blueos(base, 600) {
                        eprintln!("FAIL {journey_id} recovery — {err}");
                        totals.failed += 1;
                        step_results.push(StepResult::Fail(err));
                    } else {
                        totals.passed += 1;
                        step_results.push(StepResult::Pass);
                    }
                }
            }
        }

        if mutating_smoke {
            if journey_id == JourneyId::SwitchLocalBlueosVersion {
                eprintln!(
                    "journey_http: restoring core image to tag `{SMOKE_CORE_MASTER_TAG}` (intended digest {TIER2_SMOKE_DUT_CORE_DIGEST})…"
                );
                let result = run_core_image_switch(
                    &catalog,
                    base,
                    SMOKE_CORE_MASTER_JSON,
                    SMOKE_CORE_MASTER_TAG,
                    allow_mutating,
                );
                if let StepResult::Fail(msg) = &result {
                    eprintln!("FAIL {journey_id} teardown core restore — {msg}");
                }
                totals.record(&result);
                step_results.push(result);
            }
            for call in mutating_smoke_teardown_calls(journey_id) {
                let result = run_smoke_http_call(&catalog, base, call, allow_mutating);
                if let StepResult::Fail(msg) = &result {
                    eprintln!(
                        "FAIL {journey_id} teardown {:?} {} — {msg}",
                        call.route.method, call.route.path
                    );
                }
                totals.record(&result);
                step_results.push(result);
            }
            if let Some(body) = hotspot_creds_snapshot.as_deref() {
                match wifi_rf::post_hotspot_credentials_json(base, body) {
                    Ok(()) => {
                        eprintln!("journey_http: restored hotspot credentials snapshot");
                        totals.passed += 1;
                        step_results.push(StepResult::Pass);
                    }
                    Err(err) => {
                        eprintln!("FAIL {journey_id} hotspot credentials restore — {err}");
                        totals.failed += 1;
                        step_results.push(StepResult::Fail(err));
                    }
                }
            }
            if let Err(err) = wifi_rf::rf_teardown(journey_id) {
                eprintln!("FAIL {journey_id} wifi RF teardown — {err}");
                totals.failed += 1;
                step_results.push(StepResult::Fail(format!("wifi RF teardown — {err}")));
            }
        }

        let outcome = summarize_journey(&step_results);
        if outcome == JourneyResult::Fail {
            any_fail = true;
        }
        journey_lines.push(format!("{outcome:?} {journey_id}: {} step(s)", steps.len()));
        if emit_report {
            report_journeys.push(JourneyReportEntry::from_run(
                journey_id,
                &journey.availability,
                outcome,
                &step_results,
            ));
        }
    }

    println!();
    println!(
        "summary: passed={} failed={} skipped={} unasserted={}",
        totals.passed, totals.failed, totals.skipped, totals.unasserted
    );
    for line in journey_lines {
        println!("  {line}");
    }

    if let Some(path) = report_path {
        if !dry_run {
            let suite = if smoke {
                SuiteKind::Smoke
            } else if mutating_smoke {
                SuiteKind::MutatingSmoke
            } else {
                SuiteKind::Full
            };
            let report = JourneyHttpReport {
                schema_version: SCHEMA_VERSION,
                suite,
                base: base.clone().unwrap_or_default(),
                dut: dut_version.as_ref().map(ReportDut::from_dut),
                started_at: started_at.unwrap_or_else(utc_rfc3339_now),
                finished_at: utc_rfc3339_now(),
                counts: totals.into(),
                journeys: report_journeys,
            };
            if let Err(err) = write_journey_http_report(&path, &report) {
                eprintln!("journey_http: report: {err}");
                process::exit(2);
            }
        }
    }

    if any_fail || totals.failed > 0 {
        process::exit(1);
    }
}

fn skip_reasons(journey: &blueos_catalog::UserJourney, fixtures: &FixtureInventory) -> Vec<String> {
    evaluate_journey(journey, fixtures)
        .into_iter()
        .filter_map(|status| match status {
            PreconditionStatus::Satisfied => None,
            PreconditionStatus::Missing(reason) => Some(reason),
            PreconditionStatus::Unevaluable(reason) => Some(format!("unevaluable: {reason}")),
        })
        .collect()
}

fn parse_journey_id(raw: &str) -> JourneyId {
    JourneyId::ALL
        .iter()
        .copied()
        .find(|id| id.as_str() == raw)
        .unwrap_or_else(|| usage_and_exit(&format!("unknown journey id: {raw}")))
}

fn usage_and_exit(message: &str) -> ! {
    eprintln!("journey_http: {message}");
    print_help();
    process::exit(2);
}

fn print_help() {
    eprintln!(
        "usage: journey_http --base <url> [--fixtures internet,pirate,advanced] [--smoke | --mutating-smoke | --wifi-endpoints] [--wifi-modes open,wpa,wpa2,transition,wpa3] [--dry-run] [--allow-mutating] [--journey <id>] [--report <path.json>]"
    );
}

fn run_wifi_mode(base: &str, mode: ApMode) -> Result<String, String> {
    let ssid = wifi_rf::mode_ssid(mode);
    wifi_rf::dut_hotspot_off(base);
    let _ = wifi_rf::host_ap_down();
    wifi_rf::assert_scan_absent(base, &ssid, std::time::Duration::from_secs(30))?;
    wifi_rf::host_ap_up(mode.as_str())?;
    wifi_rf::assert_scan_present(base, &ssid, std::time::Duration::from_secs(45))?;
    let (status, body) = execute_curl(
        &HttpMethod::Post,
        &format!("{base}/wifi-manager/v1.0/connect?hidden=false"),
        true,
        Some(&wifi_rf::connect_body_for_mode(mode)),
        None,
    )?;
    if mode == ApMode::Wpa3 && !wifi_rf::expect_wpa3_join(base)? {
        if status != 200 || wifi_rf::wait_dut_lease(base, &ssid, 5).is_err() {
            return Ok("WPA3 rejected as expected".into());
        }
        return Err("WPA3 connected although EXPECT_WPA3 says it must reject".into());
    }
    if status != 200 {
        return Err(format!("POST /connect HTTP {status}: {body}"));
    }
    wifi_rf::l3_assert_associated(base, mode)
}
