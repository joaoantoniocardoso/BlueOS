use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const BRANDING_UPLOADER: &str = "core/frontend/src/components/customization/BrandingUploader.vue";
const CUSTOMIZATION_MAIN: &str = "core/services/customization/main.py";
const CUSTOMIZATION_STORE: &str = "core/frontend/src/store/customization.ts";
const THEME_CUSTOMIZATION: &str =
    "core/frontend/src/components/customization/ThemeCustomization.vue";

#[allow(dead_code)]
pub fn journeys() -> Vec<UserJourney> {
    vec![
        change_ui_theme_color(),
        reset_ui_theme_color(),
        upload_custom_logo(),
        remove_custom_logo(),
        upload_custom_vehicle_image(),
        remove_custom_vehicle_image(),
        upload_3d_model_override(),
        delete_3d_model_override(),
    ]
}

fn change_ui_theme_color() -> UserJourney {
    UserJourney {
        id: JourneyId("change_ui_theme_color".into()),
        summary: Grounded::known(
            "Customise the primary color that drives the BlueOS interface gradient and scrollbar"
                .into(),
            Provenance::doc(ADV, 968),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 923)),
        services: customization_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "set_theme_color",
            "Settings Appearance panel saves a chosen primary color and regenerates theme CSS",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
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
            operator_step(
                "Click Apply to save the chosen color",
                Some(sourced_route(HttpMethod::Put, "/theme", Some("v1.0"), 144)),
                Provenance::source(THEME_CUSTOMIZATION, 74),
            ),
        ]),
        chains_from: None,
    }
}

fn reset_ui_theme_color() -> UserJourney {
    UserJourney {
        id: JourneyId("reset_ui_theme_color".into()),
        summary: Grounded::known(
            "Restore the default BlueOS primary theme color".into(),
            Provenance::source(THEME_CUSTOMIZATION, 89),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 923)),
        services: customization_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "reset_theme_color",
            "Settings Appearance panel resets theme config and regenerates default CSS",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
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
                "Click Reset and confirm restoring the default BlueOS theme",
                Some(sourced_route(
                    HttpMethod::Delete,
                    "/theme",
                    Some("v1.0"),
                    156,
                )),
                Provenance::source(THEME_CUSTOMIZATION, 88),
            ),
        ]),
        chains_from: None,
    }
}

fn upload_custom_logo() -> UserJourney {
    UserJourney {
        id: JourneyId("upload_custom_logo".into()),
        summary: Grounded::known(
            "Upload a custom company logo for BlueOS branding".into(),
            Provenance::doc(ADV, 879),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
        services: customization_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "upload_branding_logo",
            "Settings Customization panel uploads a square logo image",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
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
            operator_step(
                "Upload the logo",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/branding/logo",
                    Some("v1.0"),
                    273,
                )),
                Provenance::source(BRANDING_UPLOADER, 31),
            ),
        ]),
        chains_from: None,
    }
}

fn remove_custom_logo() -> UserJourney {
    UserJourney {
        id: JourneyId("remove_custom_logo".into()),
        summary: Grounded::known(
            "Remove the custom company logo and revert to default branding".into(),
            Provenance::source(THEME_CUSTOMIZATION, 179),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
        services: customization_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "remove_branding_logo",
            "Settings Customization panel removes the uploaded logo asset",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("A custom logo has been uploaded".into()),
            Provenance::source(THEME_CUSTOMIZATION, 175),
        )]),
        steps: GroundedSet::known(vec![
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
                "Click Remove on the project logo and confirm",
                Some(sourced_route(
                    HttpMethod::Delete,
                    "/branding/logo",
                    Some("v1.0"),
                    279,
                )),
                Provenance::source(BRANDING_UPLOADER, 45),
            ),
        ]),
        chains_from: None,
    }
}

fn upload_custom_vehicle_image() -> UserJourney {
    UserJourney {
        id: JourneyId("upload_custom_vehicle_image".into()),
        summary: Grounded::known(
            "Upload a custom vehicle image shown in the interface".into(),
            Provenance::doc(ADV, 885),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
        services: customization_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "upload_branding_vehicle_image",
            "Settings Customization panel uploads a square vehicle image",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
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
            operator_step(
                "Upload the vehicle image",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/branding/vehicle-image",
                    Some("v1.0"),
                    295,
                )),
                Provenance::source(BRANDING_UPLOADER, 31),
            ),
        ]),
        chains_from: None,
    }
}

fn remove_custom_vehicle_image() -> UserJourney {
    UserJourney {
        id: JourneyId("remove_custom_vehicle_image".into()),
        summary: Grounded::known(
            "Remove the custom vehicle image and revert to default branding".into(),
            Provenance::source(THEME_CUSTOMIZATION, 194),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
        services: customization_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "remove_branding_vehicle_image",
            "Settings Customization panel removes the uploaded vehicle image asset",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("A custom vehicle image has been uploaded".into()),
            Provenance::source(THEME_CUSTOMIZATION, 190),
        )]),
        steps: GroundedSet::known(vec![
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
                "Click Remove on the vehicle image and confirm",
                Some(sourced_route(
                    HttpMethod::Delete,
                    "/branding/vehicle-image",
                    Some("v1.0"),
                    305,
                )),
                Provenance::source(BRANDING_UPLOADER, 45),
            ),
        ]),
        chains_from: None,
    }
}

fn upload_3d_model_override() -> UserJourney {
    UserJourney {
        id: JourneyId("upload_3d_model_override".into()),
        summary: Grounded::known(
            "Replace the Vehicle Setup 3D model with a custom glTF (.glb) file".into(),
            Provenance::doc(ADV, 917),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 652)),
        services: customization_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "upload_model_override",
            "Settings Customization panel uploads a .glb served under userdata/modeloverrides/",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
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
            operator_step(
                "Upload the model override",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/models",
                    Some("v1.0"),
                    197,
                )),
                Provenance::source(CUSTOMIZATION_STORE, 180),
            ),
        ]),
        chains_from: None,
    }
}

fn delete_3d_model_override() -> UserJourney {
    UserJourney {
        id: JourneyId("delete_3d_model_override".into()),
        summary: Grounded::known(
            "Delete an uploaded 3D model override".into(),
            Provenance::source(THEME_CUSTOMIZATION, 369),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 652)),
        services: customization_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "delete_model_override",
            "Settings Customization panel deletes a model from the override list",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("At least one model override has been uploaded".into()),
            Provenance::source(THEME_CUSTOMIZATION, 143),
        )]),
        steps: GroundedSet::known(vec![
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
                "Click the delete button on a model override and confirm",
                Some(sourced_route(
                    HttpMethod::Delete,
                    "/models/{name}",
                    Some("v1.0"),
                    215,
                )),
                Provenance::source(THEME_CUSTOMIZATION, 155),
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn customization_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId("customization".into()),
        Provenance::source(CUSTOMIZATION_MAIN, 34),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId("customization".into()),
        method,
        path: path.into(),
        version: version.map(str::to_string),
    }
}

fn sourced_route(
    method: HttpMethod,
    path: &str,
    version: Option<&str>,
    line: u32,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(CUSTOMIZATION_MAIN, line),
    )
}

fn operator_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description: description.into(),
            route,
            outcome: None,
        },
        provenance,
    )
}
