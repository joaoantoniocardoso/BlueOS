use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use blueos_catalog::catalog::Catalog;
use blueos_catalog::cli::flag_value;
use catalog_analysis::api_contract::{
    api_coverage_report, diff_snapshots, load_api_contract_baseline, write_api_contract_baseline,
    ApiContractSnapshot, DEFAULT_API_CONTRACT_BASELINE,
};
use catalog_extract::extract_fastapi::extract_fastapi_from_repo;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }

    let snapshot = args.iter().any(|arg| arg == "--snapshot");
    let json_output = args.iter().any(|arg| arg == "--json");
    let coverage = args.iter().any(|arg| arg == "--coverage");
    let check = args.iter().any(|arg| arg == "--check" || arg == "--diff");
    let baseline_path = flag_value(&args, "--baseline")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_API_CONTRACT_BASELINE)
        });

    let repo_root = flag_value(&args, "--repo-root")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));
    let routes = match extract_fastapi_from_repo(&repo_root) {
        Ok(routes) => routes,
        Err(error) => {
            eprintln!("api_contracts extract failed: {error:?}");
            return ExitCode::FAILURE;
        }
    };
    let current = ApiContractSnapshot::from_extracted(&routes);
    if let Some(key) = current.duplicate_keys().first() {
        let version = key.version.as_deref().unwrap_or("-");
        eprintln!(
            "api_contracts: duplicate key {} {} {} {version}",
            key.service, key.method, key.path
        );
        return ExitCode::FAILURE;
    }

    if snapshot {
        return match write_api_contract_baseline(&baseline_path, &current) {
            Ok(()) => {
                println!(
                    "wrote {} routes to {}",
                    current.routes.len(),
                    baseline_path.display()
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("api_contracts snapshot failed: {error}");
                ExitCode::FAILURE
            }
        };
    }

    if coverage {
        let catalog = Catalog::bootstrap();
        let report = api_coverage_report(&catalog, &routes);
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("serialize coverage")
            );
        } else {
            println!(
                "api coverage: total={} mapped={} unmapped={}",
                report.total, report.mapped, report.unmapped
            );
            for (source, count) in &report.by_source {
                println!("  {source}: {count}");
            }
            if !report.unmapped_routes.is_empty() {
                println!("unmapped:");
                for key in &report.unmapped_routes {
                    let version = key.version.as_deref().unwrap_or("-");
                    println!("  {} {} {} {version}", key.service, key.method, key.path);
                }
            }
            if !report.orphan_hits.is_empty() {
                println!("orphan hits (catalog cites a route the inventory does not have):");
                for key in &report.orphan_hits {
                    let version = key.version.as_deref().unwrap_or("-");
                    println!("  {} {} {} {version}", key.service, key.method, key.path);
                }
            }
        }
        return ExitCode::SUCCESS;
    }

    if json_output && !check {
        println!(
            "{}",
            serde_json::to_string_pretty(&current).expect("serialize snapshot")
        );
        return ExitCode::SUCCESS;
    }

    if !check {
        eprintln!("error: specify --check, --coverage, --json, or --snapshot");
        print_help();
        return ExitCode::from(2);
    }

    let baseline = match load_api_contract_baseline(&baseline_path) {
        Ok(baseline) => baseline,
        Err(error) => {
            eprintln!("api_contracts compare failed: {error}");
            eprintln!("run with --snapshot to create {}", baseline_path.display());
            return ExitCode::FAILURE;
        }
    };
    let diff = diff_snapshots(&baseline, &current);
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&diff).expect("serialize diff")
        );
    } else {
        println!(
            "api contracts: added={} removed={} changed={}",
            diff.added.len(),
            diff.removed.len(),
            diff.changed.len()
        );
        for key in &diff.added {
            let version = key.version.as_deref().unwrap_or("-");
            println!(
                "  added {} {} {} {version}",
                key.service, key.method, key.path
            );
        }
        for key in &diff.removed {
            let version = key.version.as_deref().unwrap_or("-");
            println!(
                "  removed {} {} {} {version}",
                key.service, key.method, key.path
            );
        }
        for change in &diff.changed {
            println!(
                "  changed {} {} {} {}: {} -> {}",
                change.key.service,
                change.key.method,
                change.key.path,
                change.field,
                change.before,
                change.after
            );
        }
    }
    if diff.is_clean() {
        if !json_output {
            println!("api contracts: PASS (matches baseline)");
        }
        ExitCode::SUCCESS
    } else if diff.is_breaking() {
        eprintln!("api contracts: FAIL (removed or changed vs baseline)");
        ExitCode::FAILURE
    } else {
        eprintln!("api contracts: FAIL (added vs baseline; --snapshot to accept)");
        ExitCode::FAILURE
    }
}

fn print_help() {
    eprintln!(
        "usage:\n\
         \x20 api_contracts [--check|--diff] [--coverage] [--json] [--snapshot] [--baseline <path>]\n\
         \x20               [--repo-root <path>]\n\
         \n\
         --repo-root     extract from another checkout, to compare a device's image against HEAD\n\
         \n\
         --check/--diff  fail on any baseline drift (added, removed, or status/response_model)\n\
         --coverage      report inventory routes with no journey/page/SLO/probe, plus orphan hits\n\
         --json          emit snapshot, coverage, or diff as JSON\n\
         --snapshot      write the extracted inventory to the baseline file"
    );
}
