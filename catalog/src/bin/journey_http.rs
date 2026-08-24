// Live HTTP journey runner: `journey_http --base http://<pi> [--fixtures internet,pirate,advanced]`.
// Tier-1 smoke (GET + known status only): `journey_http --base http://<pi> --smoke`.
// Tier-2 mutating smoke (allowlisted reversible journeys): `journey_http --base http://<pi> --mutating-smoke`.
// Frontend PWA/cache contract: `journey_http --base http://<pi> --frontend-cache`.
// Extension install/upgrade/downgrade/uninstall: `journey_http --base http://<pi> --extension-lifecycle --allow-mutating`.
// Offline plan: `journey_http --dry-run` (no --base).
use std::process;

use catalog_harness::journey_http::{print_help, run, JourneyHttpCli};
use catalog_kernel::id::journey::JourneyId;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return;
    }

    let mut cli = JourneyHttpCli {
        base: None,
        fixtures_spec: None,
        dry_run: false,
        allow_mutating: false,
        smoke: false,
        mutating_smoke: false,
        negative: false,
        ui: false,
        journey_filter: None,
        report_path: None,
        wifi_modes_spec: None,
        wifi_endpoints: false,
        frontend_cache: false,
        extension_lifecycle: false,
    };

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--base" => {
                index += 1;
                cli.base = Some(
                    args.get(index)
                        .cloned()
                        .unwrap_or_else(|| usage_and_exit("--base requires a URL")),
                );
            }
            "--fixtures" => {
                index += 1;
                cli.fixtures_spec = Some(
                    args.get(index)
                        .cloned()
                        .unwrap_or_else(|| usage_and_exit("--fixtures requires a spec")),
                );
            }
            "--dry-run" => cli.dry_run = true,
            "--allow-mutating" => cli.allow_mutating = true,
            "--smoke" => cli.smoke = true,
            "--mutating-smoke" => cli.mutating_smoke = true,
            "--negative" => cli.negative = true,
            "--ui" => cli.ui = true,
            "--wifi-modes" => {
                index += 1;
                cli.wifi_modes_spec = Some(
                    args.get(index)
                        .cloned()
                        .unwrap_or_else(|| usage_and_exit("--wifi-modes requires a csv")),
                );
            }
            "--wifi-endpoints" => cli.wifi_endpoints = true,
            "--frontend-cache" => cli.frontend_cache = true,
            "--extension-lifecycle" => cli.extension_lifecycle = true,
            "--journey" => {
                index += 1;
                let id = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| usage_and_exit("--journey requires an id"));
                cli.journey_filter = Some(parse_journey_id(&id));
            }
            "--report" => {
                index += 1;
                cli.report_path = Some(
                    args.get(index)
                        .cloned()
                        .unwrap_or_else(|| usage_and_exit("--report requires a path")),
                );
            }
            other => usage_and_exit(&format!("unknown argument: {other}")),
        }
        index += 1;
    }

    run(cli);
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
