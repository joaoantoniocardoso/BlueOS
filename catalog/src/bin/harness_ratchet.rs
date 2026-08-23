use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use blueos_catalog::harness_ratchet::{
    compare_harness_ratchet, harness_ratchet_counts, load_harness_ratchet_baseline,
    write_harness_ratchet_baseline, HarnessRatchetCounts, DEFAULT_BASELINE_PATH,
};
use blueos_catalog::Catalog;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let snapshot = args.iter().any(|arg| arg == "--snapshot");
    let baseline_path = args
        .windows(2)
        .find_map(|window| {
            if window[0] == "--baseline" {
                Some(PathBuf::from(&window[1]))
            } else {
                None
            }
        })
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_BASELINE_PATH));

    let catalog = Catalog::bootstrap();
    let current = harness_ratchet_counts(&catalog);

    if snapshot {
        match write_harness_ratchet_baseline(&baseline_path, current) {
            Ok(()) => {
                print_counts("snapshot", current);
                println!("wrote {}", baseline_path.display());
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("harness_ratchet snapshot failed: {err}");
                ExitCode::FAILURE
            }
        }
    } else {
        let baseline = match load_harness_ratchet_baseline(&baseline_path) {
            Ok(counts) => counts,
            Err(err) => {
                eprintln!("harness_ratchet compare failed: {err}");
                eprintln!("run with --snapshot to create {}", baseline_path.display());
                return ExitCode::FAILURE;
            }
        };
        print_counts("baseline", baseline);
        print_counts("current", current);
        let regressions = compare_harness_ratchet(baseline, current);
        if regressions.is_empty() {
            println!("harness ratchet: PASS (no worsening vs baseline)");
            ExitCode::SUCCESS
        } else {
            eprintln!("harness ratchet: FAIL (worsened vs baseline)");
            for regression in regressions {
                eprintln!(
                    "  {}: baseline={} current={}",
                    regression.field, regression.baseline, regression.current
                );
            }
            ExitCode::FAILURE
        }
    }
}

fn print_counts(label: &str, counts: HarnessRatchetCounts) {
    let fields: Vec<String> = counts
        .named_fields()
        .into_iter()
        .map(|(field, value)| format!("{field}={value}"))
        .collect();
    println!("{label}: {}", fields.join(" "));
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use blueos_catalog::harness_ratchet::{
        compare_harness_ratchet, harness_ratchet_counts, HarnessRatchetCounts,
        DEFAULT_BASELINE_PATH,
    };
    use blueos_catalog::Catalog;

    #[test]
    fn bootstrap_counts_match_committed_baseline() {
        let catalog = Catalog::bootstrap();
        let current = harness_ratchet_counts(&catalog);
        let baseline_path = PathBuf::from(DEFAULT_BASELINE_PATH);
        let baseline_json = fs::read_to_string(&baseline_path).expect("baseline file");
        let baseline: HarnessRatchetCounts =
            serde_json::from_str(&baseline_json).expect("baseline json");
        assert!(
            compare_harness_ratchet(baseline, current).is_empty(),
            "committed baseline must not be worse than current counts"
        );
    }

    #[test]
    fn worsening_mutating_missing_effect_read_regresses() {
        let catalog = Catalog::bootstrap();
        let current = harness_ratchet_counts(&catalog);
        let worse = HarnessRatchetCounts {
            mutating_smoke_missing_effect_read: current.mutating_smoke_missing_effect_read + 1,
            ..current
        };
        let regressions = compare_harness_ratchet(current, worse);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].field, "mutating_smoke_missing_effect_read");
    }

    #[test]
    fn worsening_orchestrated_missing_ui_plan_regresses() {
        let catalog = Catalog::bootstrap();
        let current = harness_ratchet_counts(&catalog);
        let worse = HarnessRatchetCounts {
            client_orchestrated_missing_ui_plan: current.client_orchestrated_missing_ui_plan + 1,
            ..current
        };
        let regressions = compare_harness_ratchet(current, worse);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].field, "client_orchestrated_missing_ui_plan");
    }
}
