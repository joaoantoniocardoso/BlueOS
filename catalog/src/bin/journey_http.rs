use std::process;

use blueos_catalog::{
    evaluate_journey, http_journeys, http_steps, journey_fixtures_ready, parse_fixture_list,
    run_http_step, summarize_journey, Catalog, FixtureInventory, JourneyId, JourneyResult,
    PreconditionStatus, RunCounts, StepResult,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut base: Option<String> = None;
    let mut fixtures_spec: Option<String> = None;
    let mut dry_run = false;
    let mut allow_mutating = false;
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

    if !dry_run && base.is_none() {
        usage_and_exit("--base is required unless --dry-run is set");
    }

    let fixtures = match fixtures_spec {
        Some(spec) => match parse_fixture_list(&spec) {
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

    println!(
        "journey_http: {} Http journeys (dry_run={dry_run}, allow_mutating={allow_mutating})",
        journeys.len()
    );
    if let Some(base) = &base {
        println!("base: {base}");
    }
    println!();

    for journey in journeys {
        let journey_id = journey.id;
        if !journey_fixtures_ready(journey, &fixtures) {
            let reasons = skip_reasons(journey, &fixtures);
            totals.skipped += http_steps(journey).len().max(1);
            journey_lines.push(format!("SKIP {journey_id}: {}", reasons.join("; ")));
            continue;
        }

        let steps = http_steps(journey);
        if steps.is_empty() {
            journey_lines.push(format!("SKIP {journey_id}: no runnable HTTP steps"));
            totals.skipped += 1;
            continue;
        }

        if dry_run {
            for step in &steps {
                println!(
                    "DRY-RUN {} step {} {:?} {}",
                    journey_id, step.step_index, step.route.method, step.route.path
                );
                totals.skipped += 1;
            }
            journey_lines.push(format!("DRY-RUN {journey_id}: {} step(s)", steps.len()));
            continue;
        }

        let base = base.as_deref().expect("base checked above");
        let mut step_results = Vec::new();
        for step in &steps {
            let result = run_http_step(&catalog, base, step, allow_mutating);
            if let StepResult::Fail(msg) = &result {
                eprintln!(
                    "FAIL {} step {} {:?} {} — {msg}",
                    journey_id, step.step_index, step.route.method, step.route.path
                );
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
        "usage: journey_http --base <url> [--fixtures internet,pirate] [--dry-run] [--allow-mutating] [--journey <id>]"
    );
}
