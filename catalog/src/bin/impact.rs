use std::collections::BTreeMap;
use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

use blueos_catalog::cli::flag_value;
use catalog_paths::repo_root;
use catalog_provenance::source_index::{
    build_source_index_from_walk, git_diff_repo_paths, index_git_diff_pathspecs,
    work_order_for_changed_paths, CatalogEntity, ReviewUrgency, SourceIndexEntry, DOC_SCOPE_NOTE,
    WORK_ORDER_NOTE,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }

    let json_output = args.iter().any(|arg| arg == "--json");
    let path = flag_value(&args, "--path");
    let since = flag_value(&args, "--since");
    let until = flag_value(&args, "--until");

    if until.is_some() && since.is_none() {
        eprintln!("error: --until requires --since");
        return ExitCode::from(2);
    }
    if path.is_some() && (since.is_some() || until.is_some()) {
        eprintln!("error: specify exactly one of --path or --since");
        return ExitCode::from(2);
    }

    match (path.as_deref(), since.as_deref()) {
        (Some(path), None) => run_path(path, json_output),
        (None, Some(since)) => {
            let until_ref = until.as_deref().unwrap_or("HEAD");
            run_since(since, until_ref, json_output)
        }
        (Some(_), Some(_)) => {
            eprintln!("error: specify exactly one of --path or --since");
            ExitCode::from(2)
        }
        (None, None) => {
            eprintln!("error: specify --path or --since");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn run_path(path: &str, json_output: bool) -> ExitCode {
    let index = build_source_index_from_walk();
    let entries = index.entries.get(path).cloned().unwrap_or_default();
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&PathImpact {
                path: path.to_string(),
                entries,
            })
            .expect("serialize path impact")
        );
    } else {
        print_path_impact(path, &entries);
    }
    ExitCode::SUCCESS
}

fn run_since(since: &str, until: &str, json_output: bool) -> ExitCode {
    let repo = repo_root();
    if git_output(&repo, &["rev-parse", "--verify", since]).is_err() {
        eprintln!("error: unknown git ref {since:?}");
        return ExitCode::from(2);
    }
    if git_output(&repo, &["rev-parse", "--verify", until]).is_err() {
        eprintln!("error: unknown git ref {until:?}");
        return ExitCode::from(2);
    }
    let until_resolved = git_output(&repo, &["rev-parse", until]).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(2);
    });

    let index = build_source_index_from_walk();
    let pathspecs = index_git_diff_pathspecs(&index);
    let repo_diff_paths =
        git_diff_repo_paths(&repo, since, until, &pathspecs).unwrap_or_else(|err| {
            eprintln!("{err}");
            std::process::exit(2);
        });
    let repo_diff_path_count = repo_diff_paths.len();
    let impacted_paths: Vec<String> = repo_diff_paths
        .into_iter()
        .filter(|path| index.entries.contains_key(path))
        .collect();

    let order = work_order_for_changed_paths(
        &index,
        since,
        &until_resolved,
        &impacted_paths,
        repo_diff_path_count,
    );
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&order).expect("serialize work order")
        );
    } else {
        print_work_order(&order, &pathspecs);
    }
    ExitCode::SUCCESS
}

fn print_path_impact(path: &str, entries: &[SourceIndexEntry]) {
    println!("impact --path {path}");
    if entries.is_empty() {
        println!("(no catalog entities cite this path)");
        return;
    }
    let mut by_module: BTreeMap<&str, Vec<&SourceIndexEntry>> = BTreeMap::new();
    for entry in entries {
        by_module
            .entry(entry.module.as_str())
            .or_default()
            .push(entry);
    }
    for (module, rows) in by_module {
        println!();
        println!("Module: {module}");
        for entry in rows {
            println!(
                "  {} root={} model={} line={} status={:?}",
                format_entity(&entry.entity),
                entry.root,
                entry.model_path,
                entry
                    .line
                    .map(|line| line.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                entry.status,
            );
        }
    }
}

fn print_work_order(order: &catalog_provenance::source_index::WorkOrder, pathspecs: &[String]) {
    let entry_count: usize = order.modules.values().map(Vec::len).sum();
    println!(
        "impact --since {}..{} ({} impacted indexed paths of {} repo diff paths across {})",
        order.since,
        order.until,
        order.impacted_paths.len(),
        order.repo_diff_path_count,
        pathspecs.join(", ")
    );
    println!("work-order entries: {entry_count}");
    println!();
    println!("{WORK_ORDER_NOTE}");
    println!("{DOC_SCOPE_NOTE}");
    if order.modules.is_empty() {
        println!();
        println!("(no catalog entities cite any changed indexed path)");
        return;
    }
    for (module, rows) in &order.modules {
        println!();
        println!("Module: {module}");
        for row in rows {
            let urgency = match row.urgency {
                ReviewUrgency::NeedsReview => "needs-review",
                ReviewUrgency::ConfirmedDrifted => "confirmed-drifted",
            };
            println!(
                "  [{urgency}] {} path={} model={} line={} status={:?}",
                format_entity(&row.entity),
                row.path,
                row.model_path,
                row.line
                    .map(|line| line.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                row.resolve_status,
            );
        }
    }
}

fn format_entity(entity: &CatalogEntity) -> String {
    match entity {
        CatalogEntity::Service { id } => format!("Service({id})"),
        CatalogEntity::Journey { id } => format!("Journey({id})"),
        CatalogEntity::Page { id } => format!("Page({id})"),
        CatalogEntity::Unknown { reason } => format!("Unknown({reason})"),
    }
}

fn git_output(repo: &Path, cmd: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(cmd)
        .current_dir(repo)
        .output()
        .map_err(|err| format!("failed to spawn git: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            cmd.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn print_help() {
    eprintln!(
        "usage:\n\
         \x20 impact --path <file> [--json]\n\
         \x20 impact --since <tag|sha> [--until <tag|sha|HEAD>] [--json]\n\
         \n\
         --path  list catalog entities that cite a source file\n\
         --since left end of the diff range (required for work orders)\n\
         --until right end of the diff range (default: HEAD)\n\
         \n\
         {DOC_SCOPE_NOTE}"
    );
}

#[derive(serde::Serialize)]
struct PathImpact {
    path: String,
    entries: Vec<SourceIndexEntry>,
}
