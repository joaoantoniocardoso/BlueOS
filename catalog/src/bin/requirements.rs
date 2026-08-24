use std::env;
use std::process::ExitCode;

use blueos_catalog::catalog::Catalog;
use blueos_catalog::requirement::RequirementCatalog;
use blueos_catalog::requirements_report::{
    diff_requirement_reports, load_requirements_baseline, render_diff_report, render_rtm_csv,
    render_srs, requirements_json, validate_rtm_completeness, write_requirements_baseline,
};

const DEFAULT_VERSION: &str = "1.4-dev";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }

    let version = flag_value(&args, "--version").unwrap_or_else(|| DEFAULT_VERSION.to_string());
    let diff_tag = flag_value(&args, "--diff");
    let snapshot = args.iter().any(|arg| arg == "--snapshot");
    let json_output = args.iter().any(|arg| arg == "--json");
    let srs_output = args.iter().any(|arg| arg == "--srs");
    let rtm_output = args.iter().any(|arg| arg == "--rtm");

    if !json_output && !srs_output && !rtm_output && diff_tag.is_none() && !snapshot {
        eprintln!("error: specify at least one of --json, --srs, --rtm, --diff, --snapshot");
        print_help();
        return ExitCode::from(2);
    }

    let catalog = Catalog::bootstrap();
    let requirements = RequirementCatalog::from_catalog(&catalog);

    if snapshot {
        let tag = diff_tag.as_deref().unwrap_or(&version);
        return match write_requirements_baseline(tag, &requirements) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("error: {err}");
                ExitCode::from(1)
            }
        };
    }

    if let Some(ref diff_tag) = diff_tag {
        let baseline = match load_requirements_baseline(diff_tag) {
            Ok(baseline) => baseline,
            Err(err) => {
                eprintln!("error: {err}");
                return ExitCode::from(1);
            }
        };
        let current = requirements_json(&requirements, &version);
        let diff = diff_requirement_reports(&baseline, &current);
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&diff).expect("serialize diff")
            );
        } else {
            print!("{}", render_diff_report(&diff));
        }
    }

    if json_output && diff_tag.is_none() {
        println!(
            "{}",
            serde_json::to_string_pretty(&requirements_json(&requirements, &version))
                .expect("serialize requirements json")
        );
    }

    if srs_output {
        print!("{}", render_srs(&requirements, &version));
    }

    if rtm_output {
        let filtered = requirements.filter_by_version(&version);
        if let Err(errors) = validate_rtm_completeness(&catalog, &filtered) {
            for error in errors {
                eprintln!("error: {error}");
            }
            return ExitCode::from(1);
        }
        print!("{}", render_rtm_csv(&catalog, &filtered));
    }

    ExitCode::SUCCESS
}

fn flag_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find_map(|pair| {
        if pair[0] == name {
            Some(pair[1].clone())
        } else {
            None
        }
    })
}

fn print_help() {
    eprintln!(
        "usage:\n\
         \x20 requirements --version <tag> [--json] [--srs] [--rtm] [--diff <tag>] [--snapshot]\n\
         \n\
         --version  filter requirements to a DUT tag baseline (default {DEFAULT_VERSION})\n\
         --json     emit filtered requirements as JSON\n\
         --srs      render a software requirements document (markdown)\n\
         --rtm      emit a CSV traceability matrix\n\
         --diff     compare current derivation to committed requirements-baselines/<tag>.json\n\
         --snapshot write requirements-baselines/<tag>.json (tag from --diff or --version)"
    );
}
