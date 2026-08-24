use crate::catalog::Catalog;
use catalog_core::validate::{
    functions_check_applies, requirements_check_applies, ValidationError,
};
use catalog_derive::feature::validate_capabilities;
use catalog_derive::function::{
    capability_exists, function_id_looks_like_http_path, ActionCatalog,
};
use catalog_derive::requirement::{RequirementCatalog, RequirementClass};

use crate::harness_ratchet::harness_ratchet_counts;

pub fn check_harness_annotation_stubs(catalog: &Catalog) -> Vec<ValidationError> {
    // H0 invariant: validate() stays green while Unknowns remain; ratchet is harness_ratchet bin.
    let _ = harness_ratchet_counts(catalog);
    Vec::new()
}

pub fn check_requirements(catalog: &Catalog) -> Vec<ValidationError> {
    if !requirements_check_applies(catalog) {
        return Vec::new();
    }
    if let Err(capability_errors) = validate_capabilities(catalog) {
        return capability_errors
            .into_iter()
            .map(|detail| ValidationError::RequirementStructure { detail })
            .collect();
    }
    let requirements = RequirementCatalog::from_catalog(catalog);
    match requirements.validate_structure(catalog) {
        Ok(()) => Vec::new(),
        Err(req_errors) => req_errors
            .into_iter()
            .map(|error| ValidationError::RequirementStructure {
                detail: error.to_string(),
            })
            .collect(),
    }
}

pub fn check_functions(catalog: &Catalog) -> Vec<ValidationError> {
    if !functions_check_applies(catalog) {
        return Vec::new();
    }

    let functions = ActionCatalog::from_catalog(catalog);
    let requirements = RequirementCatalog::from_catalog(catalog);
    check_function_integrity(catalog, &functions, &requirements)
}

fn check_function_integrity(
    catalog: &Catalog,
    functions: &ActionCatalog,
    requirements: &RequirementCatalog,
) -> Vec<ValidationError> {
    use std::collections::HashSet;

    let journey_ids: HashSet<&str> = catalog
        .journeys()
        .iter()
        .map(|journey| journey.id.as_str())
        .collect();
    let mut errors = Vec::new();

    for function in functions.functions() {
        let function_id = function.id.as_str().to_string();
        if !capability_exists(function.capability) {
            errors.push(ValidationError::UnknownFunctionCapability {
                function: function_id.clone(),
                capability: function.capability.as_str().to_string(),
            });
        }
        if function.verifying_journeys.is_empty() {
            errors.push(ValidationError::FunctionNoVerifyingJourneys {
                function: function_id.clone(),
            });
        }
        let capability_key = function.capability.as_str();
        let capability_keyed =
            function_id == capability_key || function_id.starts_with(&format!("{capability_key}/"));
        if journey_ids.contains(function.id.as_str()) && !capability_keyed {
            errors.push(ValidationError::FunctionIdEqualsJourneyId {
                function: function_id.clone(),
            });
        }
        if function_id_looks_like_http_path(function.id.as_str()) {
            errors.push(ValidationError::FunctionIdContainsHttpPath {
                function: function_id.clone(),
            });
        }
        let fun_suffix = format!("/FUN/{}", function.id.as_str());
        let has_fun = requirements
            .requirements()
            .iter()
            .any(|req| req.kind == RequirementClass::Functional && req.id.0.ends_with(&fun_suffix));
        if !has_fun {
            errors.push(ValidationError::MissingFunRequirement {
                function: function_id,
            });
        }
    }

    for requirement in requirements.requirements() {
        if requirement.kind != RequirementClass::Functional {
            continue;
        }
        let id = requirement.id.0.clone();
        if requirement.function_id.is_none() {
            errors.push(ValidationError::FunRequirementMissingFunctionId { id: id.clone() });
        }
        if requirement.requirement_verifications.is_empty() {
            errors.push(ValidationError::FunRequirementNoVerifyingJourneys { id });
        }
    }

    for requirement in requirements.requirements() {
        if requirement.kind != RequirementClass::System || requirement.id.0.contains("/SYS-OVR/") {
            continue;
        }
        for child in &requirement.subrequirements {
            if !child.0.contains("/FUN/") {
                errors.push(ValidationError::SystemChildNotFunId {
                    id: requirement.id.0.clone(),
                    child: child.0.clone(),
                });
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog_derive::function::ActionCatalog;
    use catalog_derive::requirement::{RequirementCatalog, RequirementClass, RequirementId};

    #[test]
    fn bootstrap_catalog_passes_full_validate() {
        let catalog = Catalog::bootstrap();
        assert!(catalog.validate().is_ok());
        assert!(crate::validate::validate(&catalog).is_ok());
    }

    #[test]
    fn function_integrity_checks_fail_when_fun_rows_are_broken() {
        let catalog = Catalog::bootstrap();
        let functions = ActionCatalog::from_catalog(&catalog);

        let mut missing_fun = RequirementCatalog::from_catalog(&catalog);
        missing_fun
            .requirements_mut()
            .retain(|req| req.kind != RequirementClass::Functional);
        assert!(check_function_integrity(&catalog, &functions, &missing_fun)
            .iter()
            .any(|error| matches!(error, ValidationError::MissingFunRequirement { .. })));

        let mut no_id = RequirementCatalog::from_catalog(&catalog);
        for req in no_id.requirements_mut() {
            if req.kind == RequirementClass::Functional {
                req.function_id = None;
            }
        }
        assert!(check_function_integrity(&catalog, &functions, &no_id)
            .iter()
            .any(|error| matches!(
                error,
                ValidationError::FunRequirementMissingFunctionId { .. }
            )));

        let mut no_journeys = RequirementCatalog::from_catalog(&catalog);
        for req in no_journeys.requirements_mut() {
            if req.kind == RequirementClass::Functional {
                req.requirement_verifications.clear();
            }
        }
        assert!(check_function_integrity(&catalog, &functions, &no_journeys)
            .iter()
            .any(|error| matches!(
                error,
                ValidationError::FunRequirementNoVerifyingJourneys { .. }
            )));

        let mut bad_child = RequirementCatalog::from_catalog(&catalog);
        let child = RequirementId("REQ/test/test/SYS/not_a_fun".to_string());
        let sys = bad_child
            .requirements_mut()
            .iter_mut()
            .find(|req| {
                req.kind == RequirementClass::System
                    && !req.id.0.contains("/SYS-OVR/")
                    && !req.subrequirements.is_empty()
            })
            .expect("system requirement with FUN children");
        sys.subrequirements[0] = child;
        assert!(check_function_integrity(&catalog, &functions, &bad_child)
            .iter()
            .any(|error| matches!(error, ValidationError::SystemChildNotFunId { .. })));
    }
}
