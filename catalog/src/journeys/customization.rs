use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const BRANDING_UPLOADER: &str = "core/frontend/src/components/customization/BrandingUploader.vue";
const CUSTOMIZATION_MAIN: &str = "core/services/customization/main.py";
const CUSTOMIZATION_STORE: &str = "core/frontend/src/store/customization.ts";
const THEME_CUSTOMIZATION: &str =
    "core/frontend/src/components/customization/ThemeCustomization.vue";

pub const JOURNEYS: &[UserJourney] = &[
    CHANGE_UI_THEME_COLOR,
    RESET_UI_THEME_COLOR,
    UPLOAD_CUSTOM_LOGO,
    REMOVE_CUSTOM_LOGO,
    UPLOAD_CUSTOM_VEHICLE_IMAGE,
    REMOVE_CUSTOM_VEHICLE_IMAGE,
    UPLOAD_3D_MODEL_OVERRIDE,
    DELETE_3D_MODEL_OVERRIDE,
];

const CHANGE_UI_THEME_COLOR: UserJourney = UserJourney {
    id: JourneyId::ChangeUiThemeColor,
    summary: Grounded::known(
        "Customise the primary color that drives the BlueOS interface gradient and scrollbar",
        Provenance::doc(ADV, 968),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 923)),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::SetThemeColor,
        "Settings Appearance panel saves a chosen primary color and regenerates theme CSS",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3),
        ),
        operator_step(
            "Pick a primary color with the color picker",
            None,
            Provenance::doc(ADV, 977),
        ),
        operator_step_with_outcome(
            "Click Apply to save the chosen color",
            Some(sourced_route(HttpMethod::Put, "/theme", Some("v1.0"), 144)),
            Provenance::source(THEME_CUSTOMIZATION, 74),
            Some(source_outcome(200, 144)),
        ),
    ]),
    chains_from: None,
};

const RESET_UI_THEME_COLOR: UserJourney = UserJourney {
    id: JourneyId::ResetUiThemeColor,
    summary: Grounded::known(
        "Restore the default BlueOS primary theme color",
        Provenance::source(THEME_CUSTOMIZATION, 89),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 923)),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ResetThemeColor,
        "Settings Appearance panel resets theme config and regenerates default CSS",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3),
        ),
        operator_step_with_outcome(
            "Click Reset and confirm restoring the default BlueOS theme",
            Some(sourced_route(
                HttpMethod::Delete,
                "/theme",
                Some("v1.0"),
                156,
            )),
            Provenance::source(THEME_CUSTOMIZATION, 88),
            Some(source_outcome(204, 156)),
        ),
    ]),
    chains_from: None,
};

const UPLOAD_CUSTOM_LOGO: UserJourney = UserJourney {
    id: JourneyId::UploadCustomLogo,
    summary: Grounded::known(
        "Upload a custom company logo for BlueOS branding",
        Provenance::doc(ADV, 879),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::UploadBrandingLogo,
        "Settings Customization panel uploads a square logo image",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3),
        ),
        operator_step(
            "Choose an image file for the project logo",
            None,
            Provenance::doc(ADV, 881),
        ),
        operator_step_with_outcome(
            "Upload the logo",
            Some(sourced_route(
                HttpMethod::Post,
                "/branding/logo",
                Some("v1.0"),
                273,
            )),
            Provenance::source(BRANDING_UPLOADER, 31),
            Some(source_outcome(200, 275)),
        ),
    ]),
    chains_from: None,
};

const REMOVE_CUSTOM_LOGO: UserJourney = UserJourney {
    id: JourneyId::RemoveCustomLogo,
    summary: Grounded::known(
        "Remove the custom company logo and revert to default branding",
        Provenance::source(THEME_CUSTOMIZATION, 179),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveBrandingLogo,
        "Settings Customization panel removes the uploaded logo asset",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A custom logo has been uploaded"),
        Provenance::source(THEME_CUSTOMIZATION, 175),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3),
        ),
        operator_step_with_outcome(
            "Click Remove on the project logo and confirm",
            Some(sourced_route(
                HttpMethod::Delete,
                "/branding/logo",
                Some("v1.0"),
                279,
            )),
            Provenance::source(BRANDING_UPLOADER, 45),
            Some(source_outcome(204, 279)),
        ),
    ]),
    chains_from: None,
};

const UPLOAD_CUSTOM_VEHICLE_IMAGE: UserJourney = UserJourney {
    id: JourneyId::UploadCustomVehicleImage,
    summary: Grounded::known(
        "Upload a custom vehicle image shown in the interface",
        Provenance::doc(ADV, 885),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::UploadBrandingVehicleImage,
        "Settings Customization panel uploads a square vehicle image",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3),
        ),
        operator_step(
            "Choose an image file for the vehicle image",
            None,
            Provenance::doc(ADV, 887),
        ),
        operator_step_with_outcome(
            "Upload the vehicle image",
            Some(sourced_route(
                HttpMethod::Post,
                "/branding/vehicle-image",
                Some("v1.0"),
                295,
            )),
            Provenance::source(BRANDING_UPLOADER, 31),
            Some(source_outcome(200, 301)),
        ),
    ]),
    chains_from: None,
};

const REMOVE_CUSTOM_VEHICLE_IMAGE: UserJourney = UserJourney {
    id: JourneyId::RemoveCustomVehicleImage,
    summary: Grounded::known(
        "Remove the custom vehicle image and revert to default branding",
        Provenance::source(THEME_CUSTOMIZATION, 194),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveBrandingVehicleImage,
        "Settings Customization panel removes the uploaded vehicle image asset",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A custom vehicle image has been uploaded"),
        Provenance::source(THEME_CUSTOMIZATION, 190),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3),
        ),
        operator_step_with_outcome(
            "Click Remove on the vehicle image and confirm",
            Some(sourced_route(
                HttpMethod::Delete,
                "/branding/vehicle-image",
                Some("v1.0"),
                307,
            )),
            Provenance::source(BRANDING_UPLOADER, 45),
            Some(source_outcome(204, 307)),
        ),
    ]),
    chains_from: None,
};

const UPLOAD_3D_MODEL_OVERRIDE: UserJourney = UserJourney {
    id: JourneyId::Upload3dModelOverride,
    summary: Grounded::known(
        "Replace the Vehicle Setup 3D model with a custom glTF (.glb) file",
        Provenance::doc(ADV, 917),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 652)),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::UploadModelOverride,
        "Settings Customization panel uploads a .glb served under userdata/modeloverrides/",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3),
        ),
        operator_step(
            "Open the Upload model dialog, select a .glb file, and set the destination name",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 204),
        ),
        operator_step_with_outcome(
            "Upload the model override",
            Some(sourced_route(
                HttpMethod::Post,
                "/models",
                Some("v1.0"),
                197,
            )),
            Provenance::source(CUSTOMIZATION_STORE, 180),
            Some(source_outcome(200, 199)),
        ),
    ]),
    chains_from: None,
};

const DELETE_3D_MODEL_OVERRIDE: UserJourney = UserJourney {
    id: JourneyId::Delete3dModelOverride,
    summary: Grounded::known(
        "Delete an uploaded 3D model override",
        Provenance::source(THEME_CUSTOMIZATION, 369),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 652)),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DeleteModelOverride,
        "Settings Customization panel deletes a model from the override list",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("At least one model override has been uploaded"),
        Provenance::source(THEME_CUSTOMIZATION, 143),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3),
        ),
        operator_step_with_outcome(
            "Click the delete button on a model override and confirm",
            Some(sourced_route(
                HttpMethod::Delete,
                "/models/{name}",
                Some("v1.0"),
                215,
            )),
            Provenance::source(THEME_CUSTOMIZATION, 155),
            Some(source_outcome(204, 215)),
        ),
    ]),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const CUSTOMIZATION_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Customization,
    Provenance::source(CUSTOMIZATION_MAIN, 34),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Customization,
        method,
        path,
        version,
    }
}

const fn sourced_route(
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    line: u32,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(CUSTOMIZATION_MAIN, line),
    )
}

const fn operator_step_with_outcome(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description,
            route,
            outcome,
        },
        provenance,
    )
}

const fn source_outcome(status: u16, line: u32) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            transition: None,
        },
        Provenance::source(CUSTOMIZATION_MAIN, line),
    )
}

const fn operator_step(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description,
            route,
            outcome: None,
        },
        provenance,
    )
}
