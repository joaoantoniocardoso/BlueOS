use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, BodyKind, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome,
    UserJourney, Visibility,
};
use crate::journey_presence::{
    PRESENCE_CHANGE_UI_THEME_COLOR, PRESENCE_DELETE3D_MODEL_OVERRIDE, PRESENCE_REMOVE_CUSTOM_LOGO,
    PRESENCE_REMOVE_CUSTOM_VEHICLE_IMAGE, PRESENCE_RESET_UI_THEME_COLOR,
    PRESENCE_UPLOAD3D_MODEL_OVERRIDE, PRESENCE_UPLOAD_CUSTOM_LOGO,
    PRESENCE_UPLOAD_CUSTOM_VEHICLE_IMAGE,
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
        Provenance::doc(ADV, 968, "### Theme Styling"),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 923, "{% pirate() %}")),
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
            Provenance::doc(ADV, 200, "##### BlueOS Settings"),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3, "<v-row align=\"center\" no-gutters class=\"customization-head"),
        ),
        operator_step(
            "Pick a primary color with the color picker",
            None,
            Provenance::doc(ADV, 977, "adjusting the BlueOS theme, the most important thing to unde"),
        ),
        operator_step_with_outcome(
            "Click Apply to save the chosen color",
            Some(sourced_route(HttpMethod::Put, "/theme", Some("v1.0"), 144, "@theme_router.put(\"\", response_model")),
            Provenance::source(THEME_CUSTOMIZATION, 74, "<v-btn"),
            Some(source_outcome(200, 144, "@theme_router.put(\"\", response_model=ThemeStatus, summary=\"S")),
        ),
    ]),
    availability: PRESENCE_CHANGE_UI_THEME_COLOR,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "Primary theme color is userdata CSS restorable via reset or a prior GET /theme snapshot",
        ),
    ),
    chains_from: None,
};

const RESET_UI_THEME_COLOR: UserJourney = UserJourney {
    id: JourneyId::ResetUiThemeColor,
    summary: Grounded::known(
        "Restore the default BlueOS primary theme color",
        Provenance::source(
            THEME_CUSTOMIZATION,
            89,
            "v-tooltip=\"'Restore the default BlueOS theme'\"",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 923, "{% pirate() %}"),
    ),
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
            Provenance::doc(ADV, 200, "##### BlueOS Settings"),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(
                THEME_CUSTOMIZATION,
                3,
                "<v-row align=\"center\" no-gutters class=\"customization-header",
            ),
        ),
        operator_step_with_outcome(
            "Click Reset and confirm restoring the default BlueOS theme",
            Some(sourced_route(
                HttpMethod::Delete,
                "/theme",
                Some("v1.0"),
                156,
                "@theme_router.delete(\"\", status_code=status.HTTP_204_NO_CONT",
            )),
            Provenance::source(THEME_CUSTOMIZATION, 88, "<v-btn"),
            Some(source_outcome(
                204,
                156,
                "@theme_router.delete(\"\", status_code=status.HTTP_204_NO_CONT",
            )),
        ),
    ]),
    availability: PRESENCE_RESET_UI_THEME_COLOR,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "DELETE /theme drops custom primary color and regenerates the default BlueOS theme CSS",
        ),
    ),
    chains_from: None,
};

const UPLOAD_CUSTOM_LOGO: UserJourney = UserJourney {
    id: JourneyId::UploadCustomLogo,
    summary: Grounded::known(
        "Upload a custom company logo for BlueOS branding",
        Provenance::doc(ADV, 879, "#### Company Logo"),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(
            ADV,
            875,
            "The vehicle identifier components in the sidebar can be modi",
        ),
    ),
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
            Provenance::doc(ADV, 200, "##### BlueOS Settings"),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(
                THEME_CUSTOMIZATION,
                3,
                "<v-row align=\"center\" no-gutters class=\"customization-header",
            ),
        ),
        operator_step(
            "Choose an image file for the project logo",
            None,
            Provenance::doc(ADV, 881, "- square images work best"),
        ),
        operator_step_with_outcome(
            "Upload the logo",
            Some(sourced_route(
                HttpMethod::Post,
                "/branding/logo",
                Some("v1.0"),
                273,
                "@branding_router.post(\"/logo\", response_model=BrandingAsset,",
            )),
            Provenance::source(BRANDING_UPLOADER, 31, "@click=\"trigger_picker\""),
            Some(source_outcome(
                200,
                275,
                "async def upload_logo(file: UploadFile = File(...)) -> Brand",
            )),
        ),
    ]),
    availability: PRESENCE_UPLOAD_CUSTOM_LOGO,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "Uploaded logo is a userdata branding file removable via DELETE /branding/logo",
        ),
    ),
    chains_from: None,
};

const REMOVE_CUSTOM_LOGO: UserJourney = UserJourney {
    id: JourneyId::RemoveCustomLogo,
    summary: Grounded::known(
        "Remove the custom company logo and revert to default branding",
        Provenance::source(
            THEME_CUSTOMIZATION,
            179,
            "empty-label=\"No custom logo set.\"",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(
            ADV,
            875,
            "The vehicle identifier components in the sidebar can be modi",
        ),
    ),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveBrandingLogo,
        "Settings Customization panel removes the uploaded logo asset",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A custom logo has been uploaded"),
        Provenance::source(THEME_CUSTOMIZATION, 175, ":asset=\"logo\""),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200, "##### BlueOS Settings"),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(
                THEME_CUSTOMIZATION,
                3,
                "<v-row align=\"center\" no-gutters class=\"customization-header",
            ),
        ),
        operator_step_with_outcome(
            "Click Remove on the project logo and confirm",
            Some(sourced_route(
                HttpMethod::Delete,
                "/branding/logo",
                Some("v1.0"),
                279,
                "@branding_router.delete(\"/logo\", status_code=status.HTTP_204",
            )),
            Provenance::source(BRANDING_UPLOADER, 45, "@click=\"$emit('remove')\""),
            Some(source_outcome(
                204,
                279,
                "@branding_router.delete(\"/logo\", status_code=status.HTTP_204",
            )),
        ),
    ]),
    availability: PRESENCE_REMOVE_CUSTOM_LOGO,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "DELETE /branding/logo removes the custom logo and reverts to default branding",
        ),
    ),
    chains_from: None,
};

const UPLOAD_CUSTOM_VEHICLE_IMAGE: UserJourney = UserJourney {
    id: JourneyId::UploadCustomVehicleImage,
    summary: Grounded::known(
        "Upload a custom vehicle image shown in the interface",
        Provenance::doc(ADV, 885, "#### Vehicle Icon"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875, "The vehicle identifier components ")),
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
            Provenance::doc(ADV, 200, "##### BlueOS Settings"),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3, "<v-row align=\"center\" no-gutters class=\"customization-head"),
        ),
        operator_step(
            "Choose an image file for the vehicle image",
            None,
            Provenance::doc(ADV, 887, "- square images work best"),
        ),
        operator_step_with_outcome(
            "Upload the vehicle image",
            Some(sourced_route(HttpMethod::Post, "/branding/vehicle-image", Some("v1.0"), 295, "@branding_router.p")),
            Provenance::source(BRANDING_UPLOADER, 31, "@click=\"trigger_picker\""),
            Some(source_outcome(200, 301, "async def upload_vehicle_image(file: UploadFile = File(...))")),
        ),
    ]),
    availability: PRESENCE_UPLOAD_CUSTOM_VEHICLE_IMAGE,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "Uploaded vehicle image is a userdata branding file removable via DELETE /branding/vehicle-image",
        ),
    ),
    chains_from: None,
};

const REMOVE_CUSTOM_VEHICLE_IMAGE: UserJourney = UserJourney {
    id: JourneyId::RemoveCustomVehicleImage,
    summary: Grounded::known(
        "Remove the custom vehicle image and revert to default branding",
        Provenance::source(THEME_CUSTOMIZATION, 194, "empty-label=\"No custom vehicle image set.\""),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875, "The vehicle identifier components ")),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveBrandingVehicleImage,
        "Settings Customization panel removes the uploaded vehicle image asset",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A custom vehicle image has been uploaded"),
        Provenance::source(THEME_CUSTOMIZATION, 190, ":asset=\"vehicle_image\""),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200, "##### BlueOS Settings"),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3, "<v-row align=\"center\" no-gutters class=\"customization-head"),
        ),
        operator_step_with_outcome(
            "Click Remove on the vehicle image and confirm",
            Some(sourced_route(HttpMethod::Delete, "/branding/vehicle-image", Some("v1.0"), 307, "status_code=stat")),
            Provenance::source(BRANDING_UPLOADER, 45, "@click=\"$emit('remove')\""),
            Some(source_outcome(204, 307, "status_code=status.HTTP_204_NO_CONTENT,")),
        ),
    ]),
    availability: PRESENCE_REMOVE_CUSTOM_VEHICLE_IMAGE,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "DELETE /branding/vehicle-image removes the custom image and reverts to default branding",
        ),
    ),
    chains_from: None,
};

const UPLOAD_3D_MODEL_OVERRIDE: UserJourney = UserJourney {
    id: JourneyId::Upload3dModelOverride,
    summary: Grounded::known(
        "Replace the Vehicle Setup 3D model with a custom glTF (.glb) file",
        Provenance::doc(
            ADV,
            917,
            "The 3D model used in the [Vehicle Setup](#vehicle-setup) pag",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 652, "{% pirate() %}"),
    ),
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
            Provenance::doc(ADV, 200, "##### BlueOS Settings"),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(
                THEME_CUSTOMIZATION,
                3,
                "<v-row align=\"center\" no-gutters class=\"customization-header",
            ),
        ),
        operator_step(
            "Open the Upload model dialog, select a .glb file, and set the destination name",
            None,
            Provenance::source(
                THEME_CUSTOMIZATION,
                204,
                "<v-dialog v-model=\"model_dialog\" max-width=\"500\">",
            ),
        ),
        operator_step_with_outcome(
            "Upload the model override",
            Some(sourced_route(
                HttpMethod::Post,
                "/models",
                Some("v1.0"),
                197,
                "@models_router.post(\"\", response_model=ModelEntry, summary=\"",
            )),
            Provenance::source(CUSTOMIZATION_STORE, 180, "await back_axios({"),
            Some(source_outcome(200, 199, "async def upload_model(")),
        ),
    ]),
    availability: PRESENCE_UPLOAD3D_MODEL_OVERRIDE,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "Custom glTF override is a userdata file deletable via DELETE /models/{name}",
        ),
    ),
    chains_from: None,
};

const DELETE_3D_MODEL_OVERRIDE: UserJourney = UserJourney {
    id: JourneyId::Delete3dModelOverride,
    summary: Grounded::known(
        "Delete an uploaded 3D model override",
        Provenance::source(THEME_CUSTOMIZATION, 369, "if (!window.confirm(`Delete the model override \"${model.name"),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 652, "{% pirate() %}")),
    services: CUSTOMIZATION_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DeleteModelOverride,
        "Settings Customization panel deletes a model from the override list",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("At least one model override has been uploaded"),
        Provenance::source(THEME_CUSTOMIZATION, 143, "<v-list v-if=\"models.length\" dense class=\"model-list\">"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Settings from the sidebar",
            None,
            Provenance::doc(ADV, 200, "##### BlueOS Settings"),
        ),
        operator_step(
            "Expand the Customization panel under Appearance",
            None,
            Provenance::source(THEME_CUSTOMIZATION, 3, "<v-row align=\"center\" no-gutters class=\"customization-head"),
        ),
        operator_step_with_outcome(
            "Click the delete button on a model override and confirm",
            Some(sourced_route(HttpMethod::Delete, "/models/{name}", Some("v1.0"), 215, "@models_router.delete(")),
            Provenance::source(THEME_CUSTOMIZATION, 155, "<v-btn icon small color=\"error\" @click=\"delete_model(mod"),
            Some(source_outcome(204, 215, "@models_router.delete(")),
        ),
    ]),
    availability: PRESENCE_DELETE3D_MODEL_OVERRIDE,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "DELETE /models/{name} removes one override; Vehicle Setup falls back to the stock model",
        ),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const CUSTOMIZATION_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Customization,
    Provenance::source(CUSTOMIZATION_MAIN, 34, "SERVICE_NAME = \"customization\""),
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
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(CUSTOMIZATION_MAIN, line, anchor),
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

const fn source_outcome(status: u16, line: u32, anchor: &'static str) -> Grounded<StepOutcome> {
    let body_kind = if status == 204 {
        BodyKind::Empty
    } else {
        BodyKind::Unknown
    };
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind,
            transition: None,
        },
        Provenance::source(CUSTOMIZATION_MAIN, line, anchor),
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
