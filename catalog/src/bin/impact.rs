use std::collections::BTreeMap;
use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

use blueos_catalog::provenance_walk::repo_root;
use blueos_catalog::source_index::{
    build_source_index_from_walk, index_git_diff_pathspecs, repo_path_to_index_key,
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

    match (path.as_deref(), since.as_deref()) {
        (Some(path), None) => run_path(path, json_output),
        (None, Some(since)) => run_since(since, json_output),
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

fn run_since(since: &str, json_output: bool) -> ExitCode {
    let repo = repo_root();
    let head = git_output(&repo, &["rev-parse", "HEAD"]).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(2);
    });
    if git_output(&repo, &["rev-parse", "--verify", since]).is_err() {
        eprintln!("error: unknown git ref {since:?}");
        return ExitCode::from(2);
    }

    let index = build_source_index_from_walk();
    let pathspecs = index_git_diff_pathspecs(&index);
    let mut diff_cmd = vec!["diff", "--name-only", since, "HEAD", "--"];
    for pathspec in &pathspecs {
        diff_cmd.push(pathspec.as_str());
    }
    let diff = git_output(&repo, &diff_cmd).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(2);
    });
    let repo_diff_paths: Vec<String> = diff
        .lines()
        .filter(|line| !line.is_empty())
        .map(repo_path_to_index_key)
        .collect();
    let repo_diff_path_count = repo_diff_paths.len();
    let impacted_paths: Vec<String> = repo_diff_paths
        .into_iter()
        .filter(|path| index.entries.contains_key(path))
        .collect();

    let order =
        work_order_for_changed_paths(&index, since, &head, &impacted_paths, repo_diff_path_count);
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

fn print_work_order(order: &blueos_catalog::source_index::WorkOrder, pathspecs: &[String]) {
    let entry_count: usize = order.modules.values().map(Vec::len).sum();
    println!(
        "impact --since {}..{} ({} impacted indexed paths of {} repo diff paths across {})",
        order.since,
        order.head,
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

fn flag_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find_map(|pair| {
        if pair[0] == name {
            Some(pair[1].clone())
        } else {
            None
        }
    })
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
         \x20 impact --since <tag|sha> [--json]\n\
         \n\
         --path  list catalog entities that cite a source file\n\
         --since diff repo-local indexed path prefixes from ref to HEAD and emit a work order\n\
         \n\
         {DOC_SCOPE_NOTE}"
    );
}

#[derive(serde::Serialize)]
struct PathImpact {
    path: String,
    entries: Vec<SourceIndexEntry>,
}
