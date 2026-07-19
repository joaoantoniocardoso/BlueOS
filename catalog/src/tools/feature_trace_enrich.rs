//! Enriches `catalog/feature_presence_map.json` with GitHub PR / issue / commit
//! traces via `gh` + `git`, writing `catalog/feature_traces.json` (schema_version 2)
//! consumed by [`crate::feature_trace`].
//!
//! Discovery: path-scoped backport/follow-up PR probing, issue harvest (landing
//! full signals; related PRs closing-only). Invoked by:
//! `cargo run -p blueos-catalog --bin enrich_feature_traces`

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};

use super::shell;

const REPO: &str = "bluerobotics/BlueOS";
const FILES_CAP: usize = 400;
const BODY_CAP: usize = 20000;
// Bound gh resolution for path-hot files (e.g. nginx/frontend shared paths).
const MAX_DISCOVERY_SHAS: usize = 80;

#[derive(Debug, Clone, Serialize)]
struct CommitEntry {
    sha: String,
    short: String,
    subject: String,
    author_name: String,
    author_email: Option<String>,
    authored_at: Option<String>,
    body: String,
    url: String,
    files_changed: Vec<String>,
    files_changed_truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
struct PrEntry {
    number: u64,
    title: Option<String>,
    url: Option<String>,
    state: Option<String>,
    author: Option<String>,
    merged_at: Option<String>,
    closed_at: Option<String>,
    base_ref: Option<String>,
    head_ref: Option<String>,
    labels: Vec<String>,
    body: String,
    body_truncated: bool,
    commit_shas: Vec<String>,
    commit_headlines: Vec<String>,
    files_changed: Vec<String>,
    files_changed_truncated: bool,
    merge_commit_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct IssueEntry {
    number: u64,
    title: Option<String>,
    url: Option<String>,
    state: Option<String>,
    author: Option<String>,
    created_at: Option<String>,
    closed_at: Option<String>,
    labels: Vec<String>,
    missing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct IssueSourceRec {
    kind: String,
    pr: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct ClusterIssueRef {
    number: u64,
    sources: Vec<IssueSourceRec>,
}

#[derive(Debug, Clone, Serialize)]
struct JourneyDiscoveryOut {
    discovery_paths: Vec<String>,
    follow_up_prs: Vec<u64>,
    backport_prs: Vec<u64>,
    issues: Vec<ClusterIssueRef>,
}

#[derive(Debug, Clone, Serialize)]
struct IntroClusterOut {
    intro_commit: String,
    landing_prs: Vec<u64>,
    squash_merge: bool,
    merge_method: String,
    intro_sha_in_pr_commits: bool,
    merge_commit_sha: Option<String>,
    by_journey: BTreeMap<String, JourneyDiscoveryOut>,
}

/// Run-scoped dedup indexes for commits/pull_requests/issues.
struct Ctx<'a> {
    root: &'a Path,
    cache_dir: &'a Path,
    commits: BTreeMap<String, CommitEntry>,
    pull_requests: BTreeMap<u64, PrEntry>,
    issues: BTreeMap<u64, IssueEntry>,
    pr_raw: HashMap<u64, Value>,
}

impl<'a> Ctx<'a> {
    fn new(root: &'a Path, cache_dir: &'a Path) -> Self {
        Self {
            root,
            cache_dir,
            commits: BTreeMap::new(),
            pull_requests: BTreeMap::new(),
            issues: BTreeMap::new(),
            pr_raw: HashMap::new(),
        }
    }

    fn get_commit(&mut self, sha: &str, fallback: Option<(&str, &str)>) -> CommitEntry {
        if !self.commits.contains_key(sha) {
            let entry = build_commit_entry(self.root, sha, fallback);
            self.commits.insert(sha.to_string(), entry);
        }
        self.commits[sha].clone()
    }

    fn get_pr(&mut self, number: u64) -> Result<(Value, PrEntry), String> {
        if !self.pr_raw.contains_key(&number) {
            let raw = gh_pr_detail(self.root, self.cache_dir, number)?;
            self.pr_raw.insert(number, raw);
        }
        let raw = self.pr_raw[&number].clone();
        if let std::collections::btree_map::Entry::Vacant(e) = self.pull_requests.entry(number) {
            let entry = build_pr_entry(number, &raw);
            e.insert(entry);
            if let Some(commits) = raw.get("commits").and_then(|c| c.as_array()) {
                for c in commits {
                    let Some(oid) = c.get("oid").and_then(|o| o.as_str()) else {
                        continue;
                    };
                    let author_names: Vec<String> = c
                        .get("authors")
                        .and_then(|a| a.as_array())
                        .into_iter()
                        .flatten()
                        .filter_map(|a| {
                            a.get("login")
                                .and_then(|v| v.as_str())
                                .or_else(|| a.get("name").and_then(|v| v.as_str()))
                                .map(String::from)
                        })
                        .collect();
                    let subject = c
                        .get("messageHeadline")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let author_name = author_names.join(", ");
                    self.get_commit(oid, Some((subject, &author_name)));
                }
            }
        }
        Ok((raw, self.pull_requests[&number].clone()))
    }

    fn get_issue(&mut self, number: u64) -> IssueEntry {
        if let Some(entry) = self.issues.get(&number) {
            return entry.clone();
        }
        let entry = match gh_issue(self.root, self.cache_dir, number) {
            None => IssueEntry {
                number,
                title: None,
                url: Some(format!("https://github.com/{REPO}/issues/{number}")),
                state: None,
                author: None,
                created_at: None,
                closed_at: None,
                labels: vec![],
                missing: true,
            },
            Some(raw) => IssueEntry {
                number,
                title: raw.get("title").and_then(|v| v.as_str()).map(String::from),
                url: raw.get("url").and_then(|v| v.as_str()).map(String::from),
                state: raw.get("state").and_then(|v| v.as_str()).map(String::from),
                author: raw
                    .get("author")
                    .and_then(|a| a.get("login"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                created_at: raw
                    .get("createdAt")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                closed_at: raw
                    .get("closedAt")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                labels: names_of(raw.get("labels")),
                missing: false,
            },
        };
        self.issues.insert(number, entry.clone());
        entry
    }
}

fn issue_ref_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(concat!(
            r"(?i)",
            r"(?:clos(?:e[sd]?|ing)|fix(?:e[sd]|ing)?|resolv(?:e[sd]?|ing))\s+",
            r"(?P<closing>#\d+(?:\s*(?:,|&|and)\s*#\d+)*)",
            r"|(?:^|[\s(])#(?P<bare>\d{1,6})\b",
            r"|github\.com/[\w.-]+/[\w.-]+/issues/(?P<url>\d+)",
        ))
        .expect("valid ISSUE_REF_RE")
    })
}

// Shared wiring/lock/config paths create false backport/follow-up graphs.
fn discovery_path_noise_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(concat!(
            r"(?:^|/)",
            r"(?:",
            r"package\.json|bun\.lockb|yarn\.lock|package-lock\.json|uv\.lock|",
            r"vite\.config\.[jt]s|menus\.ts|router/index\.ts|",
            r"nginx\.conf|start-blueos-core|pyproject\.toml|",
            r"App\.vue|frontend_services\.ts|",
            r"\.github/",
            r")",
        ))
        .expect("valid DISCOVERY_PATH_NOISE_RE")
    })
}

fn backport_title_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bbackport|\[(?:backport/)?\d+\.\d+").expect("valid BACKPORT_TITLE_RE")
    })
}

fn release_branch_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^origin/(\d+\.\d+)(-dev)?$").expect("valid RELEASE_BRANCH_RE"))
}

fn release_line_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d+\.\d+(-dev)?$").expect("valid RELEASE_LINE_RE"))
}

fn digits_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\d+").expect("valid digits regex"))
}

fn cache_key_sanitize_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[^a-zA-Z0-9._-]").expect("valid cache key sanitize regex"))
}

fn names_of(labels: Option<&Value>) -> Vec<String> {
    labels
        .and_then(|l| l.as_array())
        .into_iter()
        .flatten()
        .filter_map(|lb| lb.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect()
}

fn run_ok_dyn(root: &Path, args: &[String]) -> Option<String> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    shell::run_ok(&refs, root)
}

/// `gh` invocation layer: retries transient 5xx/timeout failures with backoff
/// (see `shell::run_with_retry`). Used by every PR/issue/commit-pulls fetch so
/// all callers benefit; `git` calls keep using the non-retrying `run_ok_dyn`.
fn run_gh_dyn(root: &Path, args: &[String]) -> Result<String, String> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    shell::run_with_retry(|| shell::run(&refs, root))
}

fn run_gh_ok_dyn(root: &Path, args: &[String]) -> Option<String> {
    run_gh_dyn(root, args).ok().filter(|s| !s.is_empty())
}

fn cache_path(cache_dir: &Path, key: &str) -> std::path::PathBuf {
    let safe = cache_key_sanitize_re().replace_all(key, "_");
    cache_dir.join(format!("{safe}.json"))
}

fn cached_json<F: FnOnce() -> Value>(cache_dir: &Path, key: &str, fetcher: F) -> Value {
    let _ = fs::create_dir_all(cache_dir);
    let path = cache_path(cache_dir, key);
    if path.exists() {
        if let Ok(text) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                return v;
            }
        }
    }
    let data = fetcher();
    if let Ok(text) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(&path, text + "\n");
    }
    data
}

fn cached_json_try<F: FnOnce() -> Result<Value, String>>(
    cache_dir: &Path,
    key: &str,
    fetcher: F,
) -> Result<Value, String> {
    let _ = fs::create_dir_all(cache_dir);
    let path = cache_path(cache_dir, key);
    if path.exists() {
        if let Ok(text) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                return Ok(v);
            }
        }
    }
    let data = fetcher()?;
    if let Ok(text) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(&path, text + "\n");
    }
    Ok(data)
}

#[cfg(test)]
mod cache_write_through_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn unique_cache_dir(label: &str) -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("feature_trace_enrich_test_{label}_{n}"))
    }

    #[test]
    fn cached_json_writes_successful_fetch_before_returning() {
        let dir = unique_cache_dir("cached_json");
        let key = "issue_123";
        let result = cached_json(&dir, key, || json!({"number": 123, "title": "ok"}));
        assert_eq!(result, json!({"number": 123, "title": "ok"}));

        let on_disk = fs::read_to_string(cache_path(&dir, key)).expect("cache file must exist");
        let parsed: Value = serde_json::from_str(&on_disk).unwrap();
        assert_eq!(parsed, json!({"number": 123, "title": "ok"}));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cached_json_try_writes_successful_fetch_before_returning() {
        let dir = unique_cache_dir("cached_json_try");
        let key = "pr_v2_456";
        let result: Result<Value, String> =
            cached_json_try(&dir, key, || Ok(json!({"number": 456, "state": "OPEN"})));
        assert_eq!(result, Ok(json!({"number": 456, "state": "OPEN"})));

        let on_disk = fs::read_to_string(cache_path(&dir, key)).expect("cache file must exist");
        let parsed: Value = serde_json::from_str(&on_disk).unwrap();
        assert_eq!(parsed, json!({"number": 456, "state": "OPEN"}));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cached_json_try_does_not_write_on_failure() {
        let dir = unique_cache_dir("cached_json_try_fail");
        let key = "pr_v2_789";
        let result: Result<Value, String> = cached_json_try(&dir, key, || Err("boom".to_string()));
        assert_eq!(result, Err("boom".to_string()));
        assert!(!cache_path(&dir, key).exists());

        let _ = fs::remove_dir_all(&dir);
    }
}

fn cap_list(items: Vec<String>, cap: usize) -> (Vec<String>, bool) {
    if items.len() > cap {
        (items[..cap].to_vec(), true)
    } else {
        (items, false)
    }
}

fn cap_text(text: &str, cap: usize) -> (String, bool) {
    if text.chars().count() > cap {
        (text.chars().take(cap).collect(), true)
    } else {
        (text.to_string(), false)
    }
}

fn extract_issue_numbers(texts: &[Option<&str>]) -> BTreeSet<u64> {
    let mut found = BTreeSet::new();
    let re = issue_ref_re();
    for text in texts.iter().flatten() {
        for cap in re.captures_iter(text) {
            if let Some(closing) = cap.name("closing") {
                for d in digits_re().find_iter(closing.as_str()) {
                    if let Ok(n) = d.as_str().parse::<u64>() {
                        found.insert(n);
                    }
                }
            } else if let Some(bare) = cap.name("bare") {
                if let Ok(n) = bare.as_str().parse::<u64>() {
                    found.insert(n);
                }
            } else if let Some(url) = cap.name("url") {
                if let Ok(n) = url.as_str().parse::<u64>() {
                    found.insert(n);
                }
            }
        }
    }
    found
}

fn discover_release_branches(root: &Path) -> Vec<String> {
    let Some(out) = shell::run_ok(&["git", "branch", "-r"], root) else {
        return vec![];
    };
    let mut branches: BTreeSet<String> = BTreeSet::new();
    for line in out.lines() {
        if line.contains("->") {
            continue;
        }
        let name = line.trim();
        if release_branch_re().is_match(name) {
            branches.insert(name.to_string());
        }
    }
    branches.into_iter().collect()
}

fn module_tokens(module: &str) -> BTreeSet<String> {
    let module_lower = module.to_lowercase();
    let mut tokens: BTreeSet<String> = BTreeSet::new();
    tokens.insert(module_lower.clone());
    tokens.insert(module_lower.replace('_', "-"));
    if let Some(first) = module.split('_').next() {
        if module.contains('_') {
            let first_lower = first.to_lowercase();
            if first_lower.len() >= 4 {
                tokens.insert(first_lower);
            }
        }
    }
    tokens
}

fn matches_module(path: &str, module: &str, tokens: &BTreeSet<String>) -> bool {
    let pl = path.to_lowercase();
    path.contains(&format!("/services/{module}/"))
        || path.contains(&format!("/{module}/"))
        || tokens.iter().any(|t| pl.contains(t.as_str()))
}

fn camel_boundary_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"([a-z0-9])([A-Z])").expect("valid camel boundary regex"))
}

/// Extend module tokens with words pulled from the module's own hint paths
/// (journey path override / `MODULE_DEFAULT_PATH`). Covers modules whose
/// declared name (e.g. `zenohd`) doesn't literally appear in its actual
/// source directory (`components/zenoh-inspector/`), without loosening the
/// module gate for modules that have no such hint (MODULE_S binaries).
fn hint_word_tokens(hint_paths: &[String]) -> BTreeSet<String> {
    const STOP_WORDS: &[&str] = &[
        "view",
        "vue",
        "manager",
        "index",
        "config",
        "component",
        "store",
        "main",
        "default",
        "service",
        "settings",
        "core",
        "services",
    ];
    let camel = camel_boundary_re();
    let mut tokens = BTreeSet::new();
    for path in hint_paths {
        let stem = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let spaced = camel.replace_all(stem, "$1 $2");
        for word in spaced.split(|c: char| !c.is_ascii_alphanumeric()) {
            let lower = word.to_lowercase();
            if lower.len() >= 4 && !STOP_WORDS.contains(&lower.as_str()) {
                tokens.insert(lower);
            }
        }
    }
    tokens
}

/// Keep feature-local paths; drop shared wiring that creates false-positive graphs.
///
/// `hint_paths` are module-specific path hints (journey overrides, module default
/// path, `MODULE_S` focused paths) used as the last-resort fallback. Unlike a
/// generic `/services/` sweep, they stay scoped to the journey's own module, so a
/// bootstrap-era commit that touches many unrelated services can't leak into
/// every journey sharing that commit (see `BOOTSTRAP_SPLIT.md`).
fn discovery_paths(paths: &[String], module: Option<&str>, hint_paths: &[String]) -> Vec<String> {
    let noise = discovery_path_noise_re();
    let mut cleaned: Vec<String> = paths
        .iter()
        .filter(|p| !p.is_empty() && !noise.is_match(p))
        .cloned()
        .collect();
    if cleaned.is_empty() {
        cleaned = paths.iter().filter(|p| !p.is_empty()).cloned().collect();
    }

    if let Some(module) = module {
        let mut tokens = module_tokens(module);
        tokens.extend(hint_word_tokens(hint_paths));
        let preferred: Vec<String> = cleaned
            .iter()
            .filter(|p| matches_module(p, module, &tokens))
            .cloned()
            .collect();
        if !preferred.is_empty() {
            return dedup_sorted(widen_with_feature_dirs(preferred));
        }

        let focused: Vec<String> = cleaned
            .iter()
            .filter(|p| {
                (p.contains("/components/") || p.contains("/views/") || p.contains("/store/"))
                    && matches_module(p, module, &tokens)
            })
            .cloned()
            .collect();
        if !focused.is_empty() {
            return dedup_sorted(widen_with_feature_dirs(focused));
        }

        let hints: Vec<String> = hint_paths
            .iter()
            .filter(|p| !p.is_empty())
            .cloned()
            .collect();
        return dedup_sorted(widen_with_feature_dirs(hints));
    }

    let services: Vec<String> = cleaned
        .iter()
        .filter(|p| p.contains("/services/"))
        .cloned()
        .collect();
    if !services.is_empty() {
        return dedup_sorted(services);
    }
    let focused: Vec<String> = cleaned
        .iter()
        .filter(|p| p.contains("/components/") || p.contains("/views/") || p.contains("/store/"))
        .cloned()
        .collect();
    if !focused.is_empty() {
        return dedup_sorted(widen_with_feature_dirs(focused));
    }
    dedup_sorted(cleaned)
}

/// `ttyd` / `linux2rest` / etc. have no in-repo tree. Allow `start-blueos-core`
/// only when paired with a MODULE_S pickaxe token filter on follow-up PRs.
fn module_s_token(module: &str) -> Option<&'static str> {
    use super::feature_presence::MODULE_S;
    MODULE_S
        .iter()
        .find(|(m, _, _)| *m == module)
        .map(|(_, term, _)| *term)
}

fn entry_mentions_token(entry: &PrEntry, token: &str) -> bool {
    let t = token.to_lowercase();
    if entry
        .title
        .as_deref()
        .unwrap_or("")
        .to_lowercase()
        .contains(&t)
    {
        return true;
    }
    if entry.body.to_lowercase().contains(&t) {
        return true;
    }
    entry
        .files_changed
        .iter()
        .any(|f| f.to_lowercase().contains(&t))
}

/// All `OVERRIDES` rows (`Path` and `Pickaxe`'s path field) declared for a
/// journey. Multi-row journeys (e.g. `ViewCameraStreams`) get every row, not
/// just the first match — see `PRECISION_DESIGN.md` §2.
fn journey_override_paths(journey_id: &str) -> Vec<String> {
    use super::feature_presence::{Override, OVERRIDES};

    OVERRIDES
        .iter()
        .filter(|(k, _)| *k == journey_id)
        .map(|(_, ov)| match ov {
            Override::Path(path) => (*path).to_string(),
            Override::Pickaxe(_, path) => (*path).to_string(),
        })
        .collect()
}

/// The `Override::Pickaxe` term declared for a journey, if any. Unlike
/// `journey_override_paths` (which every override row contributes to), only
/// `Pickaxe` rows carry a term — `Path` rows have none.
fn journey_pickaxe_term(journey_id: &str) -> Option<&'static str> {
    use super::feature_presence::{Override, OVERRIDES};

    OVERRIDES.iter().find_map(|(k, ov)| {
        if *k != journey_id {
            return None;
        }
        match ov {
            Override::Pickaxe(term, _) => Some(*term),
            Override::Path(_) => None,
        }
    })
}

/// Path hints for a journey's own module, drawn from the same tables
/// `generate_feature_presence` uses to resolve intro commits: journey path
/// overrides, the module's default service/component path, and nginx's config.
fn module_hint_paths(journey_id: &str, module: &str) -> Vec<String> {
    use super::feature_presence::{MODULE_DEFAULT_PATH, MODULE_S};

    let mut hints: Vec<String> = journey_override_paths(journey_id);
    if let Some((_, path)) = MODULE_DEFAULT_PATH.iter().find(|(m, _)| *m == module) {
        hints.push((*path).to_string());
    }
    if MODULE_S.iter().any(|(m, _, _)| *m == module) && module == "nginx" {
        hints.push("core/tools/nginx/nginx.conf".to_string());
    }
    hints
}

/// Precedence per `PRECISION_DESIGN.md` §1: an explicit journey override is the
/// discovery path — full stop. Broad module-token candidate matching (which can
/// leak a sibling's path, e.g. `store/mavlink.ts` for a camera journey) is not
/// consulted when an override exists.
fn resolve_journey_discovery_paths(
    journey_id: &str,
    module: &str,
    candidate_paths: &[String],
    hint_paths: &[String],
) -> Vec<String> {
    let overrides = journey_override_paths(journey_id);
    if !overrides.is_empty() {
        return dedup_sorted(widen_with_feature_dirs(overrides));
    }
    discovery_paths(candidate_paths, Some(module), hint_paths)
}

/// Add the immediate parent directory of each file when it is a feature folder
/// (not a generic hub like `components` / `configuration` / `src`). That covers
/// siblings added later (`zenoh-inspector/ZenohNetwork.vue`) without widening all
/// the way to coarse hubs (`vehiclesetup/`), which pulls unrelated follow-ups.
fn widen_with_feature_dirs(paths: Vec<String>) -> Vec<String> {
    const HUBS: &[&str] = &[
        "components",
        "views",
        "store",
        "types",
        "src",
        "frontend",
        "core",
        "services",
        "configuration",
        "overview",
        "common",
        "utils",
        "libs",
        // Feature-component hubs whose sibling journeys each already have a
        // distinct file-level `OVERRIDES` row (see `PRECISION_INVENTORY.md`);
        // widening to the shared directory would re-merge those siblings'
        // discovery_paths (e.g. `ConfigureCameraStream`/`ViewCameraStreams`,
        // `AddCustomManifest`/`BrowseExtensionStore`, wifi dialogs/manager).
        "video-manager",
        "kraken",
        "wifi",
    ];
    let mut out: BTreeSet<String> = paths.into_iter().collect();
    let parents: Vec<String> = out
        .iter()
        .filter_map(|p| {
            let parent = std::path::Path::new(p).parent()?;
            let name = parent.file_name()?.to_str()?;
            if HUBS.contains(&name) {
                return None;
            }
            Some(parent.to_string_lossy().replace('\\', "/"))
        })
        .collect();
    out.extend(parents);
    out.into_iter().collect()
}

fn dedup_sorted(items: Vec<String>) -> Vec<String> {
    let set: BTreeSet<String> = items.into_iter().collect();
    set.into_iter().collect()
}

fn git_log_shas(
    root: &Path,
    ref_: &str,
    paths: &[String],
    since_exclusive_sha: Option<&str>,
    max_shas: usize,
    pickaxe_term: Option<&str>,
) -> Vec<String> {
    if paths.is_empty() {
        return vec![];
    }
    let rev = match since_exclusive_sha {
        Some(sha) => format!("{sha}..{ref_}"),
        None => ref_.to_string(),
    };
    let mut cmd: Vec<String> = vec!["git".into(), "log".into(), "--format=%H".into(), rev];
    if let Some(term) = pickaxe_term {
        // Real `-S` pickaxe: only commits whose diff adds/removes this term, not
        // just commits that touch the path (see `IMPROVE_AUDIT.md` §T3 — a plain
        // path-scoped log can't separate siblings sharing one file/handler).
        cmd.push("-S".into());
        cmd.push(term.to_string());
    }
    cmd.push("--".into());
    cmd.extend(paths.iter().cloned());
    let Some(out) = run_ok_dyn(root, &cmd) else {
        return vec![];
    };
    out.lines()
        .filter(|s| !s.trim().is_empty())
        .take(max_shas)
        .map(String::from)
        .collect()
}

fn git_files_changed(root: &Path, sha: &str) -> (Vec<String>, bool) {
    let Some(out) = shell::run_ok(&["git", "show", "--name-only", "--format=", sha], root) else {
        return (vec![], false);
    };
    let files: Vec<String> = out
        .lines()
        .filter(|f| !f.trim().is_empty())
        .map(String::from)
        .collect();
    cap_list(files, FILES_CAP)
}

fn build_commit_entry(root: &Path, sha: &str, fallback: Option<(&str, &str)>) -> CommitEntry {
    let Some(out) = shell::run_ok(
        &[
            "git",
            "show",
            "-s",
            "--format=%H%n%s%n%an%n%ae%n%aI%n%b",
            sha,
        ],
        root,
    ) else {
        // Sha not present locally (e.g. a squash-merged PR's original branch commits).
        return CommitEntry {
            sha: sha.to_string(),
            short: sha.chars().take(12).collect(),
            subject: fallback.map(|f| f.0.to_string()).unwrap_or_default(),
            author_name: fallback.map(|f| f.1.to_string()).unwrap_or_default(),
            author_email: None,
            authored_at: None,
            body: String::new(),
            url: format!("https://github.com/{REPO}/commit/{sha}"),
            files_changed: vec![],
            files_changed_truncated: false,
        };
    };
    let lines: Vec<&str> = out.split('\n').collect();
    let body = if lines.len() > 5 {
        lines[5..].join("\n")
    } else {
        String::new()
    };
    let (files, files_truncated) = git_files_changed(root, sha);
    let full_sha = lines.first().copied().unwrap_or_default();
    CommitEntry {
        sha: full_sha.to_string(),
        short: full_sha.chars().take(12).collect(),
        subject: lines.get(1).copied().unwrap_or_default().to_string(),
        author_name: lines.get(2).copied().unwrap_or_default().to_string(),
        author_email: lines.get(3).map(|s| s.to_string()),
        authored_at: lines.get(4).map(|s| s.to_string()),
        body,
        url: format!("https://github.com/{REPO}/commit/{full_sha}"),
        files_changed: files,
        files_changed_truncated: files_truncated,
    }
}

fn gh_commit_pulls(root: &Path, cache_dir: &Path, sha: &str) -> Vec<Value> {
    let key = format!("commit_pulls_{}", &sha[..sha.len().min(12)]);
    let data = cached_json(cache_dir, &key, || {
        let raw = run_gh_ok_dyn(
            root,
            &[
                "gh".into(),
                "api".into(),
                format!("repos/{REPO}/commits/{sha}/pulls"),
            ],
        );
        shell::sleep_cold();
        match raw {
            None => Value::Array(vec![]),
            Some(s) => serde_json::from_str(&s).unwrap_or(Value::Array(vec![])),
        }
    });
    data.as_array().cloned().unwrap_or_default()
}

fn gh_pr_detail(root: &Path, cache_dir: &Path, number: u64) -> Result<Value, String> {
    let key = format!("pr_v2_{number}");
    // v2 cache key: v1's `pr_{number}.json` cache lacks `files`/`mergeCommit`, so
    // reuse under a new key rather than silently returning stale (incomplete) data.
    cached_json_try(cache_dir, &key, || {
        let out = run_gh_dyn(
            root,
            &[
                "gh".into(),
                "pr".into(),
                "view".into(),
                number.to_string(),
                "--repo".into(),
                REPO.into(),
                "--json".into(),
                "number,title,url,state,author,mergedAt,closedAt,body,closingIssuesReferences,\
                 commits,labels,baseRefName,headRefName,files,mergeCommit"
                    .into(),
            ],
        )?;
        shell::sleep_cold();
        serde_json::from_str(&out).map_err(|e| e.to_string())
    })
}

fn build_pr_entry(number: u64, raw: &Value) -> PrEntry {
    let files_raw: Vec<String> = raw
        .get("files")
        .and_then(|f| f.as_array())
        .into_iter()
        .flatten()
        .filter_map(|f| f.get("path").and_then(|p| p.as_str()).map(String::from))
        .collect();
    let (files, files_truncated) = cap_list(files_raw, FILES_CAP);
    let body_raw = raw.get("body").and_then(|b| b.as_str()).unwrap_or("");
    let (body, body_truncated) = cap_text(body_raw, BODY_CAP);
    let mut commit_shas: Vec<String> = vec![];
    let mut commit_headlines: Vec<String> = vec![];
    for c in raw
        .get("commits")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        let Some(oid) = c.get("oid").and_then(|o| o.as_str()) else {
            continue;
        };
        commit_shas.push(oid.to_string());
        commit_headlines.push(
            c.get("messageHeadline")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        );
    }
    PrEntry {
        number,
        title: raw.get("title").and_then(|v| v.as_str()).map(String::from),
        url: raw.get("url").and_then(|v| v.as_str()).map(String::from),
        state: raw.get("state").and_then(|v| v.as_str()).map(String::from),
        author: raw
            .get("author")
            .and_then(|a| a.get("login"))
            .and_then(|v| v.as_str())
            .map(String::from),
        merged_at: raw
            .get("mergedAt")
            .and_then(|v| v.as_str())
            .map(String::from),
        closed_at: raw
            .get("closedAt")
            .and_then(|v| v.as_str())
            .map(String::from),
        base_ref: raw
            .get("baseRefName")
            .and_then(|v| v.as_str())
            .map(String::from),
        head_ref: raw
            .get("headRefName")
            .and_then(|v| v.as_str())
            .map(String::from),
        labels: names_of(raw.get("labels")),
        body,
        body_truncated,
        commit_shas,
        commit_headlines,
        files_changed: files,
        files_changed_truncated: files_truncated,
        merge_commit_sha: raw
            .get("mergeCommit")
            .and_then(|m| m.get("oid"))
            .and_then(|v| v.as_str())
            .map(String::from),
    }
}

fn gh_issue(root: &Path, cache_dir: &Path, number: u64) -> Option<Value> {
    let key = format!("issue_{number}");
    let data = cached_json(cache_dir, &key, || {
        let raw = run_gh_ok_dyn(
            root,
            &[
                "gh".into(),
                "issue".into(),
                "view".into(),
                number.to_string(),
                "--repo".into(),
                REPO.into(),
                "--json".into(),
                "number,title,url,state,author,createdAt,closedAt,labels".into(),
            ],
        );
        shell::sleep_cold();
        match raw {
            None => Value::Null,
            Some(s) => serde_json::from_str(&s).unwrap_or(Value::Null),
        }
    });
    if data.is_null() {
        None
    } else {
        Some(data)
    }
}

/// Confirmatory pass: issues that cross-reference this PR in their timeline.
fn gh_pr_timeline_issue_refs(root: &Path, cache_dir: &Path, number: u64) -> Vec<u64> {
    let key = format!("pr_timeline_{number}");
    let data = cached_json(cache_dir, &key, || {
        let raw = run_gh_ok_dyn(
            root,
            &[
                "gh".into(),
                "api".into(),
                format!("repos/{REPO}/issues/{number}/timeline"),
                "--paginate".into(),
            ],
        );
        shell::sleep_cold();
        let Some(s) = raw else {
            return Value::Array(vec![]);
        };
        let Ok(events) = serde_json::from_str::<Value>(&s) else {
            return Value::Array(vec![]);
        };
        let mut refs = vec![];
        for e in events.as_array().into_iter().flatten() {
            if e.get("event").and_then(|v| v.as_str()) != Some("cross-referenced") {
                continue;
            }
            let Some(issue) = e.get("source").and_then(|s| s.get("issue")) else {
                continue;
            };
            if issue.get("pull_request").is_some() {
                continue; // referencer is itself a PR, not a plain issue
            }
            if let Some(n) = issue.get("number").and_then(|v| v.as_u64()) {
                refs.push(Value::from(n));
            }
        }
        Value::Array(refs)
    });
    data.as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_u64())
        .collect()
}

/// Probe §A: path-scoped git log over each release line, restricted to `origin/<branch>`.
fn discover_backport_prs(
    ctx: &mut Ctx,
    paths: &[String],
    intro_sha: &str,
) -> Result<Vec<u64>, String> {
    let mut candidates: BTreeSet<u64> = BTreeSet::new();
    for branch in discover_release_branches(ctx.root) {
        for sha in git_log_shas(ctx.root, &branch, paths, None, MAX_DISCOVERY_SHAS, None) {
            if sha == intro_sha {
                continue;
            }
            for p in gh_commit_pulls(ctx.root, ctx.cache_dir, &sha) {
                if let Some(n) = p.get("number").and_then(|v| v.as_u64()) {
                    candidates.insert(n);
                }
            }
        }
    }

    let mut backport_prs = vec![];
    for number in candidates {
        let (raw, _entry) = ctx.get_pr(number)?;
        let base = raw
            .get("baseRefName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let title = raw.get("title").and_then(|v| v.as_str()).unwrap_or("");
        if !release_line_re().is_match(base) {
            continue;
        }
        // Require explicit backport markers — release-line PRs that merely touch
        // shared wiring (historical 1.0.x mods, etc.) are not feature backports.
        if !backport_title_re().is_match(title) {
            continue;
        }
        backport_prs.push(number);
    }
    Ok(backport_prs)
}

/// Probe §B: path-scoped git log intro..master, minus the landing PR.
/// When `require_token` is set (MODULE_S via start-blueos-core), keep only PRs
/// that mention that token in title/body/files.
/// When `pickaxe_term` is set (a journey's `Override::Pickaxe` term, e.g.
/// `start_ardupilot`), the underlying git log is `-S`-scoped to commits whose
/// diff actually touches that term, separating siblings that share one file
/// (see `IMPROVE_AUDIT.md` §T3).
fn discover_follow_up_prs(
    ctx: &mut Ctx,
    paths: &[String],
    intro_sha: &str,
    landing_prs: &BTreeSet<u64>,
    require_token: Option<&str>,
    pickaxe_term: Option<&str>,
) -> Result<Vec<u64>, String> {
    let mut candidates: BTreeSet<u64> = BTreeSet::new();
    for sha in git_log_shas(
        ctx.root,
        "origin/master",
        paths,
        Some(intro_sha),
        MAX_DISCOVERY_SHAS,
        pickaxe_term,
    ) {
        for p in gh_commit_pulls(ctx.root, ctx.cache_dir, &sha) {
            if let Some(n) = p.get("number").and_then(|v| v.as_u64()) {
                candidates.insert(n);
            }
        }
    }

    let mut follow_up_prs = vec![];
    for number in candidates {
        if landing_prs.contains(&number) {
            continue;
        }
        let (_raw, entry) = ctx.get_pr(number)?;
        if is_incidental_follow_up(&entry, paths) {
            continue;
        }
        if let Some(token) = require_token {
            if !entry_mentions_token(&entry, token) {
                continue;
            }
        }
        follow_up_prs.push(number);
    }
    Ok(follow_up_prs)
}

fn discovery_path_covers(file: &str, discovery_paths: &[String]) -> bool {
    discovery_paths
        .iter()
        .any(|p| file == p.as_str() || file.starts_with(&format!("{p}/")))
}

fn covered_file_count(files_changed: &[String], discovery_paths: &[String]) -> usize {
    files_changed
        .iter()
        .filter(|f| discovery_path_covers(f, discovery_paths))
        .count()
}

/// Title substrings for repo-wide sweeps that only incidentally touch feature paths.
const FOLLOW_UP_TITLE_DENY_SUBSTR: &[&str] = &[
    "isort",
    "pylint",
    "ruff",
    "mypy",
    "flake8",
    "eslint",
    "prettier",
    "clippy",
    "random typo",
    "random typos",
    "typo fix",
    "rework project to use",
    "sanitize lib",
    "update kraken",
    "100% mobile",
    "mobile friendly",
    "sentry sdk",
    "limit_ram_usage",
    "uv package manager",
    "migrate to uv",
    "pyproject.toml",
    "poetry to ",
    "chaining operator",
    "optional chaining",
    "base image",
];

fn follow_up_title_denied(title: &str) -> bool {
    let lower = title.to_lowercase();
    FOLLOW_UP_TITLE_DENY_SUBSTR
        .iter()
        .any(|needle| lower.contains(needle))
        || {
            // Word-ish match for short lint tokens that appear as whole words.
            lower
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .any(|w| matches!(w, "lint" | "lints" | "linter" | "linting" | "black"))
        }
}

/// Drop incidental follow-ups: title denylist, thin path overlap on medium PRs,
/// and repo-wide sweep fraction on large PRs.
fn is_incidental_follow_up(entry: &PrEntry, discovery_paths: &[String]) -> bool {
    let title = entry.title.as_deref().unwrap_or("");
    if follow_up_title_denied(title) {
        return true;
    }

    let files = &entry.files_changed;
    let covered = covered_file_count(files, discovery_paths);

    // Medium PRs that only graze one discovery path are almost always sweeps.
    if files.len() > 12 && covered < 2 {
        return true;
    }

    // Large diffs must still be mostly about this feature.
    if files.len() > 20 {
        let frac = covered as f64 / files.len() as f64;
        if frac < 0.10 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod incidental_follow_up_tests {
    use super::*;

    fn pr(title: &str, files: &[&str]) -> PrEntry {
        PrEntry {
            number: 1,
            title: Some(title.to_string()),
            url: None,
            state: None,
            author: None,
            merged_at: None,
            closed_at: None,
            base_ref: None,
            head_ref: None,
            labels: vec![],
            body: String::new(),
            body_truncated: false,
            commit_shas: vec![],
            commit_headlines: vec![],
            files_changed: files.iter().map(|s| (*s).to_string()).collect(),
            files_changed_truncated: false,
            merge_commit_sha: None,
        }
    }

    #[test]
    fn title_denylist_drops_isort() {
        let paths = vec!["core/services/helper".into()];
        assert!(is_incidental_follow_up(
            &pr("Core: isort fixes", &["core/services/helper/main.py"]),
            &paths
        ));
    }

    #[test]
    fn medium_pr_with_single_graze_dropped() {
        let paths = vec!["core/frontend/src/components/zenoh-inspector".into()];
        let mut files = vec!["core/frontend/src/components/zenoh-inspector/A.vue".to_string()];
        for i in 0..13 {
            files.push(format!("core/unrelated/file_{i}.py"));
        }
        let mut entry = pr("Tweaks around the tree", &[]);
        entry.files_changed = files;
        assert!(is_incidental_follow_up(&entry, &paths));
    }

    #[test]
    fn zenoh_style_medium_pr_with_two_hits_kept() {
        let paths = vec!["core/frontend/src/components/zenoh-inspector".into()];
        let mut files = vec![
            "core/frontend/src/components/zenoh-inspector/A.vue".to_string(),
            "core/frontend/src/components/zenoh-inspector/B.vue".to_string(),
        ];
        for i in 0..12 {
            files.push(format!("core/services/other/f_{i}.py"));
        }
        let mut entry = pr("Update zenoh and enable shared memory", &[]);
        entry.files_changed = files;
        assert!(!is_incidental_follow_up(&entry, &paths));
    }

    #[test]
    fn small_feature_pr_kept() {
        let paths = vec!["core/services/disk_usage".into()];
        assert!(!is_incidental_follow_up(
            &pr(
                "core: services: disk_usage: Add speed test",
                &[
                    "core/services/disk_usage/main.py",
                    "core/frontend/src/views/Disk.vue",
                ],
            ),
            &paths
        ));
    }
}

#[cfg(test)]
mod journey_override_tests {
    use super::*;

    #[test]
    fn sibling_overrides_do_not_leak_into_each_other() {
        // InspectZenohNetwork / RunLanSpeedTest are unrelated journeys with
        // distinct OVERRIDES rows sharing a hypothetical commit's file list.
        let candidate_paths = vec![
            "core/frontend/src/views/ZenohInspectorView.vue".to_string(),
            "core/services/pardal".to_string(),
            "core/start-blueos-core".to_string(),
        ];

        let zenoh_paths =
            resolve_journey_discovery_paths("InspectZenohNetwork", "zenohd", &candidate_paths, &[]);
        let pardal_paths =
            resolve_journey_discovery_paths("RunLanSpeedTest", "pardal", &candidate_paths, &[]);

        assert!(zenoh_paths
            .iter()
            .any(|p| p.contains("ZenohInspectorView.vue")));
        assert!(!zenoh_paths.iter().any(|p| p.contains("pardal")));

        assert!(pardal_paths.iter().any(|p| p.contains("pardal")));
        assert!(!pardal_paths
            .iter()
            .any(|p| p.contains("ZenohInspectorView.vue")));
    }

    #[test]
    fn camera_journey_discovery_excludes_mavlink_store_leak() {
        // A shared bootstrap-era commit that also touched mavlink2rest's
        // frontend store — the historical source of the store/mavlink.ts leak.
        let candidate_paths = vec![
            "core/frontend/src/store/video.ts".to_string(),
            "core/frontend/src/components/video-manager/VideoManager.vue".to_string(),
            "core/frontend/src/store/mavlink.ts".to_string(),
            "core/start-blueos-core".to_string(),
        ];

        let paths = resolve_journey_discovery_paths(
            "ViewCameraStreams",
            "mavlink_camera_manager",
            &candidate_paths,
            &module_hint_paths("ViewCameraStreams", "mavlink_camera_manager"),
        );

        assert!(paths.iter().any(|p| p.contains("store/video.ts")));
        assert!(paths.iter().any(|p| p.contains("VideoManager.vue")));
        assert!(!paths.iter().any(|p| p.contains("mavlink.ts")));
    }

    #[test]
    fn camera_sibling_journeys_are_not_remerged_by_video_manager_widening() {
        // ConfigureCameraStream's own OVERRIDES row lives in the same
        // `video-manager/` dir as ViewCameraStreams' — widening must not pull
        // the whole shared dir (or ViewCameraStreams' VideoManager.vue) back in.
        let candidate_paths = vec![
            "core/frontend/src/components/video-manager/VideoStreamCreationDialog.vue".to_string(),
            "core/start-blueos-core".to_string(),
        ];

        let configure_paths = resolve_journey_discovery_paths(
            "ConfigureCameraStream",
            "mavlink_camera_manager",
            &candidate_paths,
            &module_hint_paths("ConfigureCameraStream", "mavlink_camera_manager"),
        );
        let view_paths = resolve_journey_discovery_paths(
            "ViewCameraStreams",
            "mavlink_camera_manager",
            &candidate_paths,
            &module_hint_paths("ViewCameraStreams", "mavlink_camera_manager"),
        );

        assert_ne!(configure_paths, view_paths);
        assert!(!configure_paths
            .iter()
            .any(|p| p.contains("VideoManager.vue")
                || p == "core/frontend/src/components/video-manager"));
        assert!(configure_paths
            .iter()
            .any(|p| p.contains("VideoStreamCreationDialog.vue")));
    }
}

/// BlueOS disables squash/merge-commit merges repo-wide (rebase-only), so
/// `merge_commit_sha not in commit_shas` is always true and can't tell rebase
/// from squash on its own (see `SQUASH_QA.md`). Classify via: parent count on
/// `merge_commit_sha` (>=2 ⇒ an actual merge commit); otherwise compare the
/// rebased tip's subject chain to the PR's pre-rebase commit headlines —
/// matching (in either order) means rebase, a mismatch means squash.
fn classify_merge_method(
    root: &Path,
    merge_commit_sha: Option<&str>,
    headlines: &[String],
) -> String {
    let Some(merge_sha) = merge_commit_sha else {
        return "unknown".to_string();
    };
    let Some(parents_out) =
        shell::run_ok(&["git", "rev-list", "--parents", "-n1", merge_sha], root)
    else {
        return "unknown".to_string();
    };
    if parents_out.split_whitespace().count() >= 3 {
        return "merge_commit".to_string();
    }
    if headlines.is_empty() {
        return "unknown".to_string();
    }
    let n_arg = format!("-n{}", headlines.len());
    let Some(chain_out) = run_ok_dyn(
        root,
        &[
            "git".into(),
            "log".into(),
            "--format=%s".into(),
            n_arg,
            merge_sha.into(),
        ],
    ) else {
        return "unknown".to_string();
    };
    let chain: Vec<&str> = chain_out.lines().collect();
    let forward: Vec<&str> = headlines.iter().map(String::as_str).collect();
    let reversed: Vec<&str> = forward.iter().rev().copied().collect();
    if subject_chains_match(&chain, &forward) || subject_chains_match(&chain, &reversed) {
        "rebase".to_string()
    } else {
        "squash".to_string()
    }
}

/// `gh`'s `messageHeadline` truncates long subjects with a trailing `…`
/// (unlike `git log --format=%s`), so compare as a prefix when truncated.
fn subject_chains_match(git_subjects: &[&str], gh_headlines: &[&str]) -> bool {
    git_subjects.len() == gh_headlines.len()
        && git_subjects
            .iter()
            .zip(gh_headlines.iter())
            .all(
                |(git_subject, gh_headline)| match gh_headline.strip_suffix('…') {
                    Some(prefix) => git_subject.starts_with(prefix),
                    None => git_subject == gh_headline,
                },
            )
}

fn add_source(
    cluster_sources: &mut BTreeMap<u64, Vec<IssueSourceRec>>,
    number: u64,
    kind: &str,
    pr: Option<u64>,
) {
    let recs = cluster_sources.entry(number).or_default();
    let rec = IssueSourceRec {
        kind: kind.to_string(),
        pr,
    };
    if !recs.contains(&rec) {
        recs.push(rec);
    }
}

fn build_intro_cluster(
    ctx: &mut Ctx,
    intro_sha: &str,
    group: &[Value],
) -> Result<IntroClusterOut, String> {
    let intro_commit = ctx.get_commit(intro_sha, None);

    let mut landing_prs_set: BTreeSet<u64> = BTreeSet::new();
    for p in gh_commit_pulls(ctx.root, ctx.cache_dir, intro_sha) {
        if let Some(n) = p.get("number").and_then(|v| v.as_u64()) {
            landing_prs_set.insert(n);
        }
    }
    let landing_prs: Vec<u64> = landing_prs_set.iter().copied().collect();
    for &number in &landing_prs {
        ctx.get_pr(number)?;
    }

    let primary_entry = landing_prs
        .first()
        .and_then(|n| ctx.pull_requests.get(n).cloned());
    let primary_commit_shas = primary_entry
        .as_ref()
        .map(|e| e.commit_shas.clone())
        .unwrap_or_default();
    let merge_commit_sha = primary_entry
        .as_ref()
        .and_then(|e| e.merge_commit_sha.clone());
    let primary_commit_headlines = primary_entry
        .as_ref()
        .map(|e| e.commit_headlines.clone())
        .unwrap_or_default();
    let intro_sha_in_pr_commits = primary_commit_shas.iter().any(|s| s == intro_sha);
    let merge_method = classify_merge_method(
        ctx.root,
        merge_commit_sha.as_deref(),
        &primary_commit_headlines,
    );
    let squash_merge = merge_method == "squash";

    let mut candidate_paths: BTreeSet<String> =
        intro_commit.files_changed.iter().cloned().collect();
    if let Some(entry) = &primary_entry {
        candidate_paths.extend(entry.files_changed.iter().cloned());
    }
    let candidate_paths: Vec<String> = candidate_paths.into_iter().collect();

    // Facts about the landing commit/PR (issue refs on intro commit + full
    // harvest on landing PRs) are shared by every journey in this cluster —
    // computed once, then extended per-journey with that journey's own
    // backport/follow-up closing-issue refs.
    let mut shared_sources: BTreeMap<u64, Vec<IssueSourceRec>> = BTreeMap::new();
    let primary_pr = landing_prs.first().copied();
    for number in extract_issue_numbers(&[
        Some(intro_commit.subject.as_str()),
        Some(intro_commit.body.as_str()),
    ]) {
        add_source(&mut shared_sources, number, "commit", primary_pr);
    }
    for &pr_number in &landing_prs {
        let (raw, _entry) = ctx.get_pr(pr_number)?;
        for r in raw
            .get("closingIssuesReferences")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(n) = r.get("number").and_then(|v| v.as_u64()) {
                add_source(&mut shared_sources, n, "closing", Some(pr_number));
            }
        }
        let body = raw.get("body").and_then(|v| v.as_str());
        let title = raw.get("title").and_then(|v| v.as_str());
        for number in extract_issue_numbers(&[body, title]) {
            add_source(&mut shared_sources, number, "body", Some(pr_number));
        }
        for c in raw
            .get("commits")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let headline = c.get("messageHeadline").and_then(|v| v.as_str());
            for number in extract_issue_numbers(&[headline]) {
                add_source(&mut shared_sources, number, "commit", Some(pr_number));
            }
        }
        for number in gh_pr_timeline_issue_refs(ctx.root, ctx.cache_dir, pr_number) {
            add_source(&mut shared_sources, number, "timeline", Some(pr_number));
        }
    }

    let mut by_journey: BTreeMap<String, JourneyDiscoveryOut> = BTreeMap::new();
    for journey in group {
        let Some(journey_id) = journey.get("journey").and_then(|v| v.as_str()) else {
            continue;
        };
        let module = journey.get("module").and_then(|v| v.as_str()).unwrap_or("");
        let hints = module_hint_paths(journey_id, module);
        let mut paths =
            resolve_journey_discovery_paths(journey_id, module, &candidate_paths, &hints);
        let mut require_token: Option<&str> = None;
        if paths.is_empty() {
            if let Some(token) = module_s_token(module) {
                // Noisy shared launcher — only keep PRs that mention this binary.
                paths = vec!["core/start-blueos-core".to_string()];
                require_token = Some(token);
            }
        }

        let pickaxe_term = journey_pickaxe_term(journey_id);
        let backport_prs = discover_backport_prs(ctx, &paths, intro_sha)?;
        let follow_up_prs = discover_follow_up_prs(
            ctx,
            &paths,
            intro_sha,
            &landing_prs_set,
            require_token,
            pickaxe_term,
        )?;

        let mut journey_sources = shared_sources.clone();
        for &pr_number in backport_prs.iter().chain(follow_up_prs.iter()) {
            let (raw, _entry) = ctx.get_pr(pr_number)?;
            for r in raw
                .get("closingIssuesReferences")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
            {
                if let Some(n) = r.get("number").and_then(|v| v.as_u64()) {
                    add_source(&mut journey_sources, n, "closing", Some(pr_number));
                }
            }
        }

        let issue_numbers: Vec<u64> = journey_sources.keys().copied().collect();
        for number in issue_numbers {
            ctx.get_issue(number);
        }

        by_journey.insert(
            journey_id.to_string(),
            JourneyDiscoveryOut {
                discovery_paths: paths,
                follow_up_prs,
                backport_prs,
                issues: journey_sources
                    .into_iter()
                    .map(|(number, sources)| ClusterIssueRef { number, sources })
                    .collect(),
            },
        );
    }

    Ok(IntroClusterOut {
        intro_commit: intro_sha.to_string(),
        landing_prs,
        squash_merge,
        merge_method,
        intro_sha_in_pr_commits,
        merge_commit_sha,
        by_journey,
    })
}

fn generated_at_now(root: &Path) -> String {
    shell::run_ok(&["date", "+%Y-%m-%dT%H:%M:%S%:z"], root).unwrap_or_default()
}

const GOLDEN_JOURNEY_IDS: &[&str] = &[
    "InspectZenohNetwork",
    "ChangeUiThemeColor",
    "InspectDiskUsage",
    "RunInternetSpeedTest",
    "LevelHorizon",
];

/// `journeys[].intro_commit` + `intro_clusters[sha]` for one journey: the
/// cluster (shared `landing_prs`) and that journey's own `by_journey` entry.
fn journey_cluster_view<'a>(output: &'a Value, journey_id: &str) -> Option<(&'a Value, &'a Value)> {
    let journeys = output.get("journeys")?.as_array()?;
    let journey = journeys
        .iter()
        .find(|j| j.get("journey").and_then(|v| v.as_str()) == Some(journey_id))?;
    let sha = journey.get("intro_commit")?.as_str()?;
    let cluster = output.get("intro_clusters")?.get(sha)?;
    let by_journey = cluster.get("by_journey")?.get(journey_id)?;
    Some((cluster, by_journey))
}

fn u64_set(value: &Value, field: &str) -> BTreeSet<u64> {
    value
        .get(field)
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(Value::as_u64)
        .collect()
}

fn issue_numbers(by_journey: &Value) -> BTreeSet<u64> {
    by_journey
        .get("issues")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|i| i.get("number").and_then(Value::as_u64))
        .collect()
}

/// Precision-regression gate (`IMPROVE_DESIGN.md` T2 / `PRECISION_QA.md` §1):
/// fails loudly if any of the 5 golden journeys' PR/issue sets drift from the
/// values locked in as the precision baseline.
fn check_strict_goldens(output: &Value) -> Result<(), String> {
    let mut violations: Vec<String> = vec![];

    match journey_cluster_view(output, "InspectZenohNetwork") {
        None => {
            violations.push("InspectZenohNetwork: journey/cluster not found in output".to_string())
        }
        Some((cluster, by_journey)) => {
            let seen: BTreeSet<u64> = u64_set(cluster, "landing_prs")
                .union(&u64_set(by_journey, "follow_up_prs"))
                .copied()
                .collect();
            for pr in [3300, 3313, 3953] {
                if !seen.contains(&pr) {
                    violations.push(format!(
                        "InspectZenohNetwork: golden PR #{pr} missing from landing/follow-up set"
                    ));
                }
            }
        }
    }

    match journey_cluster_view(output, "ChangeUiThemeColor") {
        None => {
            violations.push("ChangeUiThemeColor: journey/cluster not found in output".to_string())
        }
        Some((_cluster, by_journey)) => {
            let follow = u64_set(by_journey, "follow_up_prs");
            let backport = u64_set(by_journey, "backport_prs");
            if !follow.is_empty() {
                violations.push(format!(
                    "ChangeUiThemeColor: expected empty follow_up_prs, got {follow:?}"
                ));
            }
            if !backport.is_empty() {
                violations.push(format!(
                    "ChangeUiThemeColor: expected empty backport_prs, got {backport:?}"
                ));
            }
        }
    }

    match journey_cluster_view(output, "InspectDiskUsage") {
        None => {
            violations.push("InspectDiskUsage: journey/cluster not found in output".to_string())
        }
        Some((_cluster, by_journey)) => {
            let follow = u64_set(by_journey, "follow_up_prs");
            let expected: BTreeSet<u64> = [3681, 3691, 3743].into_iter().collect();
            if follow != expected {
                violations.push(format!(
                    "InspectDiskUsage: expected follow_up_prs == {expected:?}, got {follow:?}"
                ));
            }
        }
    }

    match journey_cluster_view(output, "RunInternetSpeedTest") {
        None => {
            violations.push("RunInternetSpeedTest: journey/cluster not found in output".to_string())
        }
        Some((cluster, by_journey)) => {
            let landing = u64_set(cluster, "landing_prs");
            let follow = u64_set(by_journey, "follow_up_prs");
            let backport = u64_set(by_journey, "backport_prs");
            if !landing.contains(&3602) {
                violations
                    .push("RunInternetSpeedTest: golden landing PR #3602 missing".to_string());
            }
            if follow.contains(&3686) || backport.contains(&3686) {
                violations.push(
                    "RunInternetSpeedTest: PR #3686 must be absent from follow_up/backport"
                        .to_string(),
                );
            }
            if !issue_numbers(by_journey).contains(&2146) {
                violations.push("RunInternetSpeedTest: golden issue #2146 missing".to_string());
            }
        }
    }

    match journey_cluster_view(output, "LevelHorizon") {
        None => violations.push("LevelHorizon: journey/cluster not found in output".to_string()),
        Some((_cluster, by_journey)) => {
            let follow = u64_set(by_journey, "follow_up_prs");
            let backport = u64_set(by_journey, "backport_prs");
            if !backport.contains(&3867) {
                violations.push("LevelHorizon: golden backport PR #3867 missing".to_string());
            }
            if follow.contains(&3930) || backport.contains(&3930) {
                violations.push(
                    "LevelHorizon: PR #3930 must be absent from follow_up/backport".to_string(),
                );
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} golden check(s) failed:\n  - {}",
            violations.len(),
            violations.join("\n  - ")
        ))
    }
}

#[cfg(test)]
mod strict_goldens_tests {
    use super::*;

    fn cluster_fixture(
        journey: &str,
        sha: &str,
        landing_prs: &[u64],
        follow_up_prs: &[u64],
        backport_prs: &[u64],
        issues: &[u64],
    ) -> Value {
        json!({
            "journeys": [{"journey": journey, "intro_commit": sha}],
            "intro_clusters": {
                sha: {
                    "landing_prs": landing_prs,
                    "by_journey": {
                        journey: {
                            "follow_up_prs": follow_up_prs,
                            "backport_prs": backport_prs,
                            "issues": issues.iter().map(|n| json!({"number": n, "sources": []})).collect::<Vec<_>>(),
                        }
                    }
                }
            }
        })
    }

    fn merge(fixtures: Vec<Value>) -> Value {
        let mut journeys = vec![];
        let mut clusters = serde_json::Map::new();
        for f in fixtures {
            journeys.extend(f["journeys"].as_array().unwrap().clone());
            for (sha, cluster) in f["intro_clusters"].as_object().unwrap() {
                clusters.insert(sha.clone(), cluster.clone());
            }
        }
        json!({"journeys": journeys, "intro_clusters": Value::Object(clusters)})
    }

    fn all_golden_fixture() -> Value {
        merge(vec![
            cluster_fixture(
                "InspectZenohNetwork",
                "sha_zenoh",
                &[3300],
                &[3313, 3953, 3386],
                &[],
                &[],
            ),
            cluster_fixture("ChangeUiThemeColor", "sha_theme", &[9001], &[], &[], &[]),
            cluster_fixture(
                "InspectDiskUsage",
                "sha_disk",
                &[9002],
                &[3681, 3691, 3743],
                &[],
                &[],
            ),
            cluster_fixture(
                "RunInternetSpeedTest",
                "sha_speed",
                &[3602],
                &[],
                &[],
                &[2146],
            ),
            cluster_fixture("LevelHorizon", "sha_horizon", &[9003], &[], &[3867], &[]),
        ])
    }

    #[test]
    fn passes_when_all_five_goldens_hold() {
        assert_eq!(check_strict_goldens(&all_golden_fixture()), Ok(()));
    }

    #[test]
    fn fails_when_zenoh_landing_pr_missing() {
        let output = merge(vec![
            cluster_fixture(
                "InspectZenohNetwork",
                "sha_zenoh",
                &[3300],
                &[3953],
                &[],
                &[],
            ),
            cluster_fixture("ChangeUiThemeColor", "sha_theme", &[9001], &[], &[], &[]),
            cluster_fixture(
                "InspectDiskUsage",
                "sha_disk",
                &[9002],
                &[3681, 3691, 3743],
                &[],
                &[],
            ),
            cluster_fixture(
                "RunInternetSpeedTest",
                "sha_speed",
                &[3602],
                &[],
                &[],
                &[2146],
            ),
            cluster_fixture("LevelHorizon", "sha_horizon", &[9003], &[], &[3867], &[]),
        ]);
        let err = check_strict_goldens(&output).unwrap_err();
        assert!(err.contains("InspectZenohNetwork"));
        assert!(err.contains("3313"));
    }

    #[test]
    fn fails_when_ui_theme_has_unexpected_follow_up() {
        let output = merge(vec![
            cluster_fixture(
                "InspectZenohNetwork",
                "sha_zenoh",
                &[3300],
                &[3313, 3953],
                &[],
                &[],
            ),
            cluster_fixture(
                "ChangeUiThemeColor",
                "sha_theme",
                &[9001],
                &[4242],
                &[],
                &[],
            ),
            cluster_fixture(
                "InspectDiskUsage",
                "sha_disk",
                &[9002],
                &[3681, 3691, 3743],
                &[],
                &[],
            ),
            cluster_fixture(
                "RunInternetSpeedTest",
                "sha_speed",
                &[3602],
                &[],
                &[],
                &[2146],
            ),
            cluster_fixture("LevelHorizon", "sha_horizon", &[9003], &[], &[3867], &[]),
        ]);
        let err = check_strict_goldens(&output).unwrap_err();
        assert!(err.contains("ChangeUiThemeColor"));
    }

    #[test]
    fn fails_when_disk_usage_follow_ups_not_exact() {
        let output = merge(vec![
            cluster_fixture(
                "InspectZenohNetwork",
                "sha_zenoh",
                &[3300],
                &[3313, 3953],
                &[],
                &[],
            ),
            cluster_fixture("ChangeUiThemeColor", "sha_theme", &[9001], &[], &[], &[]),
            cluster_fixture(
                "InspectDiskUsage",
                "sha_disk",
                &[9002],
                &[3681, 3691],
                &[],
                &[],
            ),
            cluster_fixture(
                "RunInternetSpeedTest",
                "sha_speed",
                &[3602],
                &[],
                &[],
                &[2146],
            ),
            cluster_fixture("LevelHorizon", "sha_horizon", &[9003], &[], &[3867], &[]),
        ]);
        let err = check_strict_goldens(&output).unwrap_err();
        assert!(err.contains("InspectDiskUsage"));
    }

    #[test]
    fn fails_when_speed_test_has_denied_pr_or_missing_issue() {
        let output = merge(vec![
            cluster_fixture(
                "InspectZenohNetwork",
                "sha_zenoh",
                &[3300],
                &[3313, 3953],
                &[],
                &[],
            ),
            cluster_fixture("ChangeUiThemeColor", "sha_theme", &[9001], &[], &[], &[]),
            cluster_fixture(
                "InspectDiskUsage",
                "sha_disk",
                &[9002],
                &[3681, 3691, 3743],
                &[],
                &[],
            ),
            cluster_fixture(
                "RunInternetSpeedTest",
                "sha_speed",
                &[3602],
                &[3686],
                &[],
                &[],
            ),
            cluster_fixture("LevelHorizon", "sha_horizon", &[9003], &[], &[3867], &[]),
        ]);
        let err = check_strict_goldens(&output).unwrap_err();
        assert!(err.contains("RunInternetSpeedTest"));
        assert!(err.contains("3686"));
        assert!(err.contains("2146"));
    }

    #[test]
    fn fails_when_level_horizon_missing_backport_or_has_denied_pr() {
        let output = merge(vec![
            cluster_fixture(
                "InspectZenohNetwork",
                "sha_zenoh",
                &[3300],
                &[3313, 3953],
                &[],
                &[],
            ),
            cluster_fixture("ChangeUiThemeColor", "sha_theme", &[9001], &[], &[], &[]),
            cluster_fixture(
                "InspectDiskUsage",
                "sha_disk",
                &[9002],
                &[3681, 3691, 3743],
                &[],
                &[],
            ),
            cluster_fixture(
                "RunInternetSpeedTest",
                "sha_speed",
                &[3602],
                &[],
                &[],
                &[2146],
            ),
            cluster_fixture("LevelHorizon", "sha_horizon", &[9003], &[], &[3930], &[]),
        ]);
        let err = check_strict_goldens(&output).unwrap_err();
        assert!(err.contains("LevelHorizon"));
        assert!(err.contains("3867"));
        assert!(err.contains("3930"));
    }

    #[test]
    fn fails_when_journey_missing_from_output_entirely() {
        let output = merge(vec![cluster_fixture(
            "InspectZenohNetwork",
            "sha_zenoh",
            &[3300],
            &[3313, 3953],
            &[],
            &[],
        )]);
        let err = check_strict_goldens(&output).unwrap_err();
        assert!(GOLDEN_JOURNEY_IDS
            .iter()
            .filter(|j| **j != "InspectZenohNetwork")
            .all(|j| err.contains(j)));
    }
}

pub fn run(args: &[String]) -> Result<(), String> {
    let mut journey_filter: Option<String> = None;
    let mut refresh = false;
    let mut only_1_5_exclusive = false;
    let mut strict_goldens = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--journey" => {
                i += 1;
                journey_filter = Some(
                    args.get(i)
                        .ok_or_else(|| "--journey requires a value".to_string())?
                        .clone(),
                );
            }
            "--refresh" => refresh = true,
            "--only-1_5_exclusive" => only_1_5_exclusive = true,
            "--strict-goldens" => strict_goldens = true,
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }

    let root = shell::repo_root();
    let catalog_dir = shell::catalog_dir();
    let presence_path = catalog_dir.join("feature_presence_map.json");
    let out_path = catalog_dir.join("feature_traces.json");
    let cache_dir = catalog_dir.join(".cache/gh_traces");

    if refresh && cache_dir.exists() {
        for entry in fs::read_dir(&cache_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
            }
        }
    }

    if !presence_path.exists() {
        return Err(format!(
            "missing {}; run generate_feature_presence first",
            presence_path.display()
        ));
    }

    let presence_text = fs::read_to_string(&presence_path).map_err(|e| e.to_string())?;
    let presence: Value = serde_json::from_str(&presence_text).map_err(|e| e.to_string())?;
    let mut journeys: Vec<Value> = presence
        .get("journeys")
        .and_then(|j| j.as_array())
        .cloned()
        .unwrap_or_default();

    if let Some(journey) = &journey_filter {
        journeys.retain(|j| j.get("journey").and_then(|v| v.as_str()) == Some(journey.as_str()));
        if journeys.is_empty() {
            return Err(format!("journey not found: {journey}"));
        }
    }

    if only_1_5_exclusive {
        journeys.retain(|j| {
            let present_in_tags: Vec<&str> = j
                .get("present_in_tags")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|t| t.as_str())
                .collect();
            let has_15 = present_in_tags.iter().any(|t| t.starts_with("1.5."));
            let present_on_1_4_dev = j
                .get("present_on_1_4_dev")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let has_14 =
                present_on_1_4_dev || present_in_tags.iter().any(|t| t.starts_with("1.4."));
            has_15 && !has_14
        });
    }

    let mut by_commit: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for j in &journeys {
        let sha = j
            .get("intro_commit")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        by_commit.entry(sha).or_default().push(j.clone());
    }

    println!(
        "Enriching {} unique intro commits across {} journeys…",
        by_commit.len(),
        journeys.len()
    );

    let mut ctx = Ctx::new(&root, &cache_dir);
    let mut clusters: BTreeMap<String, IntroClusterOut> = BTreeMap::new();
    let total = by_commit.len();
    for (i, (sha, group)) in by_commit.iter().enumerate() {
        let names: Vec<String> = group
            .iter()
            .filter_map(|j| j.get("journey").and_then(|v| v.as_str()).map(String::from))
            .collect();
        println!(
            "  [{}/{total}] {} ← {}",
            i + 1,
            &sha[..sha.len().min(12)],
            names.join(", ")
        );
        let cluster = build_intro_cluster(&mut ctx, sha, group)?;
        clusters.insert(sha.clone(), cluster);
    }

    let mut out_journeys = journeys.clone();
    out_journeys.sort_by(|a, b| {
        let ja = a.get("journey").and_then(|v| v.as_str()).unwrap_or("");
        let jb = b.get("journey").and_then(|v| v.as_str()).unwrap_or("");
        ja.cmp(jb)
    });

    let source_presence = presence_path
        .strip_prefix(&root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "catalog/feature_presence_map.json".to_string());

    let output = json!({
        "schema_version": 2,
        "repo": REPO,
        "source_presence": source_presence,
        "tool": "gh + git (cargo run -p blueos-catalog --bin enrich_feature_traces)",
        "generated_at": generated_at_now(&root),
        "commits": ctx.commits,
        "pull_requests": ctx.pull_requests,
        "issues": ctx.issues,
        "intro_clusters": clusters,
        "journeys": out_journeys,
    });

    let text = serde_json::to_string_pretty(&output).map_err(|e| e.to_string())? + "\n";
    fs::write(&out_path, &text).map_err(|e| e.to_string())?;

    let size = fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
    let out_rel = out_path
        .strip_prefix(&root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "catalog/feature_traces.json".to_string());
    println!("Wrote {out_rel} ({size} bytes)");

    if strict_goldens {
        // A --journey filter narrows the run to one journey; only gate on the
        // golden(s) that journey actually is, so an unrelated run doesn't fail
        // on goldens it never touched.
        let relevant: Vec<&str> = match &journey_filter {
            Some(j) => GOLDEN_JOURNEY_IDS
                .iter()
                .filter(|g| *g == j)
                .copied()
                .collect(),
            None => GOLDEN_JOURNEY_IDS.to_vec(),
        };
        if !relevant.is_empty() {
            let scoped = json!({
                "journeys": out_journeys
                    .iter()
                    .filter(|j| relevant.contains(&j.get("journey").and_then(|v| v.as_str()).unwrap_or("")))
                    .cloned()
                    .collect::<Vec<_>>(),
                "intro_clusters": output.get("intro_clusters").cloned().unwrap_or(json!({})),
            });
            check_strict_goldens(&scoped).map_err(|e| format!("--strict-goldens: {e}"))?;
            println!(
                "--strict-goldens: {} golden journey check(s) passed",
                relevant.len()
            );
        }
    }

    Ok(())
}
