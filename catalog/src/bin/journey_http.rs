// Live HTTP journey runner: `journey_http --base http://<pi> [--fixtures internet,pirate,advanced]`.
// Tier-1 smoke (GET + known status only): `journey_http --base http://<pi> --smoke`.
// Tier-2 mutating smoke (allowlisted reversible journeys): `journey_http --base http://<pi> --mutating-smoke`.
// Frontend PWA/cache contract: `journey_http --base http://<pi> --frontend-cache`.
// Offline plan: `journey_http --dry-run` (no --base).
use std::io::Write;
use std::path::Path;
use std::process;
use std::time::Duration;

use blueos_catalog::provenance::Grounded;
use blueos_catalog::runner::run_http_step_detailed;
use blueos_catalog::sitl_cal::{self, BoardRestore};
use blueos_catalog::wifi_rf::{self, ApMode};
use blueos_catalog::{
    dut_profile_for_host, dut_version_current_json, effect_read_after, effect_read_before,
    effect_read_enabled, effect_read_probe_journey, evaluate_http_response, evaluate_journey,
    execute_curl, fetch_dut_version, format_dry_run, format_http_fail, format_negative_dry_run,
    http_journeys, http_mutating_smoke_steps, http_smoke_steps, http_steps,
    is_camera_mutating_smoke_journey, is_mutating_smoke_journey, join_url,
    journey_availability_skip, journey_fixtures_ready, journey_http_mode_conflict,
    journey_http_requires_base, journey_mutating_smoke_ready, journey_profile_skip,
    mutating_effect_read_phases, mutating_smoke_setup_calls, mutating_smoke_skip_reason,
    mutating_smoke_teardown_calls, negative_probe_url, parse_fixture_list, resolve_http_path,
    run_core_image_switch, run_frontend_cache, run_negative_probe, run_smoke_http_call,
    summarize_journey, ui_fixture_skip_reason, ui_plan, ui_suite_plans, utc_rfc3339_now,
    wait_for_blueos, wizard_skip_plan, write_journey_http_report, BlastRadius, Catalog,
    ConflictKind, DutProfile, DutVersion, EffectReadBefore, EffectReadBeforeResult,
    FixtureInventory, HttpMethod, JourneyHttpReport, JourneyId, JourneyReportEntry, JourneyResult,
    McmStreamRestore, McmV4lRestore, NegativeProbe, PreconditionStatus, ReportConflict, ReportDut,
    RunCounts, RunnableStep, StepResult, SuiteKind, UiJourneyPlan, UserJourney,
    MUTATING_SMOKE_DEFAULT_FIXTURES, MUTATING_SMOKE_ENTRIES, NEGATIVE_PROBES, SCHEMA_VERSION,
    SMOKE_CATALOG_STREAM_JSON, SMOKE_CORE_SWITCH_JSON, SMOKE_CORE_SWITCH_TAG,
    SMOKE_DEFAULT_FIXTURES, UI_CAMERA_JOURNEYS,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut base: Option<String> = None;
    let mut fixtures_spec: Option<String> = None;
    let mut dry_run = false;
    let mut allow_mutating = false;
    let mut smoke = false;
    let mut mutating_smoke = false;
    let mut negative = false;
    let mut ui = false;
    let mut journey_filter: Option<JourneyId> = None;
    let mut report_path: Option<String> = None;
    let mut wifi_modes_spec: Option<String> = None;
    let mut wifi_endpoints = false;
    let mut frontend_cache = false;

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
            "--negative" => negative = true,
            "--ui" => ui = true,
            "--wifi-modes" => {
                index += 1;
                wifi_modes_spec = Some(
                    args.get(index)
                        .cloned()
                        .unwrap_or_else(|| usage_and_exit("--wifi-modes requires a csv")),
                );
            }
            "--wifi-endpoints" => wifi_endpoints = true,
            "--frontend-cache" => frontend_cache = true,
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

    if let Err(message) = journey_http_mode_conflict(smoke, mutating_smoke, negative, ui) {
        usage_and_exit(message);
    }
    if wifi_endpoints && (smoke || mutating_smoke || negative || ui || frontend_cache) {
        usage_and_exit(
            "--wifi-endpoints is mutually exclusive with --smoke, --mutating-smoke, --negative, --ui, and --frontend-cache",
        );
    }
    if frontend_cache && (smoke || mutating_smoke || negative || ui) {
        usage_and_exit(
            "--frontend-cache is mutually exclusive with --smoke, --mutating-smoke, --negative, and --ui",
        );
    }
    if wifi_endpoints && (dry_run || base.is_none()) {
        usage_and_exit("--wifi-endpoints requires --base and cannot use --dry-run");
    }
    if frontend_cache && (dry_run || base.is_none()) {
        usage_and_exit("--frontend-cache requires --base and cannot use --dry-run");
    }

    if let Err(message) = journey_http_requires_base(
        dry_run,
        smoke,
        mutating_smoke,
        negative,
        ui,
        base.as_deref(),
    ) {
        usage_and_exit(message);
    }

    if smoke {
        allow_mutating = false;
    } else if mutating_smoke || negative || ui {
        allow_mutating = true;
    }

    if ui {
        let fixtures_label = fixtures_spec
            .clone()
            .unwrap_or_else(|| SMOKE_DEFAULT_FIXTURES.to_string());
        let fixtures = match fixtures_spec {
            Some(spec) => match parse_fixture_list(&spec) {
                Ok(fixtures) => fixtures,
                Err(err) => {
                    eprintln!("journey_http: fixtures: {err}");
                    process::exit(2);
                }
            },
            None => match parse_fixture_list(SMOKE_DEFAULT_FIXTURES) {
                Ok(fixtures) => fixtures,
                Err(err) => {
                    eprintln!("journey_http: fixtures: {err}");
                    process::exit(2);
                }
            },
        };
        run_ui_suite(
            base.as_deref(),
            dry_run,
            journey_filter,
            report_path.as_deref(),
            &fixtures_label,
            &fixtures,
        );
        return;
    }

    if negative {
        run_negative_probes(
            base.as_deref(),
            dry_run,
            journey_filter,
            report_path.as_deref(),
        );
        return;
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

    if frontend_cache {
        let base = base.as_deref().expect("base checked above");
        println!("journey_http: frontend-cache");
        println!("base: {base}");
        let results = run_frontend_cache(base);
        let failed = results.iter().filter(|(_, ok, _)| !ok).count();
        let passed = results.len() - failed;
        for (name, ok, detail) in &results {
            println!("{} {name}: {detail}", if *ok { "PASS" } else { "FAIL" });
        }
        println!("frontend-cache: passed={passed} failed={failed}");
        if failed > 0 {
            process::exit(1);
        }
        return;
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
    let mut journeys: Vec<_> = if mutating_smoke {
        catalog
            .journeys()
            .iter()
            .filter(|journey| is_mutating_smoke_journey(journey.id))
            .collect()
    } else {
        http_journeys(&catalog)
    };
    if mutating_smoke {
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
            eprintln!(
                "journey_http: no {} journey with id {filter}",
                if mutating_smoke {
                    "mutating-smoke"
                } else {
                    "Http"
                }
            );
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

    let dut_profile = base.as_deref().map(dut_profile_for_host);

    let needs_mcm_stream_restore = mutating_smoke
        && !dry_run
        && base.is_some()
        && journeys
            .iter()
            .any(|journey| is_camera_mutating_smoke_journey(journey.id));
    let mcm_stream_restore = if needs_mcm_stream_restore {
        match McmStreamRestore::snapshot(base.as_deref().expect("base for MCM snapshot")) {
            Ok(restore) => Some(restore),
            Err(err) => {
                eprintln!("journey_http: snapshot MCM streams: {err}");
                process::exit(1);
            }
        }
    } else {
        None
    };

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

        if let Some(profile) = dut_profile.as_ref() {
            if let Some(reason) = journey_profile_skip(journey, profile) {
                let step_count = journey_step_count(journey, smoke, mutating_smoke);
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

        let availability_skip_reason = dut_version
            .as_ref()
            .and_then(|dut| journey_availability_skip(journey, dut));

        if let Some(reason) = availability_skip_reason.as_ref() {
            if ghost_presence_eligible(smoke, dut_profile.as_ref(), journey) {
                let steps = http_smoke_steps(journey);
                let base = base.as_deref().expect("base checked above");
                let (outcome, step_results, conflicts) =
                    run_ghost_presence_probes(&catalog, base, reason, &steps);
                for conflict in &conflicts {
                    eprintln!(
                        "ADVISORY {journey_id}: {:?} — {}",
                        conflict.kind, conflict.context
                    );
                    journey_lines.push(format!(
                        "ADVISORY {journey_id}: {:?} — {}",
                        conflict.kind, conflict.context
                    ));
                }
                for result in &step_results {
                    totals.record(result);
                }
                if emit_report {
                    let mut entry = JourneyReportEntry::from_run(
                        journey_id,
                        &journey.availability,
                        outcome,
                        &step_results,
                    );
                    entry.conflicts = conflicts;
                    report_journeys.push(entry);
                }
                continue;
            }

            totals.skipped += journey_step_count(journey, smoke, mutating_smoke);
            journey_lines.push(format!("SKIP {journey_id}: {reason}"));
            if emit_report {
                report_journeys.push(JourneyReportEntry::skipped(
                    journey_id,
                    &journey.availability,
                    reason.clone(),
                    journey_step_count(journey, smoke, mutating_smoke),
                ));
            }
            continue;
        }

        if !fixtures_ready {
            let reasons = skip_reasons(journey, &fixtures);
            let step_count = journey_step_count(journey, smoke, mutating_smoke);
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
        if steps.is_empty()
            && !(mutating_smoke
                && (wifi_rf::is_rf_status_journey(journey_id)
                    || is_camera_mutating_smoke_journey(journey_id)))
        {
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
        let mut journey_conflicts: Vec<ReportConflict> = Vec::new();
        let mut hotspot_creds_snapshot: Option<String> = None;
        let effect_step_index = mutating_smoke
            .then(|| {
                MUTATING_SMOKE_ENTRIES
                    .iter()
                    .find(|entry| entry.journey_id == journey_id)
                    .and_then(|entry| entry.effect_read.as_ref())
                    .map(|effect| effect.step_index)
            })
            .flatten();
        let effect_read_active =
            effect_read_enabled(effect_step_index, wifi_rf::is_rf_status_journey(journey_id));
        let effect_phases = mutating_effect_read_phases(effect_read_active);
        let mut effect_read_state: Option<EffectReadBefore> = None;
        let effect_probe_journey = effect_read_probe_journey(journey_id)
            .and_then(|probe_id| catalog.journeys().iter().find(|j| j.id == probe_id))
            .unwrap_or(journey);
        let mut v4l_restore: Option<McmV4lRestore> = None;

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
            if journey_id == JourneyId::ConfigureUvcDeviceControls {
                match McmV4lRestore::snapshot(base) {
                    Ok(restore) => v4l_restore = Some(restore),
                    Err(err) => {
                        eprintln!("FAIL {journey_id} UVC snapshot — {err}");
                        totals.failed += 1;
                        step_results.push(StepResult::Fail(err));
                        any_fail = true;
                        journey_lines.push(format!("Fail {journey_id}: UVC snapshot"));
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
            if effect_phases.contains(&"before") {
                let step_index = effect_step_index.expect("before phase implies step_index");
                match effect_read_before(&catalog, base, effect_probe_journey, step_index) {
                    EffectReadBeforeResult::Ready(state) => {
                        eprintln!(
                            "journey_http: effect_read before GET {} step {}",
                            state.probe.route.path, state.probe.step_index
                        );
                        effect_read_state = Some(state);
                    }
                    EffectReadBeforeResult::Failed(result) => {
                        if let StepResult::Fail(msg) = &result {
                            eprintln!("FAIL {journey_id} effect_read before — {msg}");
                        }
                        totals.record(&result);
                        step_results.push(result);
                    }
                    EffectReadBeforeResult::Skipped => {}
                }
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
        } else if mutating_smoke && journey_id == JourneyId::ConfigureCameraStream {
            let url = join_url(base, "/mavlink-camera-manager/streams");
            let result = match execute_curl(
                &HttpMethod::Post,
                &url,
                allow_mutating,
                Some(SMOKE_CATALOG_STREAM_JSON),
                None,
            ) {
                Ok((status, body)) => evaluate_http_response(status, &body, Some(200), None),
                Err(err) => StepResult::Fail(err),
            };
            if let StepResult::Fail(msg) = &result {
                eprintln!("FAIL {journey_id} POST /streams — {msg}");
            }
            totals.record(&result);
            step_results.push(result);
        } else if mutating_smoke && journey_id == JourneyId::ConfigureUvcDeviceControls {
            let restore = v4l_restore.as_ref().expect("UVC snapshot");
            let result = match restore.mutate_brightness_body() {
                Ok(body) => {
                    let url = join_url(base, "/mavlink-camera-manager/v4l");
                    match execute_curl(&HttpMethod::Post, &url, allow_mutating, Some(&body), None) {
                        Ok((status, resp)) => {
                            evaluate_http_response(status, &resp, Some(200), None)
                        }
                        Err(err) => StepResult::Fail(err),
                    }
                }
                Err(err) => StepResult::Fail(err),
            };
            if let StepResult::Fail(msg) = &result {
                eprintln!("FAIL {journey_id} POST /v4l — {msg}");
            }
            totals.record(&result);
            step_results.push(result);
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
                    finalize_http_step_run(
                        &mut journey_conflicts,
                        run_http_step_detailed(&catalog, base, step, allow_mutating),
                    )
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

        drop(v4l_restore);

        if mutating_smoke && effect_phases.contains(&"after") {
            if let Some(before) = effect_read_state {
                let after = effect_read_after(&catalog, base, journey_id, &before);
                if let Some(conflict) = &after.conflict {
                    eprintln!("FAIL {journey_id} effect_read — {}", conflict.context);
                    journey_conflicts.push(conflict.clone());
                }
                if let StepResult::Fail(msg) = &after.result {
                    eprintln!("FAIL {journey_id} effect_read after — {msg}");
                } else {
                    eprintln!(
                        "journey_http: effect_read after GET {} changed",
                        before.probe.route.path
                    );
                }
                totals.record(&after.result);
                step_results.push(after.result);
            }
        }

        if mutating_smoke {
            if journey_id == JourneyId::SwitchLocalBlueosVersion {
                let result = if let Some(dut) = dut_version.as_ref() {
                    let restore_json = dut_version_current_json(dut);
                    eprintln!(
                        "journey_http: restoring core image to tag `{}` (digest {})…",
                        dut.tag,
                        dut.digest.as_deref().unwrap_or("(none)")
                    );
                    run_core_image_switch(&catalog, base, &restore_json, &dut.tag, allow_mutating)
                } else {
                    StepResult::Fail(
                        "core restore: no GET /version/current snapshot from before switch".into(),
                    )
                };
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
        journey_lines.push(format!(
            "{outcome:?} {journey_id}: {} step(s)",
            step_results.len().max(1)
        ));
        if emit_report {
            let mut entry = JourneyReportEntry::from_run(
                journey_id,
                &journey.availability,
                outcome,
                &step_results,
            );
            entry.conflicts = journey_conflicts;
            report_journeys.push(entry);
        }
    }

    drop(mcm_stream_restore);

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

fn journey_step_count(journey: &UserJourney, smoke: bool, mutating_smoke: bool) -> usize {
    if smoke {
        http_smoke_steps(journey).len().max(1)
    } else if mutating_smoke {
        http_mutating_smoke_steps(journey).len().max(1)
    } else {
        http_steps(journey).len().max(1)
    }
}

fn ghost_presence_eligible(
    smoke: bool,
    profile: Option<&DutProfile>,
    journey: &UserJourney,
) -> bool {
    smoke
        && profile.is_some_and(|p| !p.never_strand_mgmt)
        && matches!(
            journey.blast_radius,
            Grounded::Known {
                value: BlastRadius::Safe,
                ..
            }
        )
}

fn run_ghost_presence_probes(
    catalog: &Catalog,
    base: &str,
    availability_reason: &str,
    steps: &[RunnableStep],
) -> (JourneyResult, Vec<StepResult>, Vec<ReportConflict>) {
    let mut step_results = Vec::new();
    let mut conflicts = Vec::new();

    for step in steps {
        let Some(path) = resolve_http_path(catalog, &step.route) else {
            step_results.push(StepResult::Skip(
                "unresolved or templated route path".into(),
            ));
            continue;
        };
        let url = join_url(base, &path);
        let response = execute_curl(&HttpMethod::Get, &url, false, None, None);
        match response {
            Ok((status_code, _body)) if (200..300).contains(&status_code) => {
                let context = format!(
                    "presence: HTTP {status_code} on GET {} while {availability_reason}",
                    step.route.path
                );
                conflicts.push(ReportConflict {
                    kind: ConflictKind::CatalogWrongStatus,
                    context,
                });
                step_results.push(StepResult::Ignored);
            }
            Ok((status_code, _body)) => {
                step_results.push(StepResult::Pass);
                eprintln!(
                    "journey_http: ghost {:?} {} HTTP {status_code} (absent as expected)",
                    step.route.method, step.route.path
                );
            }
            Err(err) => {
                step_results.push(StepResult::Pass);
                eprintln!(
                    "journey_http: ghost {:?} {} — {err} (absent as expected)",
                    step.route.method, step.route.path
                );
            }
        }
    }

    let outcome = summarize_journey(&step_results);
    (outcome, step_results, conflicts)
}

fn skip_reasons(journey: &UserJourney, fixtures: &FixtureInventory) -> Vec<String> {
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
        "usage: journey_http --base <url> [--fixtures internet,pirate,advanced] [--smoke | --mutating-smoke | --negative | --ui | --wifi-endpoints | --frontend-cache] [--wifi-modes open,wpa,wpa2,transition,wpa3] [--dry-run] [--allow-mutating] [--journey <id>] [--report <path.json>]"
    );
}

fn run_ui_suite(
    base: Option<&str>,
    dry_run: bool,
    journey_filter: Option<JourneyId>,
    report_path: Option<&str>,
    fixtures_label: &str,
    fixtures: &FixtureInventory,
) {
    let mut plans: Vec<UiJourneyPlan> = ui_suite_plans()
        .into_iter()
        .filter(|plan| journey_filter.is_none_or(|filter| plan.journey_id == filter.as_str()))
        .collect();
    if let Some(filter) = journey_filter {
        if plans.is_empty() {
            match ui_plan(filter) {
                Some(plan) => plans.push(plan),
                None => {
                    eprintln!("journey_http: no UI plan for journey {filter}");
                    process::exit(2);
                }
            }
        }
    }
    plans.insert(0, wizard_skip_plan());

    let needs_mcm = plans.iter().any(|plan| {
        UI_CAMERA_JOURNEYS
            .iter()
            .any(|id| plan.journey_id == id.as_str())
    });

    if dry_run {
        println!("{}", serde_json::to_string_pretty(&plans).unwrap());
        return;
    }

    let base = base.expect("--ui requires --base");
    if base.contains("192.168.2.2") {
        eprintln!("journey_http: --ui refuses 192.168.2.2 (physical USB vehicle)");
        process::exit(2);
    }

    let needs_sitl = plans.iter().any(|p| p.sitl_frame.is_some());
    if needs_sitl && !base.contains("192.168.0.177") {
        eprintln!("journey_http: SITL --ui only on 192.168.0.177");
        process::exit(2);
    }

    let catalog = Catalog::bootstrap();
    let journeys: std::collections::HashMap<_, _> = catalog
        .journeys()
        .iter()
        .map(|journey| (journey.id, journey))
        .collect();
    let dut_version = fetch_dut_version(base).ok();
    let started_at = utc_rfc3339_now();
    let mut totals = RunCounts::default();
    let mut report_journeys: Vec<JourneyReportEntry> = Vec::new();
    let mut any_fail = false;

    println!(
        "journey_http: ui — {} plan(s) (fixtures={fixtures_label})",
        plans.len()
    );
    println!("base: {base}");
    if let Some(dut) = &dut_version {
        let digest = dut.digest.as_deref().unwrap_or("(none)");
        println!("dut: tag={} digest={digest}", dut.tag);
    }
    println!();

    let mut fixture_skipped: Vec<(JourneyId, String)> = Vec::new();
    plans.retain(|plan| {
        if plan.journey_id == "wizard_skip" {
            return true;
        }
        let Some(journey_id) = JourneyId::ALL
            .iter()
            .copied()
            .find(|id| id.as_str() == plan.journey_id)
        else {
            return true;
        };
        let Some(journey) = journeys.get(&journey_id) else {
            return true;
        };
        if let Some(reason) = ui_fixture_skip_reason(journey, fixtures) {
            fixture_skipped.push((journey_id, reason));
            false
        } else {
            true
        }
    });
    for (journey_id, reason) in fixture_skipped {
        println!("SKIP {journey_id}: {reason}");
        totals.skipped += 1;
        report_journeys.push(JourneyReportEntry::skipped(
            journey_id,
            journeys
                .get(&journey_id)
                .map(|journey| &journey.availability)
                .expect("skipped journey in catalog"),
            reason,
            1,
        ));
    }

    let restore = if needs_sitl {
        match BoardRestore::snapshot(base) {
            Ok(restore) => Some(restore),
            Err(err) => {
                eprintln!("journey_http: snapshot board: {err}");
                process::exit(1);
            }
        }
    } else {
        None
    };

    let sitl_json = if needs_sitl {
        match sitl_cal::sitl_board_json(base) {
            Ok(json) => Some(json),
            Err(err) => {
                eprintln!("journey_http: SITL board: {err}");
                drop(restore);
                process::exit(1);
            }
        }
    } else {
        None
    };

    let mcm_restore = if needs_mcm {
        match McmStreamRestore::snapshot(base) {
            Ok(restore) => Some(restore),
            Err(err) => {
                eprintln!("journey_http: snapshot MCM streams: {err}");
                drop(restore);
                process::exit(1);
            }
        }
    } else {
        None
    };

    // Abort-wizard equivalent so Vehicle Setup is not under the first-boot dialog.
    match execute_curl(
        &HttpMethod::Post,
        &join_url(base, "/bag/v1.0/set/wizard"),
        true,
        Some(r#"{"version":4}"#),
        None,
    ) {
        Ok((200, _)) => {}
        Ok((status, body)) => eprintln!("journey_http: persist wizard skip HTTP {status}: {body}"),
        Err(err) => eprintln!("journey_http: persist wizard skip: {err}"),
    }

    let mut groups: Vec<(Option<&'static str>, Vec<UiJourneyPlan>)> = Vec::new();
    for plan in plans {
        match groups.last_mut() {
            Some((frame, bucket)) if *frame == plan.sitl_frame => bucket.push(plan),
            _ => groups.push((plan.sitl_frame, vec![plan])),
        }
    }

    for (frame, group) in groups {
        if let Some(frame) = frame {
            println!("journey_http: SITL frame {frame}");
            if let Err(err) =
                sitl_cal::set_board(base, sitl_json.as_deref().expect("needs_sitl"), Some(frame))
            {
                eprintln!("FAIL sitl_frame {frame} — {err}");
                any_fail = true;
                totals.failed += group.len();
                continue;
            }
            if let Err(err) = sitl_cal::wait_heartbeat(base, Duration::from_secs(90)) {
                eprintln!("FAIL sitl heartbeat after {frame} — {err}");
                any_fail = true;
                totals.failed += group.len();
                continue;
            }
            if frame == sitl_cal::SITL_FRAME_CALIBRATION {
                if let Err(err) = sitl_cal::release_calibration_servos(base) {
                    eprintln!("FAIL sitl calibration servos — {err}");
                    any_fail = true;
                    totals.failed += group.len();
                    continue;
                }
                if let Err(err) = sitl_cal::wait_level_attitude(base, Duration::from_secs(30)) {
                    eprintln!("FAIL sitl level attitude — {err}");
                    any_fail = true;
                    totals.failed += group.len();
                    continue;
                }
            }
            let _ = sitl_cal::post_rc_override(base, sitl_cal::SitlRc::stop());
        }

        match run_playwright_ui(base, &group, fixtures_label) {
            Ok(results) => {
                for (id, pass, detail) in results {
                    if pass {
                        println!("PASS {id}");
                        totals.passed += 1;
                    } else {
                        println!("FAIL {id} — {detail}");
                        totals.failed += 1;
                        any_fail = true;
                    }
                    emit_ui_report_entry(&id, pass, &detail, &journeys, &mut report_journeys);
                }
            }
            Err(err) => {
                eprintln!("FAIL playwright — {err}");
                any_fail = true;
                totals.failed += group.len();
            }
        }
    }

    drop(restore);
    drop(mcm_restore);

    println!();
    println!(
        "summary: passed={} failed={} skipped={} unasserted={}",
        totals.passed, totals.failed, totals.skipped, totals.unasserted
    );

    if let Some(path) = report_path {
        let report = JourneyHttpReport {
            schema_version: SCHEMA_VERSION,
            suite: SuiteKind::Ui,
            base: base.to_string(),
            dut: dut_version.as_ref().map(ReportDut::from_dut),
            started_at,
            finished_at: utc_rfc3339_now(),
            counts: totals.into(),
            journeys: report_journeys,
        };
        if let Err(err) = write_journey_http_report(path, &report) {
            eprintln!("journey_http: report: {err}");
            process::exit(2);
        }
    }

    if any_fail || totals.failed > 0 {
        process::exit(1);
    }
}

fn emit_ui_report_entry(
    id: &str,
    pass: bool,
    detail: &str,
    journeys: &std::collections::HashMap<JourneyId, &blueos_catalog::UserJourney>,
    report_journeys: &mut Vec<JourneyReportEntry>,
) -> bool {
    let Some(journey_id) = JourneyId::ALL.iter().copied().find(|j| j.as_str() == id) else {
        return false;
    };
    let Some(journey) = journeys.get(&journey_id) else {
        return false;
    };
    let result = if pass {
        StepResult::Pass
    } else {
        StepResult::Fail(detail.to_string())
    };
    report_journeys.push(JourneyReportEntry::from_run(
        journey_id,
        &journey.availability,
        summarize_journey(std::slice::from_ref(&result)),
        &[result],
    ));
    true
}

fn run_playwright_ui(
    base: &str,
    plans: &[UiJourneyPlan],
    fixtures_label: &str,
) -> Result<Vec<(String, bool, String)>, String> {
    let e2e = Path::new(env!("CARGO_MANIFEST_DIR")).join("e2e");
    let plan_path = std::env::temp_dir().join("blueos-catalog-ui-plan.json");
    let json = serde_json::to_string_pretty(plans).map_err(|err| err.to_string())?;
    std::fs::write(&plan_path, json).map_err(|err| format!("write UI_PLAN: {err}"))?;

    let output = process::Command::new("npx")
        .current_dir(&e2e)
        .env("BLUEOS_BASE", base)
        .env("BLUEOS_FIXTURES", fixtures_label)
        .env("UI_PLAN", &plan_path)
        .args([
            "playwright",
            "test",
            "tests/journey_ui.spec.ts",
            "--reporter=list",
        ])
        .output()
        .map_err(|err| format!("npx playwright: {err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let _ = std::io::stderr().write_all(combined.as_bytes());

    let mut results = Vec::new();
    for line in combined.lines() {
        let Some(rest) = line.strip_prefix("UI_RESULT ") else {
            continue;
        };
        let mut parts = rest.splitn(3, ' ');
        let id = parts.next().unwrap_or("").to_string();
        let status = parts.next().unwrap_or("");
        let detail = parts.next().unwrap_or("").to_string();
        results.push((id, status == "PASS", detail));
    }
    if results.is_empty() && !output.status.success() {
        return Err(format!(
            "playwright exited {} with no UI_RESULT lines",
            output.status
        ));
    }
    Ok(results)
}

fn run_negative_probes(
    base: Option<&str>,
    dry_run: bool,
    journey_filter: Option<JourneyId>,
    report_path: Option<&str>,
) {
    let catalog = Catalog::bootstrap();
    let journeys: std::collections::HashMap<_, _> = catalog
        .journeys()
        .iter()
        .map(|journey| (journey.id, journey))
        .collect();

    let mut probes: Vec<&NegativeProbe> = NEGATIVE_PROBES
        .iter()
        .filter(|probe| journey_filter.is_none_or(|filter| probe.journey_id == filter))
        .collect();

    if let Some(filter) = journey_filter {
        if probes.is_empty() {
            eprintln!("journey_http: no negative probes for journey {filter}");
            process::exit(2);
        }
    }

    // NP-62 guard probe first; NP-90 ordering-sensitive within pardal group.
    probes.sort_by_key(|probe| match probe.id {
        "NP-62" => (0, probe.id),
        "NP-90" => (1, probe.id),
        _ => (2, probe.id),
    });

    let mut totals = RunCounts::default();
    let mut any_fail = false;
    let emit_report = report_path.is_some() && !dry_run;
    let started_at = if emit_report {
        Some(utc_rfc3339_now())
    } else {
        None
    };

    println!(
        "journey_http: negative — {} probes (dry_run={dry_run})",
        probes.len()
    );
    if let Some(base) = base {
        println!("base: {base}");
    }
    println!();

    let dut_version = if dry_run {
        None
    } else if let Some(base_url) = base {
        fetch_dut_version(base_url).ok()
    } else {
        None
    };

    let mut current_tag: Option<String> = dut_version.as_ref().map(|dut| dut.tag.clone());
    let mut report_entries: Vec<JourneyReportEntry> = Vec::new();

    for probe in probes {
        if dry_run {
            let url = if let Some(base) = base {
                negative_probe_url(base, probe)
            } else {
                negative_probe_url("http://dry-run", probe)
            };
            println!("{}", format_negative_dry_run(probe, &url));
            totals.skipped += 1;
            continue;
        }

        let base = base.expect("base checked above");
        if probe.id == "NP-62" && current_tag.is_none() {
            match fetch_dut_version(base) {
                Ok(dut) => current_tag = Some(dut.tag),
                Err(err) => {
                    let result = StepResult::Fail(format!("fetch running tag for NP-62: {err}"));
                    eprintln!("FAIL {} — {err}", probe.id);
                    totals.record(&result);
                    any_fail = true;
                    if emit_report {
                        if let Some(journey) = journeys.get(&probe.journey_id) {
                            report_entries.push(JourneyReportEntry::from_negative_probe(
                                probe.id,
                                probe.journey_id,
                                &journey.availability,
                                &result,
                            ));
                        }
                    }
                    continue;
                }
            }
        }

        let result = run_negative_probe(base, probe, true, current_tag.as_deref());
        totals.record(&result);
        if matches!(result, StepResult::Fail(_)) {
            any_fail = true;
        }
        if emit_report {
            if let Some(journey) = journeys.get(&probe.journey_id) {
                report_entries.push(JourneyReportEntry::from_negative_probe(
                    probe.id,
                    probe.journey_id,
                    &journey.availability,
                    &result,
                ));
            }
        }
    }

    println!();
    println!(
        "summary: passed={} failed={} skipped={} unasserted={}",
        totals.passed, totals.failed, totals.skipped, totals.unasserted
    );

    if let Some(path) = report_path {
        if !dry_run {
            let report = JourneyHttpReport {
                schema_version: SCHEMA_VERSION,
                suite: SuiteKind::Negative,
                base: base.unwrap_or("").to_string(),
                dut: dut_version.as_ref().map(ReportDut::from_dut),
                started_at: started_at.unwrap_or_else(utc_rfc3339_now),
                finished_at: utc_rfc3339_now(),
                counts: totals.into(),
                journeys: report_entries,
            };
            if let Err(err) = write_journey_http_report(path, &report) {
                eprintln!("journey_http: report: {err}");
                process::exit(2);
            }
        }
    }

    if any_fail || totals.failed > 0 {
        process::exit(1);
    }
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

fn finalize_http_step_run(
    journey_conflicts: &mut Vec<ReportConflict>,
    run: blueos_catalog::runner::HttpStepRun,
) -> StepResult {
    if let Some(conflict) = run.conflict {
        journey_conflicts.push(conflict);
    }
    run.result
}

#[cfg(test)]
mod tests {
    use super::*;
    use blueos_catalog::journey::Visibility;
    use blueos_catalog::provenance::{Grounded, GroundedSet, Provenance};
    use blueos_catalog::runner::product_missing_reject_conflict;
    use blueos_catalog::version::FeatureAvailability;
    use blueos_catalog::{BodyKind, RouteRef, ServiceId};

    const DOC: Provenance = Provenance::doc("journey_http_test", 1);

    fn test_journey(blast_radius: BlastRadius) -> UserJourney {
        UserJourney {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("ghost test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(&[]),
            availability: FeatureAvailability::unknown(),
            blast_radius: Grounded::known(blast_radius, DOC),
            chains_from: None,
        }
    }

    #[test]
    fn ghost_presence_eligible_requires_smoke_safe_profile_and_not_2_2() {
        let safe = test_journey(BlastRadius::Safe);
        let play = dut_profile_for_host("http://192.168.0.177");
        let vehicle = dut_profile_for_host("http://192.168.2.2");

        assert!(ghost_presence_eligible(true, Some(&play), &safe));
        assert!(!ghost_presence_eligible(false, Some(&play), &safe));
        assert!(!ghost_presence_eligible(true, Some(&vehicle), &safe));
        assert!(!ghost_presence_eligible(true, None, &safe));

        let reversible = test_journey(BlastRadius::Reversible);
        assert!(!ghost_presence_eligible(true, Some(&play), &reversible));
        let destructive = test_journey(BlastRadius::Destructive);
        assert!(!ghost_presence_eligible(true, Some(&play), &destructive));
    }

    #[test]
    fn ghost_presence_probes_use_get_only_and_never_fail_wave() {
        let catalog = Catalog::bootstrap();
        let journey = catalog
            .journey_by_id(&JourneyId::ConnectToWifiNetwork)
            .expect("wifi journey");
        let steps = http_smoke_steps(journey);
        assert!(!steps.is_empty());
        assert!(steps
            .iter()
            .all(|step| matches!(step.route.method, HttpMethod::Get)));

        let (outcome, results, _) = run_ghost_presence_probes(
            &catalog,
            "http://127.0.0.1:1",
            "not present on test tag",
            &steps,
        );
        assert_ne!(outcome, JourneyResult::Fail);
        assert!(!results.iter().any(|r| matches!(r, StepResult::Fail(_))));
    }

    #[test]
    fn ghost_presence_probes_skip_unresolved_routes() {
        let catalog = Catalog::bootstrap();
        let steps = vec![RunnableStep {
            journey_id: JourneyId::ConnectToWifiNetwork,
            step_index: 0,
            route: RouteRef {
                service: ServiceId::Wifi,
                method: HttpMethod::Post,
                path: "/{iface}/missing",
                version: Some("v1.0"),
            },
            expected_status: Some(200),
            body_predicate: None,
            body_kind: blueos_catalog::BodyKind::Unknown,
            body: None,
            query: None,
            form_file: None,
        }];

        let (_, results, _) = run_ghost_presence_probes(
            &catalog,
            "http://127.0.0.1:1",
            "not present on test tag",
            &steps,
        );
        assert!(matches!(results[0], StepResult::Skip(_)));
    }

    #[test]
    fn wave_path_attaches_product_missing_reject_conflict() {
        let step = RunnableStep {
            journey_id: JourneyId::InspectDiskUsage,
            step_index: 1,
            route: RouteRef {
                service: ServiceId::DiskUsage,
                method: HttpMethod::Get,
                path: "/disk/usage",
                version: Some("v1.0"),
            },
            expected_status: Some(200),
            body_predicate: Some("\"detail\""),
            body_kind: BodyKind::ErrorEnvelope,
            body: None,
            query: None,
            form_file: None,
        };
        let run = blueos_catalog::runner::HttpStepRun {
            result: StepResult::Pass,
            conflict: product_missing_reject_conflict(&step, 200),
        };
        let mut journey_conflicts = Vec::new();
        let result = finalize_http_step_run(&mut journey_conflicts, run);
        assert!(matches!(result, StepResult::Pass));
        assert_eq!(journey_conflicts.len(), 1);
        assert_eq!(
            journey_conflicts[0].kind,
            ConflictKind::ProductMissingReject
        );
    }
}
