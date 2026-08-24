use serde::Serialize;

use catalog_core::catalog::Catalog;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::page::PageId;
use catalog_kernel::provenance::Observed;
use catalog_model::page::Page;

/// Pages whose Vue route landed after this 1.4-dev pin (first tag 1.4.4+ / 1.5.0+).
fn page_presence_journey(page_id: PageId) -> Option<JourneyId> {
    match page_id {
        PageId::Disk => Some(JourneyId::InspectDiskUsage),
        PageId::Records => Some(JourneyId::BrowseVideoRecordings),
        PageId::ZenohInspector => Some(JourneyId::InspectZenohNetwork),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FrontendSmokeTarget {
    pub page_id: String,
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub landmarks: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
}

pub fn concrete_page_path(route: &str) -> String {
    let route = route.split('?').next().unwrap_or(route);
    let segments: Vec<&str> = route
        .split('/')
        .filter(|segment| !segment.is_empty() && !segment.starts_with(':'))
        .collect();
    if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    }
}

pub fn frontend_smoke_targets(catalog: &Catalog) -> Vec<FrontendSmokeTarget> {
    let mut targets = Vec::with_capacity(PageId::ALL.len());
    for page_id in PageId::ALL {
        let Some(page) = catalog.page_by_id(&page_id) else {
            continue;
        };
        if let Some(target) = smoke_target_from_page(catalog, page) {
            targets.push(target);
        }
    }
    targets
}

pub fn calibration_smoke_targets(catalog: &Catalog) -> Vec<FrontendSmokeTarget> {
    frontend_smoke_targets(catalog)
}

fn smoke_target_from_page(catalog: &Catalog, page: &Page) -> Option<FrontendSmokeTarget> {
    let Observed::Known { value: route, .. } = &page.route else {
        return None;
    };
    let Observed::Known { value: name, .. } = &page.name else {
        return None;
    };
    let mut landmarks = Vec::new();
    if page.id != PageId::Main {
        if let Observed::Known {
            value: menu_title, ..
        } = &page.menu_title
        {
            landmarks.push((*menu_title).to_string());
        }
        if !landmarks.iter().any(|l| l == name) {
            landmarks.push((*name).to_string());
        }
        landmarks.extend(extra_landmarks(page.id).iter().map(|s| (*s).to_string()));
    }
    Some(FrontendSmokeTarget {
        page_id: page.id.as_str().to_string(),
        path: concrete_page_path(route),
        name: (*name).to_string(),
        landmarks,
        skip_reason: page_skip_reason(catalog, page.id),
    })
}

fn extra_landmarks(page_id: PageId) -> &'static [&'static str] {
    match page_id {
        PageId::Endpoints => &["Mavlink Router"],
        PageId::BagEditor => &["powered by ace"],
        PageId::ExtensionManager => &["INSTALLED"],
        PageId::MavlinkInspector => &["HEARTBEAT"],
        _ => &[],
    }
}

fn page_skip_reason(catalog: &Catalog, page_id: PageId) -> Option<String> {
    if page_id == PageId::Extensions {
        return Some("needs_extension_port".to_string());
    }
    if page_id == PageId::Settings {
        return Some("settings_is_dialog_not_route_on_1.4-dev".to_string());
    }
    let journey_id = page_presence_journey(page_id)?;
    let journey = catalog.journey_by_id(&journey_id)?;
    if journey.availability.present_on_1_4_dev {
        return None;
    }
    Some(format!("not_on_1.4-dev ({})", journey_id.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog_core::catalog::Catalog;
    use catalog_kernel::id::page::PageId;

    #[test]
    fn concrete_page_path_strips_optional_params() {
        assert_eq!(
            concrete_page_path("/vehicle/setup/:tab?/:subtab?"),
            "/vehicle/setup"
        );
    }

    #[test]
    fn concrete_page_path_keeps_static_segments() {
        assert_eq!(
            concrete_page_path("/vehicle/video-manager"),
            "/vehicle/video-manager"
        );
        assert_eq!(
            concrete_page_path("/vehicle/parameters"),
            "/vehicle/parameters"
        );
    }

    #[test]
    fn concrete_page_path_drops_required_params() {
        assert_eq!(concrete_page_path("/extensions/:id"), "/extensions");
    }

    #[test]
    fn frontend_smoke_targets_cover_all_pages_and_skip_absent() {
        let targets = frontend_smoke_targets(&Catalog::bootstrap());
        assert_eq!(targets.len(), PageId::ALL.len());
        let vehicle = targets
            .iter()
            .find(|t| t.page_id == "vehicle_setup")
            .unwrap();
        assert_eq!(vehicle.path, "/vehicle/setup");
        assert!(vehicle.skip_reason.is_none());
        assert!(vehicle.landmarks.iter().any(|l| l == "Vehicle Setup"));
        for page_id in ["disk", "records", "zenoh_inspector"] {
            let target = targets.iter().find(|t| t.page_id == page_id).unwrap();
            assert!(
                target
                    .skip_reason
                    .as_deref()
                    .is_some_and(|reason| reason.starts_with("not_on_1.4-dev")),
                "{page_id} should skip on 1.4-dev"
            );
        }
        let main = targets.iter().find(|t| t.page_id == "main").unwrap();
        assert!(main.landmarks.is_empty());
        let extensions = targets.iter().find(|t| t.page_id == "extensions").unwrap();
        assert_eq!(
            extensions.skip_reason.as_deref(),
            Some("needs_extension_port")
        );
        let settings = targets.iter().find(|t| t.page_id == "settings").unwrap();
        assert!(settings.skip_reason.as_deref().unwrap().contains("dialog"));
    }
}
