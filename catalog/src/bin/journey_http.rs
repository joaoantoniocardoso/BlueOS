// Live HTTP journey runner: `journey_http --base http://<pi> [--fixtures internet,pirate,advanced]`.
// Tier-1 smoke (GET + known status only): `journey_http --base http://<pi> --smoke`.
// Tier-2 mutating smoke (allowlisted reversible journeys): `journey_http --base http://<pi> --mutating-smoke`.
// Offline plan: `journey_http --dry-run` (no --base).
use std::process;

use blueos_catalog::{
    evaluate_journey, format_dry_run, format_http_fail, http_journeys, http_mutating_smoke_steps,
    http_smoke_steps, http_steps, join_url, journey_fixtures_ready, journey_http_mode_conflict,
    journey_http_requires_base, parse_fixture_list, resolve_http_path, run_http_step,
    summarize_journey, Catalog, FixtureInventory, JourneyId, JourneyResult, PreconditionStatus,
    RunCounts, StepResult, MUTATING_SMOKE_JOURNEY_IDS, SMOKE_DEFAULT_FIXTURES,
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
            "--journey" => {
                index += 1;
                let id = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| usage_and_exit("--journey requires an id"));
                journey_filter = Some(parse_journey_id(&id));
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

    let fixtures_label = if let Some(spec) = &fixtures_spec {
        spec.clone()
    } else if smoke || mutating_smoke {
        SMOKE_DEFAULT_FIXTURES.to_string()
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
        None if smoke || mutating_smoke => match parse_fixture_list(SMOKE_DEFAULT_FIXTURES) {
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
        journeys.retain(|journey| MUTATING_SMOKE_JOURNEY_IDS.contains(&journey.id));
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
    let mut any_fail = false;

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
    println!();

    for journey in journeys {
        let journey_id = journey.id;
        if !journey_fixtures_ready(journey, &fixtures) {
            let reasons = skip_reasons(journey, &fixtures);
            let step_count = if smoke {
                http_smoke_steps(journey).len().max(1)
            } else if mutating_smoke {
                http_mutating_smoke_steps(journey).len().max(1)
            } else {
                http_steps(journey).len().max(1)
            };
            totals.skipped += step_count;
            journey_lines.push(format!("SKIP {journey_id}: {}", reasons.join("; ")));
            continue;
        }

        let steps = if smoke {
            http_smoke_steps(journey)
        } else if mutating_smoke {
            http_mutating_smoke_steps(journey)
        } else {
            http_steps(journey)
        };
        if steps.is_empty() {
            let reason = if smoke {
                "no smoke-eligible GET steps with expected_status"
            } else if mutating_smoke {
                "no mutating-smoke-eligible steps with expected_status"
            } else {
                "no runnable HTTP steps"
            };
            journey_lines.push(format!("SKIP {journey_id}: {reason}"));
            totals.skipped += 1;
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
        for step in &steps {
            let resolved_url = resolve_http_path(&catalog, &step.route)
                .map(|path| join_url(base, &path))
                .unwrap_or_else(|| "(unresolved)".to_string());
            let result = run_http_step(&catalog, base, step, allow_mutating);
            if let StepResult::Fail(msg) = &result {
                eprintln!("{}", format_http_fail(journey_id, step, &resolved_url, msg));
            }
            totals.record(&result);
            step_results.push(result);
        }

        let outcome = summarize_journey(&step_results);
        if outcome == JourneyResult::Fail {
            any_fail = true;
        }
        journey_lines.push(format!("{outcome:?} {journey_id}: {} step(s)", steps.len()));
    }

    println!();
    println!(
        "summary: passed={} failed={} skipped={} unasserted={}",
        totals.passed, totals.failed, totals.skipped, totals.unasserted
    );
    for line in journey_lines {
        println!("  {line}");
    }

    if any_fail {
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
        "usage: journey_http --base <url> [--fixtures internet,pirate,advanced] [--smoke | --mutating-smoke] [--dry-run] [--allow-mutating] [--journey <id>]"
    );
}
