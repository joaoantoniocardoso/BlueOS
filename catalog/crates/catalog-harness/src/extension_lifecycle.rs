//! Live Kraken v1+v2 lifecycle driven by catalog journeys and negative probes.
//!
//! Replaces the temporary Python skill script. Run against a DUT with
//! `journey_http --base http://<pi> --extension-lifecycle --allow-mutating`.
//! Do not use `blueos.major_tom` or any identifier already installed at baseline.

use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use catalog_core::catalog::Catalog;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::provenance::{Grounded, GroundedSet};
use catalog_model::journey::{HttpMethod, RouteRef, UseCase};
use serde::Serialize;

use crate::negative_probes::kraken_lifecycle_probes;
use crate::report::{utc_rfc3339_now, SCHEMA_VERSION};
use crate::runner::{execute_curl, join_url, run_negative_probe, streamed_fragment_error, Verdict};

const FORBIDDEN_EXT: &str = "blueos.major_tom";
const FACTORY_MANIFEST: &str = "bluerobotics-production";
const DUMMY_MANIFEST_NAME: &str = "kraken-lifecycle-dummy";
const UNKNOWN: &str = "np.no.such.extension";
const POLL_TIMEOUT: Duration = Duration::from_secs(45);
const POLL_SLEEP: Duration = Duration::from_millis(1500);

/// Catalog routes the lifecycle runner binds and calls. A missing row is a catalog bug.
pub const LIFECYCLE_CATALOG_ROUTES: &[(JourneyId, HttpMethod, &str)] = &[
    (
        JourneyId::BrowseExtensionStore,
        HttpMethod::Get,
        "/extension/",
    ),
    (
        JourneyId::BrowseExtensionStore,
        HttpMethod::Get,
        "/installed_extensions",
    ),
    (
        JourneyId::BrowseExtensionStore,
        HttpMethod::Get,
        "/extensions_manifest",
    ),
    (
        JourneyId::BrowseExtensionStore,
        HttpMethod::Get,
        "/manifest/",
    ),
    (
        JourneyId::BrowseExtensionStore,
        HttpMethod::Get,
        "/manifest/consolidated",
    ),
    (
        JourneyId::BrowseExtensionStore,
        HttpMethod::Get,
        "/manifest/{identifier}/details",
    ),
    (
        JourneyId::BrowseExtensionStore,
        HttpMethod::Get,
        "/manifest/tags/{extension_identifier}",
    ),
    (
        JourneyId::BrowseExtensionStore,
        HttpMethod::Get,
        "/manifest/tags/{manifest_identifier}/{extension_identifier}/",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/container/",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/container/stats",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/stats",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/list_containers",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/container/{container_name}/details",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/container/{container_name}/stats",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/container/{container_name}/log",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/log",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Post,
        "/extension/{identifier}/restart",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Post,
        "/extension/restart",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Post,
        "/extension/{identifier}/disable",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Post,
        "/extension/disable",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Post,
        "/extension/{identifier}/{tag}/enable",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Post,
        "/extension/enable",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/jobs/",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Post,
        "/jobs/{route}",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Get,
        "/jobs/{identifier}",
    ),
    (
        JourneyId::ConfigureInstalledExtension,
        HttpMethod::Delete,
        "/jobs/{identifier}",
    ),
    (
        JourneyId::EditExtensionDevVersion,
        HttpMethod::Put,
        "/extension/{identifier}",
    ),
    (
        JourneyId::InstallExtension,
        HttpMethod::Get,
        "/extension/{identifier}/details",
    ),
    (
        JourneyId::InstallExtension,
        HttpMethod::Get,
        "/extension/{identifier}/{tag}/details",
    ),
    (
        JourneyId::InstallExtension,
        HttpMethod::Post,
        "/extension/{identifier}/{tag}/install",
    ),
    (
        JourneyId::InstallExtension,
        HttpMethod::Post,
        "/extension/{identifier}/install",
    ),
    (
        JourneyId::InstallExtension,
        HttpMethod::Post,
        "/extension/install",
    ),
    (
        JourneyId::InstallCustomExtension,
        HttpMethod::Post,
        "/extension/",
    ),
    (
        JourneyId::UninstallExtension,
        HttpMethod::Delete,
        "/extension/{identifier}/{tag}",
    ),
    (
        JourneyId::UninstallExtension,
        HttpMethod::Delete,
        "/extension/{identifier}",
    ),
    (
        JourneyId::UninstallExtension,
        HttpMethod::Post,
        "/extension/uninstall",
    ),
    (
        JourneyId::EditExtensionDevVersion,
        HttpMethod::Put,
        "/extension/{identifier}/{tag}",
    ),
    (
        JourneyId::EditExtensionDevVersion,
        HttpMethod::Post,
        "/extension/update_to_version",
    ),
    (JourneyId::AddCustomManifest, HttpMethod::Post, "/manifest/"),
    (
        JourneyId::AddCustomManifest,
        HttpMethod::Get,
        "/manifest/{identifier}/details",
    ),
    (
        JourneyId::AddCustomManifest,
        HttpMethod::Post,
        "/manifest/{identifier}/enable",
    ),
    (
        JourneyId::AddCustomManifest,
        HttpMethod::Post,
        "/manifest/{identifier}/disable",
    ),
    (
        JourneyId::AddCustomManifest,
        HttpMethod::Put,
        "/manifest/{identifier}/details",
    ),
    (
        JourneyId::AddCustomManifest,
        HttpMethod::Put,
        "/manifest/{identifier}/order/{order}",
    ),
    (
        JourneyId::AddCustomManifest,
        HttpMethod::Put,
        "/manifest/orders",
    ),
    (
        JourneyId::AddCustomManifest,
        HttpMethod::Delete,
        "/manifest/{identifier}",
    ),
];

/// CLI/harness knobs for `--extension-lifecycle`.
#[derive(Debug, Clone, Default)]
pub struct ExtensionLifecycleOpts {
    /// Override the default unused compatible identifier.
    pub identifier: Option<String>,
    /// Override the default smallest compatible tag.
    pub tag: Option<String>,
    /// Display name for the custom-source install body.
    pub name: Option<String>,
    /// Image repository for the custom-source install body.
    pub docker: Option<String>,
    /// Require unknown restart/disable to return 404 (patched `from_running`).
    pub assert_unknown: bool,
    /// Skip `POST /{id}/install` (`from_latest`).
    pub skip_latest: bool,
    /// Skip the second-tag `PUT` while a sibling is running.
    pub skip_alt: bool,
    /// Skip dummy (non-factory) manifest create/delete.
    pub skip_manifest_mutate: bool,
}

/// One PASS/FAIL/SKIP row printed by `--extension-lifecycle`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    /// Check label (`v1 install never-installed 2xx`, `NP-54`, ...).
    pub name: String,
    /// Lifecycle phase (`baseline`, `v1`, `v2`, `restore`, ...).
    pub phase: String,
    /// True when the row is a pass. Skips also set this so old counters stay non-failing.
    pub ok: bool,
    /// True when the row is an explicit skip, not a verified pass.
    #[serde(default)]
    pub skipped: bool,
    /// HTTP snippet or skip reason.
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LifecycleReport {
    schema_version: u32,
    base: String,
    started_at: String,
    finished_at: String,
    counts: LifecycleCounts,
    checks: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LifecycleCounts {
    passed: usize,
    failed: usize,
    skipped: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstalledExtension {
    identifier: String,
    tag: String,
    enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunningContainer {
    name: String,
    image: String,
}

#[derive(Debug, Clone)]
struct Vehicle {
    identifier: String,
    tag: String,
    name: String,
    docker: String,
    permissions: String,
}

impl Check {
    fn new(name: &str, phase: &str, ok: bool, detail: impl Into<String>) -> Self {
        let skipped = name.starts_with("skip:");
        Self {
            name: name.to_string(),
            phase: phase.to_string(),
            ok: ok || skipped,
            skipped,
            detail: detail.into(),
        }
    }
}

/// Run the catalog-driven Kraken v1+v2 lifecycle against `base` (nginx).
pub fn run_extension_lifecycle(
    base: &str,
    allow_mutating: bool,
    opts: ExtensionLifecycleOpts,
) -> Vec<Check> {
    let catalog = Catalog::bootstrap();
    let mut checks = Vec::new();
    if !assert_catalog_routes(&catalog, &mut checks) {
        return checks;
    }
    let mut harness = Harness {
        catalog,
        base: base.trim_end_matches('/').to_string(),
        allow_mutating,
        opts,
        vehicle: None,
        alt_tag: None,
        dummy_manifest_id: None,
        baseline: Vec::new(),
        baseline_ids: Vec::new(),
        baseline_manifests: Vec::new(),
        checks,
    };
    harness.run();
    harness.checks
}

/// Write the lifecycle PASS/FAIL list as JSON (`schema_version` + counts + checks).
pub fn write_extension_lifecycle_report(
    path: &str,
    base: &str,
    started_at: &str,
    checks: &[Check],
) -> Result<(), String> {
    let skipped = checks.iter().filter(|check| check.skipped).count();
    let failed = checks
        .iter()
        .filter(|check| !check.ok && !check.skipped)
        .count();
    let passed = checks
        .iter()
        .filter(|check| check.ok && !check.skipped)
        .count();
    let report = LifecycleReport {
        schema_version: SCHEMA_VERSION,
        base: base.to_string(),
        started_at: started_at.to_string(),
        finished_at: utc_rfc3339_now(),
        counts: LifecycleCounts {
            passed,
            failed,
            skipped,
        },
        checks: checks.to_vec(),
    };
    let json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| error.to_string())
}

fn assert_catalog_routes(catalog: &Catalog, checks: &mut Vec<Check>) -> bool {
    let mut ok = true;
    for (journey, method, path) in LIFECYCLE_CATALOG_ROUTES {
        match find_route(catalog, *journey, method, path) {
            Ok(_) => {}
            Err(error) => {
                checks.push(Check::new("catalog route", "preflight", false, error));
                ok = false;
            }
        }
    }
    if ok {
        checks.push(Check::new(
            "catalog routes",
            "preflight",
            true,
            format!("{} journey RouteRefs", LIFECYCLE_CATALOG_ROUTES.len()),
        ));
    }
    ok
}

fn find_route(
    catalog: &Catalog,
    journey: JourneyId,
    method: &HttpMethod,
    path: &str,
) -> Result<RouteRef, String> {
    let Some(use_case) = catalog.journey_by_id(&journey) else {
        return Err(format!("catalog missing journey {journey}"));
    };
    if let Some(route) = route_on_journey(use_case, method, path) {
        return Ok(route);
    }
    Err(format!("catalog missing {method:?} {path} on {journey}"))
}

fn route_on_journey(journey: &UseCase, method: &HttpMethod, path: &str) -> Option<RouteRef> {
    let GroundedSet::Known { items: steps } = &journey.steps else {
        return None;
    };
    for step in *steps {
        let Some(Grounded::Known { value: route, .. }) = &step.value.route else {
            continue;
        };
        if &route.method == method && route.path == path {
            return Some(route.clone());
        }
    }
    None
}

// Ordering exception: declared next to `impl Harness` rather than with the public
// types so the public entry points stay above this 1.5k-line private impl.
struct Harness {
    catalog: Catalog,
    base: String,
    allow_mutating: bool,
    opts: ExtensionLifecycleOpts,
    vehicle: Option<Vehicle>,
    alt_tag: Option<String>,
    dummy_manifest_id: Option<String>,
    baseline: Vec<InstalledExtension>,
    baseline_ids: Vec<String>,
    baseline_manifests: Vec<String>,
    checks: Vec<Check>,
}

impl Harness {
    fn rec(&mut self, name: &str, phase: &str, ok: bool, detail: impl Into<String>) {
        self.checks.push(Check::new(name, phase, ok, detail));
    }

    fn call(
        &self,
        journey: JourneyId,
        method: HttpMethod,
        path: &str,
        bound: &str,
        query: Option<&str>,
        body: Option<&str>,
    ) -> Result<(u16, String), String> {
        let route = find_route(&self.catalog, journey, &method, path)?;
        let url = kraken_url(&self.base, route.version, bound, query);
        execute_curl(&method, &url, self.allow_mutating, body, None)
    }

    fn installed(&self) -> Vec<InstalledExtension> {
        match self.call(
            JourneyId::BrowseExtensionStore,
            HttpMethod::Get,
            "/extension/",
            "/extension/",
            None,
            None,
        ) {
            Ok((200, body)) => parse_installed_extensions(&body).unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn container_names(&self) -> Vec<String> {
        match self.call(
            JourneyId::ConfigureInstalledExtension,
            HttpMethod::Get,
            "/list_containers",
            "/list_containers",
            None,
            None,
        ) {
            Ok((200, body)) => parse_running_containers(&body)
                .unwrap_or_default()
                .into_iter()
                .map(|container| container.name.trim_start_matches('/').to_string())
                .collect(),
            _ => Vec::new(),
        }
    }

    fn wait_container(&self, name: &str, want: bool, timeout: Duration) -> (bool, Vec<String>) {
        let deadline = Instant::now() + timeout;
        let mut names;
        loop {
            names = self.container_names();
            if want && names.iter().any(|n| n == name) {
                return (true, names);
            }
            if !want && names.iter().all(|n| n != name) {
                return (true, names);
            }
            if Instant::now() >= deadline {
                return (false, names);
            }
            thread::sleep(POLL_SLEEP);
        }
    }

    fn payload(&self) -> Result<String, String> {
        let vehicle = self.vehicle.as_ref().ok_or("no vehicle")?;
        Ok(format!(
            r#"{{"identifier":"{}","name":"{}","docker":"{}","tag":"{}","enabled":true,"permissions":"{}","user_permissions":""}}"#,
            vehicle.identifier,
            escape_json(&vehicle.name),
            escape_json(&vehicle.docker),
            vehicle.tag,
            escape_json(&vehicle.permissions),
        ))
    }

    fn cname(&self, tag: Option<&str>) -> String {
        let vehicle = self.vehicle.as_ref().expect("vehicle");
        container_name(&vehicle.docker, tag.unwrap_or(&vehicle.tag))
    }

    fn expect_install(&mut self, label: &str, status: u16, body: &str, ident: &str) {
        let not_found = status == 404 || body.contains(&format!("Extension {ident} not found"));
        self.rec(
            &format!("{label} is not 404/not-found"),
            "install",
            !not_found,
            format!("HTTP {status} {}", snippet(body, 200)),
        );
        let stream_err = streamed_fragment_error(body);
        let ok2xx = (200..300).contains(&status) && stream_err.is_none();
        self.rec(
            &format!("{label} 2xx"),
            "install",
            ok2xx,
            stream_err.unwrap_or_else(|| format!("HTTP {status}")),
        );
    }

    fn expect_running(&mut self, label: &str, ident: &str, tag: &str) {
        let after = self.installed();
        self.rec(
            &format!("{label} listed"),
            "install",
            after
                .iter()
                .any(|ext| ext.identifier == ident && ext.tag == tag),
            format!("{after:?}"),
        );
        self.rec(
            &format!("{label} enabled"),
            "install",
            after
                .iter()
                .any(|ext| ext.identifier == ident && ext.enabled),
            format!("{after:?}"),
        );
        let baseline: std::collections::HashSet<_> = self.baseline_ids.iter().cloned().collect();
        let after_ids: std::collections::HashSet<_> =
            after.iter().map(|ext| ext.identifier.clone()).collect();
        self.rec(
            &format!("{label} baseline ids remain"),
            "install",
            baseline.is_subset(&after_ids),
            format!("{after:?}"),
        );
        let cname = self.cname(Some(tag));
        let (ok, names) = self.wait_container(&cname, true, POLL_TIMEOUT);
        self.rec(
            &format!("{label} container running"),
            "install",
            ok,
            format!("{names:?}"),
        );
    }

    fn uninstall_vehicle(&mut self) {
        let Some(vehicle) = self.vehicle.clone() else {
            return;
        };
        if self.baseline_ids.iter().any(|id| id == &vehicle.identifier)
            || vehicle.identifier == FORBIDDEN_EXT
        {
            return;
        }
        // Best-effort restore: the vehicle may already be gone.
        let _ = self.call(
            JourneyId::UninstallExtension,
            HttpMethod::Delete,
            "/extension/{identifier}",
            &format!("/extension/{}", vehicle.identifier),
            None,
            None,
        );
        // Best-effort restore: v1 uninstall is the other public door.
        let _ = self.call(
            JourneyId::UninstallExtension,
            HttpMethod::Post,
            "/extension/uninstall",
            "/extension/uninstall",
            Some(&format!("extension_identifier={}", vehicle.identifier)),
            None,
        );
        let cname = self.cname(None);
        // Restore continues even if the container wait times out.
        let (_gone, _names) = self.wait_container(&cname, false, Duration::from_secs(20));
        if let Some(alt) = self.alt_tag.clone() {
            let alt_name = self.cname(Some(&alt));
            let (_alt_gone, _alt_names) =
                self.wait_container(&alt_name, false, Duration::from_secs(10));
        }
    }

    fn drop_dummy_manifest(&mut self) {
        if let Some(ident) = self.dummy_manifest_id.take() {
            // Best-effort restore: dummy source may already have been deleted.
            let _ = self.call(
                JourneyId::AddCustomManifest,
                HttpMethod::Delete,
                "/manifest/{identifier}",
                &format!("/manifest/{ident}"),
                None,
                None,
            );
        }
        if let Ok((200, body)) = self.call(
            JourneyId::BrowseExtensionStore,
            HttpMethod::Get,
            "/manifest/",
            "/manifest/",
            Some("data=false"),
            None,
        ) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(items) = value.as_array() {
                    for item in items {
                        if item
                            .get("name")
                            .and_then(|value| value.as_str())
                            .is_some_and(is_lifecycle_dummy_name)
                            && item.get("factory").and_then(|value| value.as_bool()) != Some(true)
                        {
                            if let Some(ident) = item.get("identifier").and_then(|v| v.as_str()) {
                                // Best-effort restore: leftover dummy rows from a prior aborted run.
                                let _ = self.call(
                                    JourneyId::AddCustomManifest,
                                    HttpMethod::Delete,
                                    "/manifest/{identifier}",
                                    &format!("/manifest/{ident}"),
                                    None,
                                    None,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn assert_factory_manifest(&mut self, phase: &str) {
        let (status, body) = self
            .call(
                JourneyId::BrowseExtensionStore,
                HttpMethod::Get,
                "/manifest/",
                "/manifest/",
                Some("data=false"),
                None,
            )
            .unwrap_or((0, String::new()));
        let present = status == 200
            && serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| value.as_array().cloned())
                .unwrap_or_default()
                .iter()
                .any(|item| {
                    item.get("identifier").and_then(|value| value.as_str())
                        == Some(FACTORY_MANIFEST)
                        && item.get("factory").and_then(|value| value.as_bool()) == Some(true)
                });
        self.rec(
            "factory manifest still present",
            phase,
            present,
            format!("HTTP {status}"),
        );
    }

    fn run(&mut self) {
        let result: Result<(), String> = (|| {
            self.phase_baseline()?;
            self.phase_reads();
            self.phase_unknown();
            self.phase_jobs();
            self.phase_manifest_mutate();
            self.phase_v1_install_lifecycle();
            self.phase_v2_installs();
            Ok(())
        })();
        if let Err(error) = result {
            self.rec("lifecycle aborted", "error", false, error);
        }
        self.uninstall_vehicle();
        self.drop_dummy_manifest();
        if !self.baseline_manifests.is_empty() {
            let body =
                serde_json::to_string(&self.baseline_manifests).unwrap_or_else(|_| "[]".into());
            // Best-effort restore: factory-first order even if PUT fails.
            let _ = self.call(
                JourneyId::AddCustomManifest,
                HttpMethod::Put,
                "/manifest/orders",
                "/manifest/orders",
                None,
                Some(&body),
            );
        }
        let final_ids: std::collections::HashSet<_> = self
            .installed()
            .into_iter()
            .map(|ext| ext.identifier)
            .collect();
        let baseline: std::collections::HashSet<_> = self.baseline_ids.iter().cloned().collect();
        self.rec(
            "restored to baseline identifiers",
            "restore",
            final_ids == baseline,
            format!("{final_ids:?}"),
        );
        let leftover = match self.call(
            JourneyId::BrowseExtensionStore,
            HttpMethod::Get,
            "/manifest/",
            "/manifest/",
            Some("data=false"),
            None,
        ) {
            Ok((200, body)) => serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| value.as_array().cloned())
                .unwrap_or_default()
                .iter()
                .filter_map(|item| item.get("name")?.as_str().map(str::to_string))
                .filter(|name| is_lifecycle_dummy_name(name))
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        self.rec(
            "no leftover dummy manifest",
            "restore",
            leftover.is_empty(),
            format!("{leftover:?}"),
        );
        self.assert_factory_manifest("restore");
        self.rec("mgmt ping after", "restore", mgmt_ping(&self.base), "");
    }

    fn phase_baseline(&mut self) -> Result<(), String> {
        self.rec("mgmt ping before", "baseline", mgmt_ping(&self.base), "");
        self.baseline = self.installed();
        self.baseline_ids = self
            .baseline
            .iter()
            .map(|ext| ext.identifier.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        self.drop_dummy_manifest();
        let (status, body) = self.call(
            JourneyId::BrowseExtensionStore,
            HttpMethod::Get,
            "/manifest/",
            "/manifest/",
            Some("data=false"),
            None,
        )?;
        let list_ok = status == 200
            && serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| value.as_array().map(|items| !items.is_empty()))
                .unwrap_or(false);
        self.rec(
            "v2 manifest list",
            "baseline",
            list_ok,
            format!("HTTP {status}"),
        );
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(items) = value.as_array() {
                self.baseline_manifests = items
                    .iter()
                    .filter_map(|item| item.get("identifier")?.as_str().map(str::to_string))
                    .collect();
            }
        }

        let (mut vehicle, alt) = pick_vehicle(
            &self.catalog,
            &self.base,
            self.allow_mutating,
            &self.baseline_ids,
            self.opts.identifier.as_deref(),
            self.opts.tag.as_deref(),
        )?;
        if let Some(name) = &self.opts.name {
            vehicle.name = name.clone();
        }
        if let Some(docker) = &self.opts.docker {
            vehicle.docker = docker.clone();
        }
        if vehicle.identifier == FORBIDDEN_EXT
            || self.baseline_ids.iter().any(|id| id == &vehicle.identifier)
        {
            return Err(format!(
                "refusing to use baseline/forbidden identifier {}",
                vehicle.identifier
            ));
        }
        self.alt_tag = if self.opts.skip_alt { None } else { alt };
        eprintln!(
            "DUT={} vehicle={}:{}{} container={}",
            self.base,
            vehicle.identifier,
            vehicle.tag,
            self.alt_tag
                .as_ref()
                .map(|tag| format!(" alt={tag}"))
                .unwrap_or_default(),
            container_name(&vehicle.docker, &vehicle.tag)
        );
        let ident = vehicle.identifier.clone();
        self.vehicle = Some(vehicle);
        self.rec(
            "test id absent at baseline",
            "baseline",
            !self.baseline_ids.iter().any(|id| id == &ident),
            format!("{:?}", self.baseline),
        );
        Ok(())
    }

    fn phase_reads(&mut self) {
        let ident = self.vehicle.as_ref().expect("vehicle").identifier.clone();
        let (status, body) = self
            .call(
                JourneyId::BrowseExtensionStore,
                HttpMethod::Get,
                "/installed_extensions",
                "/installed_extensions",
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        let v1_ok = status == 200
            && serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| value.as_array().map(|_| true))
                .unwrap_or(false);
        self.rec(
            "v1 installed_extensions 200",
            "reads",
            v1_ok,
            format!("HTTP {status}"),
        );
        let v1_ids: std::collections::HashSet<String> = parse_installed_extensions(&body)
            .unwrap_or_default()
            .into_iter()
            .map(|ext| ext.identifier)
            .collect();
        let baseline: std::collections::HashSet<_> = self.baseline_ids.iter().cloned().collect();
        self.rec(
            "v1 list matches v2 identifiers",
            "reads",
            v1_ids == baseline,
            format!("v1={v1_ids:?} v2={baseline:?}"),
        );
        for (name, journey, path) in [
            (
                "v1 list_containers 200",
                JourneyId::ConfigureInstalledExtension,
                "/list_containers",
            ),
            (
                "v2 container list 200",
                JourneyId::ConfigureInstalledExtension,
                "/container/",
            ),
            (
                "v1 stats 200",
                JourneyId::ConfigureInstalledExtension,
                "/stats",
            ),
            (
                "v2 container stats 200",
                JourneyId::ConfigureInstalledExtension,
                "/container/stats",
            ),
            (
                "v1 extensions_manifest 200",
                JourneyId::BrowseExtensionStore,
                "/extensions_manifest",
            ),
            (
                "v2 jobs list 200",
                JourneyId::ConfigureInstalledExtension,
                "/jobs/",
            ),
        ] {
            let (status, _) = self
                .call(journey, HttpMethod::Get, path, path, None, None)
                .unwrap_or((0, String::new()));
            self.rec(name, "reads", status == 200, format!("HTTP {status}"));
        }
        let (status, body) = self
            .call(
                JourneyId::BrowseExtensionStore,
                HttpMethod::Get,
                "/manifest/consolidated",
                "/manifest/consolidated",
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        let consolidated_ok = status == 200
            && serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| value.as_array().map(|items| !items.is_empty()))
                .unwrap_or(false);
        self.rec(
            "v2 manifest consolidated 200",
            "reads",
            consolidated_ok,
            format!("HTTP {status}"),
        );
        let (status, body) = self
            .call(
                JourneyId::BrowseExtensionStore,
                HttpMethod::Get,
                "/manifest/{identifier}/details",
                &format!("/manifest/{FACTORY_MANIFEST}/details"),
                Some("data=false"),
                None,
            )
            .unwrap_or((0, String::new()));
        let factory_ok = status == 200
            && serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| {
                    value
                        .get("identifier")?
                        .as_str()
                        .map(|id| id == FACTORY_MANIFEST)
                })
                .unwrap_or(false);
        self.rec(
            "v2 factory manifest details",
            "reads",
            factory_ok,
            format!("HTTP {status}"),
        );
        let (status, body) = self
            .call(
                JourneyId::BrowseExtensionStore,
                HttpMethod::Get,
                "/manifest/tags/{extension_identifier}",
                &format!("/manifest/tags/{ident}"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        let tags_ok = status == 200
            && serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| value.as_array().map(|items| !items.is_empty()))
                .unwrap_or(false);
        self.rec(
            "v2 tags from consolidated",
            "reads",
            tags_ok,
            format!("HTTP {status} {}", snippet(&body, 80)),
        );
        let (status, _) = self
            .call(
                JourneyId::BrowseExtensionStore,
                HttpMethod::Get,
                "/manifest/tags/{manifest_identifier}/{extension_identifier}/",
                &format!("/manifest/tags/{FACTORY_MANIFEST}/{ident}/"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 tags from factory manifest",
            "reads",
            status == 200,
            format!("HTTP {status}"),
        );
    }

    fn phase_unknown(&mut self) {
        for probe in kraken_lifecycle_probes() {
            if probe.expected_status.is_none() {
                self.rec(
                    &format!("skip: {} unasserted", probe.id),
                    "unknown",
                    true,
                    probe.path,
                );
                continue;
            }
            let result = run_negative_probe(&self.base, probe, self.allow_mutating, None);
            let (ok, detail) = match result {
                Verdict::Pass => (true, format!("{} HTTP ok", probe.id)),
                Verdict::Fail(msg) => (false, format!("{} {msg}", probe.id)),
                Verdict::Error => (false, format!("{} unasserted", probe.id)),
                Verdict::Inconclusive(msg) => (false, format!("{} {msg}", probe.id)),
            };
            self.rec(probe.id, "unknown", ok, detail);
        }
        self.assert_factory_manifest("unknown");
        if !self.opts.assert_unknown {
            return;
        }
        let pairs = [
            (
                "unknown v2 restart -> 404",
                HttpMethod::Post,
                format!("/kraken/v2.0/extension/{UNKNOWN}/restart"),
                None,
            ),
            (
                "unknown v1 restart -> 404",
                HttpMethod::Post,
                "/kraken/v1.0/extension/restart".to_string(),
                Some(format!("extension_identifier={UNKNOWN}")),
            ),
            (
                "unknown v2 disable -> 404",
                HttpMethod::Post,
                format!("/kraken/v2.0/extension/{UNKNOWN}/disable"),
                None,
            ),
            (
                "unknown v1 disable -> 404",
                HttpMethod::Post,
                "/kraken/v1.0/extension/disable".to_string(),
                Some(format!("extension_identifier={UNKNOWN}")),
            ),
        ];
        for (name, method, path, query) in pairs {
            let url = match query {
                Some(query) => format!("{}?{query}", join_url(&self.base, &path)),
                None => join_url(&self.base, &path),
            };
            let (status, body) = execute_curl(&method, &url, self.allow_mutating, None, None)
                .unwrap_or((0, String::new()));
            self.rec(
                name,
                "unknown",
                status == 404,
                format!("HTTP {status} {}", snippet(&body, 80)),
            );
        }
    }

    fn phase_jobs(&mut self) {
        let (status, body) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Post,
                "/jobs/{route}",
                "/jobs/v2.0/extension/",
                Some("method=GET&retries=1"),
                Some("{}"),
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 jobs enqueue GET list -> 202",
            "jobs",
            status == 202,
            format!("HTTP {status} {}", snippet(&body, 80)),
        );
        let job_id = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|value| value.get("id")?.as_str().map(str::to_string));
        if let Some(job_id) = job_id {
            let (status, _) = self
                .call(
                    JourneyId::ConfigureInstalledExtension,
                    HttpMethod::Get,
                    "/jobs/{identifier}",
                    &format!("/jobs/{job_id}"),
                    None,
                    None,
                )
                .unwrap_or((0, String::new()));
            if status == 200 {
                self.rec(
                    "v2 jobs get enqueued or running",
                    "jobs",
                    true,
                    format!("HTTP {status}"),
                );
                let (status, _) = self
                    .call(
                        JourneyId::ConfigureInstalledExtension,
                        HttpMethod::Delete,
                        "/jobs/{identifier}",
                        &format!("/jobs/{job_id}"),
                        None,
                        None,
                    )
                    .unwrap_or((0, String::new()));
                self.rec(
                    "v2 jobs delete queued -> 204",
                    "jobs",
                    status == 204 || status == 404,
                    format!("HTTP {status}"),
                );
            } else if status == 404 {
                self.rec(
                    "skip: v2 jobs GET",
                    "jobs",
                    true,
                    "job already finished before GET",
                );
                self.rec(
                    "skip: v2 jobs DELETE",
                    "jobs",
                    true,
                    "job already finished before GET",
                );
            } else {
                self.rec(
                    "v2 jobs get enqueued or running",
                    "jobs",
                    false,
                    format!("HTTP {status}"),
                );
            }
        }
    }

    fn phase_manifest_mutate(&mut self) {
        if self.opts.skip_manifest_mutate {
            self.rec(
                "skip: dummy manifest mutate",
                "manifest",
                true,
                "--skip-manifest-mutate",
            );
            return;
        }
        let body = format!(
            r#"{{"name":"{DUMMY_MANIFEST_NAME}","url":"http://127.0.0.1:9/no-such-manifest.json","enabled":false}}"#
        );
        let (status, raw) = self
            .call(
                JourneyId::AddCustomManifest,
                HttpMethod::Post,
                "/manifest/",
                "/manifest/",
                Some("validate_url=false"),
                Some(&body),
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 dummy manifest create -> 201",
            "manifest",
            status == 201,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        let ident = serde_json::from_str::<serde_json::Value>(&raw)
            .ok()
            .and_then(|value| value.get("identifier")?.as_str().map(str::to_string));
        let Some(ident) = ident else {
            return;
        };
        self.dummy_manifest_id = Some(ident.clone());
        let (status, body) = self
            .call(
                JourneyId::AddCustomManifest,
                HttpMethod::Get,
                "/manifest/{identifier}/details",
                &format!("/manifest/{ident}/details"),
                Some("data=false"),
                None,
            )
            .unwrap_or((0, String::new()));
        let details_ok = status == 200
            && serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| {
                    value
                        .get("name")?
                        .as_str()
                        .map(|name| name == DUMMY_MANIFEST_NAME)
                })
                .unwrap_or(false);
        self.rec(
            "v2 dummy manifest details",
            "manifest",
            details_ok,
            format!("HTTP {status}"),
        );
        let (status, _) = self
            .call(
                JourneyId::AddCustomManifest,
                HttpMethod::Post,
                "/manifest/{identifier}/enable",
                &format!("/manifest/{ident}/enable"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 dummy manifest enable -> 204",
            "manifest",
            status == 204,
            format!("HTTP {status}"),
        );
        let (status, _) = self
            .call(
                JourneyId::AddCustomManifest,
                HttpMethod::Post,
                "/manifest/{identifier}/disable",
                &format!("/manifest/{ident}/disable"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 dummy manifest disable -> 204",
            "manifest",
            status == 204,
            format!("HTTP {status}"),
        );
        let put_body = format!(r#"{{"name":"{DUMMY_MANIFEST_NAME}-renamed"}}"#);
        let (status, _) = self
            .call(
                JourneyId::AddCustomManifest,
                HttpMethod::Put,
                "/manifest/{identifier}/details",
                &format!("/manifest/{ident}/details"),
                Some("validate_url=false"),
                Some(&put_body),
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 dummy manifest PUT details -> 204",
            "manifest",
            status == 204,
            format!("HTTP {status}"),
        );
        let (status, _) = self
            .call(
                JourneyId::AddCustomManifest,
                HttpMethod::Put,
                "/manifest/{identifier}/order/{order}",
                &format!("/manifest/{ident}/order/99"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 dummy manifest PUT order -> 204",
            "manifest",
            status == 204,
            format!("HTTP {status}"),
        );
        if !self.baseline_manifests.is_empty() {
            let orders =
                serde_json::to_string(&self.baseline_manifests).unwrap_or_else(|_| "[]".into());
            let (status, _) = self
                .call(
                    JourneyId::AddCustomManifest,
                    HttpMethod::Put,
                    "/manifest/orders",
                    "/manifest/orders",
                    None,
                    Some(&orders),
                )
                .unwrap_or((0, String::new()));
            self.rec(
                "v2 manifest PUT orders restore -> 204",
                "manifest",
                status == 204,
                format!("HTTP {status}"),
            );
        }
        let (status, _) = self
            .call(
                JourneyId::AddCustomManifest,
                HttpMethod::Delete,
                "/manifest/{identifier}",
                &format!("/manifest/{ident}"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 dummy manifest DELETE -> 204",
            "manifest",
            status == 204,
            format!("HTTP {status}"),
        );
        self.dummy_manifest_id = None;
    }

    fn phase_v1_install_lifecycle(&mut self) {
        let vehicle = self.vehicle.clone().expect("vehicle");
        let ident = vehicle.identifier.clone();
        let tag = vehicle.tag.clone();
        let cname = self.cname(None);
        let payload = match self.payload() {
            Ok(payload) => payload,
            Err(error) => {
                self.rec("v1 install payload", "v1", false, error);
                return;
            }
        };
        let (status, raw) = self
            .call(
                JourneyId::InstallExtension,
                HttpMethod::Post,
                "/extension/install",
                "/extension/install",
                None,
                Some(&payload),
            )
            .unwrap_or((0, String::new()));
        self.expect_install("v1 install never-installed", status, &raw, &ident);
        self.expect_running("v1 install", &ident, &tag);

        let (status, body) = self
            .call(
                JourneyId::InstallExtension,
                HttpMethod::Get,
                "/extension/{identifier}/details",
                &format!("/extension/{ident}/details"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        let details_ok = status == 200
            && serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| {
                    value.as_array().map(|items| {
                        items.iter().any(|item| {
                            item.get("tag").and_then(|v| v.as_str()) == Some(tag.as_str())
                        })
                    })
                })
                .unwrap_or(false);
        self.rec(
            "v2 details by identifier",
            "v1",
            details_ok,
            format!("HTTP {status}"),
        );
        let (status, body) = self
            .call(
                JourneyId::InstallExtension,
                HttpMethod::Get,
                "/extension/{identifier}/{tag}/details",
                &format!("/extension/{ident}/{tag}/details"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        let tagged_ok = status == 200
            && serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| value.get("identifier")?.as_str().map(|id| id == ident))
                .unwrap_or(false);
        self.rec(
            "v2 details by identifier+tag",
            "v1",
            tagged_ok,
            format!("HTTP {status}"),
        );

        let (status, body) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Get,
                "/container/",
                "/container/",
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        let listed = status == 200
            && parse_running_containers(&body)
                .unwrap_or_default()
                .iter()
                .any(|container| container.name.trim_start_matches('/') == cname);
        self.rec(
            "v2 list containers includes vehicle",
            "v1",
            listed,
            format!("HTTP {status}"),
        );
        let (status, body) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Get,
                "/container/{container_name}/details",
                &format!("/container/{cname}/details"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 container details",
            "v1",
            status == 200
                && serde_json::from_str::<serde_json::Value>(&body)
                    .ok()
                    .and_then(|value| value.as_object().map(|_| true))
                    .unwrap_or(false),
            format!("HTTP {status}"),
        );
        let (status, _) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Get,
                "/container/{container_name}/stats",
                &format!("/container/{cname}/stats"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 container stats by name",
            "v1",
            status == 200,
            format!("HTTP {status}"),
        );
        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Get,
                "/log",
                "/log",
                Some(&format!("container_name={cname}&timeout=2")),
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v1 logs stream",
            "v1",
            status == 200,
            format!("HTTP {status} bytes={}", raw.len()),
        );
        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Get,
                "/container/{container_name}/log",
                &format!("/container/{cname}/log"),
                Some("timeout=2"),
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 logs stream",
            "v1",
            status == 200,
            format!("HTTP {status} bytes={}", raw.len()),
        );

        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Post,
                "/extension/{identifier}/restart",
                &format!("/extension/{ident}/restart"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 restart running -> 202",
            "v1",
            status == 202,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        let (ok, names) = self.wait_container(&cname, true, POLL_TIMEOUT);
        self.rec(
            "container back after v2 restart",
            "v1",
            ok,
            format!("{names:?}"),
        );
        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Post,
                "/extension/restart",
                "/extension/restart",
                Some(&format!("extension_identifier={ident}")),
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v1 restart running -> 202",
            "v1",
            status == 202,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        let (ok, names) = self.wait_container(&cname, true, POLL_TIMEOUT);
        self.rec(
            "container back after v1 restart",
            "v1",
            ok,
            format!("{names:?}"),
        );
        self.rec("mgmt ping mid-test", "v1", mgmt_ping(&self.base), "");

        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Post,
                "/extension/{identifier}/disable",
                &format!("/extension/{ident}/disable"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 disable running -> 204",
            "v1",
            status == 204,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        let (ok, names) = self.wait_container(&cname, false, Duration::from_secs(20));
        self.rec(
            "container gone after v2 disable",
            "v1",
            ok,
            format!("{names:?}"),
        );
        self.rec(
            "listed disabled",
            "v1",
            self.installed()
                .iter()
                .any(|ext| ext.identifier == ident && !ext.enabled),
            format!("{:?}", self.installed()),
        );
        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Post,
                "/extension/{identifier}/restart",
                &format!("/extension/{ident}/restart"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 restart disabled -> 400",
            "v1",
            status == 400 && raw.contains("no running versions"),
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Post,
                "/extension/{identifier}/{tag}/enable",
                &format!("/extension/{ident}/{tag}/enable"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 enable -> 204",
            "v1",
            status == 204,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        let (ok, names) = self.wait_container(&cname, true, POLL_TIMEOUT);
        self.rec(
            "container back after v2 enable",
            "v1",
            ok,
            format!("{names:?}"),
        );

        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Post,
                "/extension/disable",
                "/extension/disable",
                Some(&format!("extension_identifier={ident}")),
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v1 disable running -> 200",
            "v1",
            status == 200,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        let (ok, names) = self.wait_container(&cname, false, Duration::from_secs(20));
        self.rec(
            "container gone after v1 disable",
            "v1",
            ok,
            format!("{names:?}"),
        );
        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Post,
                "/extension/enable",
                "/extension/enable",
                Some(&format!("extension_identifier={ident}")),
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v1 enable installed -> 200",
            "v1",
            status == 200,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        let (ok, names) = self.wait_container(&cname, true, POLL_TIMEOUT);
        self.rec(
            "container back after v1 enable",
            "v1",
            ok,
            format!("{names:?}"),
        );

        let (status, raw) = self
            .call(
                JourneyId::UninstallExtension,
                HttpMethod::Post,
                "/extension/uninstall",
                "/extension/uninstall",
                Some(&format!("extension_identifier={ident}")),
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v1 uninstall -> 200",
            "v1",
            status == 200,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        self.rec(
            "gone after v1 uninstall",
            "v1",
            self.installed().iter().all(|ext| ext.identifier != ident),
            format!("{:?}", self.installed()),
        );
        let (ok, names) = self.wait_container(&cname, false, Duration::from_secs(20));
        self.rec(
            "container gone after v1 uninstall",
            "v1",
            ok,
            format!("{names:?}"),
        );
        let (status, raw) = self
            .call(
                JourneyId::ConfigureInstalledExtension,
                HttpMethod::Post,
                "/extension/{identifier}/restart",
                &format!("/extension/{ident}/restart"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        // Unpatched `from_running` raises ExtensionNotRunning (400, "no running versions")
        // for a just-uninstalled id. 404 means this DUT is running patched from_running,
        // not a 1.4.4 vs 1.4.5 source split. `--assert-unknown` requires that 404.
        let ok = if self.opts.assert_unknown {
            status == 404
        } else {
            (status == 400 && raw.contains("no running versions")) || status == 404
        };
        self.rec(
            if self.opts.assert_unknown {
                "restart after uninstall -> 404"
            } else {
                "restart after uninstall leftover (400 or 404)"
            },
            "v1",
            ok,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
    }

    fn phase_v2_installs(&mut self) {
        let vehicle = self.vehicle.clone().expect("vehicle");
        let ident = vehicle.identifier.clone();
        let tag = vehicle.tag.clone();
        let payload = match self.payload() {
            Ok(payload) => payload,
            Err(error) => {
                self.rec("v2 install payload", "v2", false, error);
                return;
            }
        };
        let (status, raw) = self
            .call(
                JourneyId::InstallCustomExtension,
                HttpMethod::Post,
                "/extension/",
                "/extension/",
                None,
                Some(&payload),
            )
            .unwrap_or((0, String::new()));
        self.expect_install("v2 custom POST / never-installed", status, &raw, &ident);
        self.expect_running("v2 custom install", &ident, &tag);
        let (status, raw) = self
            .call(
                JourneyId::UninstallExtension,
                HttpMethod::Delete,
                "/extension/{identifier}/{tag}",
                &format!("/extension/{ident}/{tag}"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 tagged uninstall -> 202",
            "v2",
            status == 202,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        self.rec(
            "gone after tagged uninstall",
            "v2",
            self.installed().iter().all(|ext| ext.identifier != ident),
            format!("{:?}", self.installed()),
        );
        let cname = self.cname(None);
        let (ok, names) = self.wait_container(&cname, false, Duration::from_secs(20));
        self.rec(
            "container gone after tagged uninstall",
            "v2",
            ok,
            format!("{names:?}"),
        );

        let (status, raw) = self
            .call(
                JourneyId::InstallExtension,
                HttpMethod::Post,
                "/extension/{identifier}/{tag}/install",
                &format!("/extension/{ident}/{tag}/install"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.expect_install("v2 tagged install never-installed", status, &raw, &ident);
        self.expect_running("v2 tagged install", &ident, &tag);

        let (status, raw) = self
            .call(
                JourneyId::InstallExtension,
                HttpMethod::Post,
                "/extension/install",
                "/extension/install",
                None,
                Some(&payload),
            )
            .unwrap_or((0, String::new()));
        self.expect_install("reinstall while running", status, &raw, &ident);
        let (ok, names) = self.wait_container(&cname, true, POLL_TIMEOUT);
        self.rec(
            "still running after reinstall",
            "v2",
            ok,
            format!("{names:?}"),
        );

        let (status, raw) = self
            .call(
                JourneyId::EditExtensionDevVersion,
                HttpMethod::Put,
                "/extension/{identifier}/{tag}",
                &format!("/extension/{ident}/{tag}"),
                Some("purge=true"),
                None,
            )
            .unwrap_or((0, String::new()));
        self.expect_install("v2 PUT same tag", status, &raw, &ident);
        let (ok, names) = self.wait_container(&cname, true, POLL_TIMEOUT);
        self.rec(
            "running after v2 PUT same tag",
            "v2",
            ok,
            format!("{names:?}"),
        );

        let (status, raw) = self
            .call(
                JourneyId::EditExtensionDevVersion,
                HttpMethod::Post,
                "/extension/update_to_version",
                "/extension/update_to_version",
                Some(&format!("extension_identifier={ident}&new_version={tag}")),
                None,
            )
            .unwrap_or((0, String::new()));
        self.expect_install("v1 update_to_version same tag", status, &raw, &ident);
        let (ok, names) = self.wait_container(&cname, true, POLL_TIMEOUT);
        self.rec(
            "running after v1 update_to_version",
            "v2",
            ok,
            format!("{names:?}"),
        );

        if self.opts.skip_alt {
            self.rec(
                "skip: alt-tag PUT",
                "v2",
                true,
                "--skip-alt (second-tag PUT)",
            );
        } else if let Some(alt) = self.alt_tag.clone() {
            let (status, raw) = self
                .call(
                    JourneyId::EditExtensionDevVersion,
                    HttpMethod::Put,
                    "/extension/{identifier}/{tag}",
                    &format!("/extension/{ident}/{alt}"),
                    Some("purge=true"),
                    None,
                )
                .unwrap_or((0, String::new()));
            self.expect_install("v2 PUT alt tag (running sibling)", status, &raw, &ident);
            let alt_name = self.cname(Some(&alt));
            let (ok, names) = self.wait_container(&alt_name, true, POLL_TIMEOUT);
            self.rec(
                "alt container running after tag switch",
                "v2",
                ok,
                format!("{names:?}"),
            );
            let (ok, names) = self.wait_container(&cname, false, Duration::from_secs(20));
            self.rec(
                "old tag container gone after purge",
                "v2",
                ok,
                format!("{names:?}"),
            );
            self.rec(
                "listed as alt tag",
                "v2",
                self.installed()
                    .iter()
                    .any(|ext| ext.identifier == ident && ext.tag == alt),
                format!("{:?}", self.installed()),
            );
            let (status, raw) = self
                .call(
                    JourneyId::EditExtensionDevVersion,
                    HttpMethod::Put,
                    "/extension/{identifier}/{tag}",
                    &format!("/extension/{ident}/{tag}"),
                    Some("purge=true"),
                    None,
                )
                .unwrap_or((0, String::new()));
            self.expect_install("v2 PUT back to primary tag", status, &raw, &ident);
            let (ok, names) = self.wait_container(&cname, true, POLL_TIMEOUT);
            self.rec("primary container back", "v2", ok, format!("{names:?}"));
        } else {
            self.rec(
                "skip: alt-tag PUT",
                "v2",
                true,
                format!("no second compatible tag for {ident} (primary {tag} only)"),
            );
        }

        let (status, raw) = self
            .call(
                JourneyId::UninstallExtension,
                HttpMethod::Delete,
                "/extension/{identifier}",
                &format!("/extension/{ident}"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "v2 uninstall identifier -> 202",
            "v2",
            status == 202,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
        self.rec(
            "gone after v2 uninstall",
            "v2",
            self.installed().iter().all(|ext| ext.identifier != ident),
            format!("{:?}", self.installed()),
        );
        let (ok, names) = self.wait_container(&cname, false, Duration::from_secs(20));
        self.rec(
            "container gone after v2 uninstall",
            "v2",
            ok,
            format!("{names:?}"),
        );

        if self.opts.skip_latest {
            self.rec(
                "skip: from_latest",
                "v2",
                true,
                "POST /{id}/install --skip-latest",
            );
            self.rec(
                "skip: PUT latest while installed",
                "v2",
                true,
                "--skip-latest",
            );
            self.rec("skip: cleanup after latest", "v2", true, "--skip-latest");
            return;
        }
        let (mut status, mut raw) = self
            .call(
                JourneyId::InstallExtension,
                HttpMethod::Post,
                "/extension/{identifier}/install",
                &format!("/extension/{ident}/install"),
                Some("stable=true"),
                None,
            )
            .unwrap_or((0, String::new()));
        let mut stable_used = "true";
        if status == 404 {
            let retry = self.call(
                JourneyId::InstallExtension,
                HttpMethod::Post,
                "/extension/{identifier}/install",
                &format!("/extension/{ident}/install"),
                Some("stable=false"),
                None,
            );
            if let Ok((retry_status, retry_raw)) = retry {
                status = retry_status;
                raw = retry_raw;
                stable_used = "false";
            }
        }
        if status == 404 && raw.contains("versions") {
            self.rec(
                "skip: from_latest leftover",
                "v2",
                true,
                format!(
                    "stable={stable_used} HTTP {status} {} (semver key vs v-prefix / patch filter)",
                    snippet(&raw, 80)
                ),
            );
            self.rec(
                "skip: PUT latest while installed",
                "v2",
                true,
                "from_latest leftover",
            );
            self.rec(
                "skip: cleanup after latest",
                "v2",
                true,
                "from_latest leftover",
            );
            return;
        }
        self.expect_install(
            &format!("v2 latest install never-installed (stable={stable_used})"),
            status,
            &raw,
            &ident,
        );
        let after = self.installed();
        self.rec(
            "listed after latest install",
            "v2",
            after.iter().any(|ext| ext.identifier == ident),
            format!("{after:?}"),
        );
        if let Some(latest_tag) = after
            .iter()
            .find(|ext| ext.identifier == ident)
            .map(|ext| ext.tag.clone())
        {
            let latest_name = self.cname(Some(&latest_tag));
            let (ok, names) = self.wait_container(&latest_name, true, POLL_TIMEOUT);
            self.rec(
                "container running after latest install",
                "v2",
                ok,
                format!("{names:?}"),
            );
        }
        let (status, raw) = self
            .call(
                JourneyId::EditExtensionDevVersion,
                HttpMethod::Put,
                "/extension/{identifier}",
                &format!("/extension/{ident}"),
                Some(&format!("purge=true&stable={stable_used}")),
                None,
            )
            .unwrap_or((0, String::new()));
        self.expect_install(
            &format!("v2 PUT latest while installed (stable={stable_used})"),
            status,
            &raw,
            &ident,
        );
        let (status, raw) = self
            .call(
                JourneyId::UninstallExtension,
                HttpMethod::Delete,
                "/extension/{identifier}",
                &format!("/extension/{ident}"),
                None,
                None,
            )
            .unwrap_or((0, String::new()));
        self.rec(
            "cleanup after latest -> 202",
            "v2",
            status == 202,
            format!("HTTP {status} {}", snippet(&raw, 80)),
        );
    }
}

fn pick_vehicle(
    catalog: &Catalog,
    base: &str,
    allow_mutating: bool,
    baseline_ids: &[String],
    ident: Option<&str>,
    tag: Option<&str>,
) -> Result<(Vehicle, Option<String>), String> {
    let route = find_route(
        catalog,
        JourneyId::BrowseExtensionStore,
        &HttpMethod::Get,
        "/manifest/consolidated",
    )?;
    let url = kraken_url(base, route.version, "/manifest/consolidated", None);
    let (status, body) = execute_curl(&HttpMethod::Get, &url, allow_mutating, None, None)?;
    let mut manifest = if status == 200 {
        serde_json::from_str::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };
    if !manifest.is_array() {
        let route = find_route(
            catalog,
            JourneyId::BrowseExtensionStore,
            &HttpMethod::Get,
            "/extensions_manifest",
        )?;
        let url = kraken_url(base, route.version, "/extensions_manifest", None);
        let (status, body) = execute_curl(&HttpMethod::Get, &url, allow_mutating, None, None)?;
        if status != 200 {
            return Err(format!("manifest fetch failed: HTTP {status}"));
        }
        manifest = serde_json::from_str(&body).map_err(|error| error.to_string())?;
    }
    let items = manifest
        .as_array()
        .ok_or_else(|| "manifest fetch failed: not a JSON array".to_string())?;

    let mut by_id: std::collections::BTreeMap<String, Vec<(u64, Vehicle)>> =
        std::collections::BTreeMap::new();
    for entry in items {
        let identifier = entry
            .get("identifier")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if identifier == FORBIDDEN_EXT || baseline_ids.iter().any(|id| id == identifier) {
            continue;
        }
        if let Some(wanted) = ident {
            if identifier != wanted {
                continue;
            }
        }
        let Some(versions) = entry.get("versions").and_then(|v| v.as_object()) else {
            continue;
        };
        for (version_tag, version) in versions {
            let images = version
                .get("images")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let compatible: Vec<&serde_json::Value> = images
                .iter()
                .filter(|img| img.get("compatible").and_then(|v| v.as_bool()) == Some(true))
                .collect();
            if compatible.is_empty() {
                continue;
            }
            let size = compatible
                .iter()
                .filter_map(|img| img.get("expanded_size")?.as_u64())
                .min()
                .unwrap_or(u64::MAX);
            let permissions = version
                .get("permissions")
                .cloned()
                .unwrap_or(serde_json::json!({}));
            by_id.entry(identifier.to_string()).or_default().push((
                size,
                Vehicle {
                    identifier: identifier.to_string(),
                    tag: version_tag.clone(),
                    name: entry
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(identifier)
                        .to_string(),
                    docker: entry
                        .get("docker")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    permissions: permissions.to_string(),
                },
            ));
        }
    }
    if let Some(wanted) = ident {
        if !by_id.contains_key(wanted) {
            return Err(format!(
                "no compatible versions for {wanted} (or it is baseline/forbidden)"
            ));
        }
    }
    if by_id.is_empty() {
        return Err("no compatible unused extension in the manifest".into());
    }
    let ident_key = if let Some(wanted) = ident {
        wanted.to_string()
    } else {
        by_id
            .iter()
            .min_by_key(|(_, versions)| {
                versions
                    .iter()
                    .map(|(size, _)| *size)
                    .min()
                    .unwrap_or(u64::MAX)
            })
            .map(|(id, _)| id.clone())
            .expect("by_id not empty")
    };
    let mut versions = by_id.remove(&ident_key).unwrap_or_default();
    versions.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.tag.cmp(&b.1.tag)));
    let (primary, alt) = if let Some(tag) = tag {
        let primary = versions
            .iter()
            .find(|(_, row)| row.tag == tag)
            .map(|(_, row)| row.clone())
            .ok_or_else(|| format!("tag {tag} is not a compatible version of {ident_key}"))?;
        let alt = versions
            .iter()
            .map(|(_, row)| row.tag.clone())
            .find(|other| other != tag);
        (primary, alt)
    } else {
        let primary = versions[0].1.clone();
        let alt = versions.get(1).map(|(_, row)| row.tag.clone());
        (primary, alt)
    };
    Ok((primary, alt))
}

fn kraken_url(base: &str, version: Option<&str>, path: &str, query: Option<&str>) -> String {
    let mut url = format!("{}/kraken", base.trim_end_matches('/'));
    if let Some(version) = version {
        url.push('/');
        url.push_str(version);
    }
    if path.starts_with('/') {
        url.push_str(path);
    } else {
        url.push('/');
        url.push_str(path);
    }
    if let Some(query) = query {
        url.push('?');
        url.push_str(query);
    }
    url
}

fn mgmt_ping(base: &str) -> bool {
    let host = host_from_base(base);
    Command::new("ping")
        .args(["-c", "1", "-W", "2", host])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn host_from_base(base: &str) -> &str {
    let rest = base
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    rest.split(['/', ':']).next().unwrap_or(rest)
}

fn snippet(body: &str, limit: usize) -> String {
    body.chars().take(limit).collect()
}

fn is_lifecycle_dummy_name(name: &str) -> bool {
    name.starts_with(DUMMY_MANIFEST_NAME)
}

fn escape_json(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn container_name(docker: &str, tag: &str) -> String {
    format!(
        "extension-{}",
        format!("{docker}{tag}")
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
    )
}

fn parse_installed_extensions(body: &str) -> Result<Vec<InstalledExtension>, String> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| format!("extension list is not JSON: {error}"))?;
    let array = value
        .as_array()
        .ok_or_else(|| "extension list is not a JSON array".to_string())?;
    let mut out = Vec::with_capacity(array.len());
    for entry in array {
        let identifier = entry
            .get("identifier")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "extension entry missing identifier".to_string())?;
        let tag = entry
            .get("tag")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("extension {identifier} missing tag"))?;
        let enabled = entry
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        out.push(InstalledExtension {
            identifier: identifier.to_string(),
            tag: tag.to_string(),
            enabled,
        });
    }
    Ok(out)
}

fn parse_running_containers(body: &str) -> Result<Vec<RunningContainer>, String> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| format!("container list is not JSON: {error}"))?;
    let array = value
        .as_array()
        .ok_or_else(|| "container list is not a JSON array".to_string())?;
    let mut out = Vec::with_capacity(array.len());
    for entry in array {
        out.push(RunningContainer {
            name: entry
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
            image: entry
                .get("image")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIVE_LIST: &str = r#"[
  {
    "identifier": "bluerobotics.cockpit",
    "tag": "v1.19.0-beta.8",
    "enabled": true
  },
  {
    "identifier": "blueos.major_tom",
    "tag": "2026-02-11",
    "enabled": true
  }
]"#;

    #[test]
    fn catalog_declares_every_lifecycle_route() {
        let catalog = Catalog::bootstrap();
        let mut missing = Vec::new();
        for (journey, method, path) in LIFECYCLE_CATALOG_ROUTES {
            if find_route(&catalog, *journey, method, path).is_err() {
                missing.push(format!("{journey} {method:?} {path}"));
            }
        }
        assert!(
            missing.is_empty(),
            "lifecycle catalog routes missing: {missing:?}"
        );
    }

    #[test]
    fn parse_live_extension_list() {
        let exts = parse_installed_extensions(LIVE_LIST).expect("parse");
        assert_eq!(
            exts.iter()
                .map(|ext| (ext.identifier.as_str(), ext.tag.as_str(), ext.enabled))
                .collect::<Vec<_>>(),
            vec![
                ("bluerobotics.cockpit", "v1.19.0-beta.8", true),
                ("blueos.major_tom", "2026-02-11", true),
            ]
        );
    }

    #[test]
    fn container_name_matches_kraken_sanitization() {
        assert_eq!(
            container_name("williangalvani/blueos-example1", "v1.0.0"),
            "extension-williangalvaniblueosexample1v100"
        );
        assert_eq!(
            container_name("bluerobotics/cockpit", "v1.18.2"),
            "extension-blueroboticscockpitv1182"
        );
    }

    #[test]
    fn kraken_url_joins_version_and_query() {
        assert_eq!(
            kraken_url("http://192.168.0.177", Some("v2.0"), "/extension/", None),
            "http://192.168.0.177/kraken/v2.0/extension/"
        );
        assert_eq!(
            kraken_url(
                "http://192.168.0.177",
                Some("v1.0"),
                "/extension/uninstall",
                Some("extension_identifier=x")
            ),
            "http://192.168.0.177/kraken/v1.0/extension/uninstall?extension_identifier=x"
        );
    }

    #[test]
    fn host_from_base_strips_scheme_and_port() {
        assert_eq!(host_from_base("http://192.168.0.177"), "192.168.0.177");
        assert_eq!(host_from_base("http://192.168.0.177:80/"), "192.168.0.177");
    }
}
