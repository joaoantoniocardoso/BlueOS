use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use catalog_provenance::provenance_anchor::{
    apply_relocated_line_fixes, fit_overlong_citation_lines,
};
use catalog_provenance::provenance_walk::{
    anchor_exempt_citation_count, build_report, file_citation_count, format_unrepaired_fix,
    gate_failed, kind_status_counts, mark_applied_relocations_resolved, relocated_citations,
    source_facing_unresolved, unanchored_citation_count, unrepaired_relocations,
    unresolved_citations, CitationKind, ProvenanceWalkReport, ResolveStatus,
};

const INVENTORY_PATH: &str = "extras/requirements-baseline/P0A_INVENTORY.md";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }
    for arg in args.iter().skip(1) {
        if arg.starts_with('-') && arg != "--json" && arg != "--write-inventory" && arg != "--fix" {
            eprintln!("unknown argument: {arg}");
            print_help();
            return ExitCode::from(2);
        }
    }
    let json_output = args.iter().any(|arg| arg == "--json");
    let write_inventory = args.iter().any(|arg| arg == "--write-inventory");
    let fix = args.iter().any(|arg| arg == "--fix");
    let mut report = build_report();

    let mut unrepaired_fix = false;
    if fix {
        let relocations = relocated_citations(&report);
        if !relocations.is_empty() {
            let applied =
                apply_relocated_line_fixes(&relocations).expect("apply relocated line fixes");
            mark_applied_relocations_resolved(&mut report, &applied);
            let unrepaired = unrepaired_relocations(&relocations, &applied);
            for rel in &unrepaired {
                eprintln!("{}", format_unrepaired_fix(rel));
            }
            unrepaired_fix = !unrepaired.is_empty();
        }
        fit_overlong_citation_lines().expect("fit overlong citation lines");
    }

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("serialize report")
        );
    } else {
        print_text_report(&report);
    }

    if write_inventory {
        write_inventory_file(&report);
    }

    if unrepaired_fix || gate_failed(&report) {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn print_help() {
    eprintln!(
        "usage: provenance_lint [--json] [--write-inventory] [--fix]\n\
         --fix rewrites catalog/src/ to repair relocated citation line numbers; \
         run `cargo build --bin provenance_lint` before any verifying lint after --fix \
         (cargo run may reuse a stale binary)."
    );
}

fn print_text_report(report: &ProvenanceWalkReport) {
    println!("provenance_lint");
    println!("docs_root_present: {}", report.docs_root_present);
    println!("total citations: {}", file_citation_count(report));
    println!(
        "unanchored citations: {}",
        unanchored_citation_count(report)
    );
    println!(
        "anchor-exempt citations: {}",
        anchor_exempt_citation_count(report)
    );
    println!();
    println!("| kind | status | count |");
    println!("| --- | --- | ---: |");
    for ((kind, status), count) in sorted_kind_status_counts(report) {
        println!("| {kind:?} | {status:?} | {count} |");
    }
    println!();
    println!("Registries covered:");
    for registry in &report.registries {
        let note = registry.note.unwrap_or("-");
        println!(
            "  {} walked={} citations={} note={note}",
            registry.name, registry.walked, registry.citation_count
        );
    }
    println!();
    let unresolved = unresolved_citations(report);
    println!("Unresolved citations: {}", unresolved.len());
    for item in unresolved {
        let citation = &item.citation;
        let target = citation
            .capture
            .as_deref()
            .unwrap_or(citation.file.as_str());
        let line = citation
            .line
            .map(|line| line.to_string())
            .unwrap_or_else(|| "-".to_string());
        let detail = item.detail.as_deref().unwrap_or("-");
        println!(
            "  [{:?}/{:?}] {}:{} ({}) root={} path={}",
            citation.kind, item.status, target, line, detail, citation.root, citation.model_path
        );
    }
}

fn sorted_kind_status_counts(
    report: &ProvenanceWalkReport,
) -> BTreeMap<(CitationKind, ResolveStatus), usize> {
    let mut rows: BTreeMap<(CitationKind, ResolveStatus), usize> = BTreeMap::new();
    for ((kind, status), count) in kind_status_counts(report) {
        rows.insert((kind, status), count);
    }
    rows
}

fn write_inventory_file(report: &ProvenanceWalkReport) {
    let inventory_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(INVENTORY_PATH);
    if let Some(parent) = inventory_path.parent() {
        fs::create_dir_all(parent).expect("create inventory directory");
    }

    let mut body = String::new();
    body.push_str("# P0A Provenance Inventory\n\n");
    body.push_str("Generated by `cargo run --bin provenance_lint -- --write-inventory`. Do not hand-edit.\n\n");
    body.push_str("## Repairs applied\n\n");
    body.push_str(
        "- `commander.rs`: ten `Provenance::source(DEV_CORE, 74)` re-typed to \
         `Provenance::doc` (BlueOS-docs table row, not repo source).\n",
    );
    body.push_str(
        "- `journey.rs` `BLAST_RADIUS_UNKNOWN`: placeholder `Provenance::doc(..., 0)` \
         replaced with `Provenance::asserted` (no file:line for un-annotated blast radius).\n\n",
    );
    body.push_str("## Summary\n\n");
    body.push_str(&format!(
        "- total citations: {}\n",
        file_citation_count(report)
    ));
    body.push_str(&format!(
        "- docs checkout present: {}\n",
        report.docs_root_present
    ));
    body.push_str(&format!(
        "- source-facing unresolved: {}\n",
        source_facing_unresolved(report).len()
    ));
    body.push_str(&format!(
        "- unanchored citations: {}\n",
        unanchored_citation_count(report)
    ));
    body.push_str(&format!(
        "- anchor-exempt citations: {}\n",
        anchor_exempt_citation_count(report)
    ));
    body.push_str(&format!(
        "- fatal unresolved (excluding skipped doc): {}\n\n",
        unresolved_citations(report).len()
    ));

    body.push_str("## Kind x status\n\n");
    body.push_str("| kind | status | count |\n");
    body.push_str("| --- | --- | ---: |\n");
    for ((kind, status), count) in sorted_kind_status_counts(report) {
        body.push_str(&format!("| `{kind:?}` | `{status:?}` | {count} |\n"));
    }
    body.push('\n');

    body.push_str("## Registry coverage\n\n");
    body.push_str("| registry | walked | citations | note |\n");
    body.push_str("| --- | --- | ---: | --- |\n");
    for registry in &report.registries {
        let note = registry.note.unwrap_or("-");
        body.push_str(&format!(
            "| `{}` | {} | {} | {note} |\n",
            registry.name, registry.walked, registry.citation_count
        ));
    }
    body.push('\n');

    body.push_str("## Unresolved citations\n\n");
    for item in unresolved_citations(report) {
        let citation = &item.citation;
        let target = citation
            .capture
            .as_deref()
            .unwrap_or(citation.file.as_str());
        let line = citation
            .line
            .map(|line| line.to_string())
            .unwrap_or_else(|| "-".to_string());
        let detail = item.detail.as_deref().unwrap_or("-");
        body.push_str(&format!(
            "- `[{:?}/{:?}]` `{target}:{line}` root=`{}` path=`{}` detail={detail}\n",
            item.citation.kind, item.status, citation.root, citation.model_path
        ));
    }

    fs::write(inventory_path, body).expect("write inventory");
}
