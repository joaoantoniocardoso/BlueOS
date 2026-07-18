use serde::Serialize;

use crate::catalog::Catalog;
use crate::page::{Page, PageId};
use crate::provenance::Observed;

const CALIBRATION_SMOKE_PAGE_IDS: [PageId; 3] = [
    PageId::VehicleSetup,
    PageId::VideoManager,
    PageId::ParameterEditor,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FrontendSmokeTarget {
    pub page_id: String,
    pub path: String,
    pub name: String,
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

pub fn calibration_smoke_targets(catalog: &Catalog) -> Vec<FrontendSmokeTarget> {
    let mut targets = Vec::with_capacity(CALIBRATION_SMOKE_PAGE_IDS.len());
    for page_id in CALIBRATION_SMOKE_PAGE_IDS {
        let Some(page) = catalog.page_by_id(&page_id) else {
            continue;
        };
        if let Some(target) = smoke_target_from_page(page) {
            targets.push(target);
        }
    }
    targets
}

fn smoke_target_from_page(page: &Page) -> Option<FrontendSmokeTarget> {
    let Observed::Known { value: route, .. } = &page.route else {
        return None;
    };
    let Observed::Known { value: name, .. } = &page.name else {
        return None;
    };
    Some(FrontendSmokeTarget {
        page_id: page.id.as_str().to_string(),
        path: concrete_page_path(route),
        name: (*name).to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

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
    fn calibration_smoke_targets_match_catalog_routes() {
        let targets = calibration_smoke_targets(&Catalog::bootstrap());
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0].page_id, "vehicle_setup");
        assert_eq!(targets[0].path, "/vehicle/setup");
        assert_eq!(targets[1].page_id, "video_manager");
        assert_eq!(targets[1].path, "/vehicle/video-manager");
        assert_eq!(targets[2].page_id, "parameter_editor");
        assert_eq!(targets[2].path, "/vehicle/parameters");
    }
}
