---
name: blueos-frontend-extraction
description: >-
  Model one BlueOS frontend Page as a first-class catalog artifact: its route,
  menu placement/visibility, view component, the Vuex stores it uses, the backend
  services it consumes (page->service fan-out), the domain state it holds
  client-side (with ownership), and the features it implements in the browser with
  no backend equivalent (e.g. sensor calibration). Use as the Frontend Extractor
  agent when building or updating a catalog `pages/<id>` artifact. Observed fields
  carry `core/frontend/src/...:LINE` provenance; asserted fields carry a rationale.
disable-model-invocation: true
---

# BlueOS Frontend Extraction

BlueOS 1.x implements large amounts of user-facing behavior in the Vue 2 frontend
(`core/frontend/src`), not the backend: whole features (sensor calibration, motor
detection, parameter editing) live in `.vue`/`.ts` files, orchestrate several
backend services at once, and often **hold domain state in the browser**. A page
does **not** map 1:1 to a service. The catalog must model the frontend so this
"invisible" surface is captured. The unit is the **Page** (a router route), not the
208 raw components.

## Non-negotiable rules

- **Provenance or Unknown.** Every OBSERVED field cites `core/frontend/src/...:LINE`
  (the router entry, the `menus.ts` entry, the component, the store, the axios/
  mavlink2rest call). No evidence -> `Observed::unknown(reason)`. Never guess.
- **Asserted fields carry a rationale, never `file:line`.** `frontend_features` and
  `client_state` are judgment; wrap in `Asserted`/`AssertedSet` with a rationale
  that *points at* the implementing component but is not raw evidence.
- **Only real backend targets are edges.** A `PageServiceCall.service` MUST be
  `ConsumeTarget::Service(ServiceId::X)` for a cataloged service (the 26) or
  `ConsumeTarget::External` for the public internet.
  Map the call by its base URL / nginx prefix / port (e.g. `mavlink2rest`=6040
  `/mavlink2rest/`, `ardupilot_manager` `/ardupilot-manager/`, etc. — see
  `core/tools/nginx/nginx.conf` and the catalog service cards).
- **Distinguish frontend features from backend passthrough.** If the page just
  proxies a backend capability (e.g. a button that POSTs to a service route), that
  is a `consumes` edge, NOT a `frontend_feature`. A `frontend_feature` is logic
  IMPLEMENTED in the client (a wizard, a state machine, a derived-from-params
  computation, a MAVLink command sequence) with no single backend capability.
- Output is `pub const PAGE: Page` in `catalog/src/pages/<id>.rs`, registered in
  `catalog/src/pages/mod.rs`. Keep the crate green.

## Ground-truth sources (read these)

- `core/frontend/src/router/index.ts` — the route: `path`, `name`, `component`.
- `core/frontend/src/menus.ts` — `title`, `icon`, `route`, `advanced: bool`
  (advanced-mode-only visibility). A page absent from a menu -> `menu_title` /
  `advanced_only` = `Unknown`.
- `core/frontend/src/views/<View>.vue` + its child components under
  `core/frontend/src/components/**` (follow the imports; wizards/flows live here).
- `core/frontend/src/store/*.ts` — the Vuex modules the page/its components use;
  this is where client-held state and many backend calls actually live.
- `core/frontend/src/libs/**` (e.g. `MAVLink2Rest`) — the MAVLink client path.
- Cross-check backend targets against the catalog service cards + `nginx.conf`.

## Procedure (rubric — freeze after the 3-page calibration set)

Run each step; record `{ value, evidence }` (observed) or `{ value, rationale }`
(asserted) or `Unknown { reason }`.

```
Frontend extraction checklist for <page_id>:
- [ ] 1. Route        router/index.ts: path + name + component file   [Observed]
- [ ] 2. Menu         menus.ts: title + `advanced` flag (Unknown if unlisted) [Observed]
- [ ] 3. Component     the view .vue + the major child components it composes [Observed: component path]
- [ ] 4. Stores        which src/store/*.ts modules the page + its components use [Observed]
- [ ] 5. Consumes      every backend call the page makes, via its components AND stores:
                         axios/back_axios/fetch base URLs, mavlink2rest commands,
                         websocket/http-stream endpoints. Map each -> a cataloged
                         ServiceId + endpoint + purpose. External URLs -> service
                         "external". [Observed: the call site file:line]
- [ ] 6. Client state  domain state kept in the browser (store fields, singleton
                         classes like the Calibrator, computed-from-params getters).
                         Classify ownership:
                           frontend_owned = lives ONLY in the client (ephemeral wizard
                             progress, derived state, not persisted to any backend);
                           shared         = cached from a backend but mutated/derived
                             client-side (e.g. autopilot parameters + derived "is
                             calibrated?");
                           backend_owned  = the client only mirrors backend state.
                         [Asserted: rationale citing the store/component]
- [ ] 7. Frontend features  capabilities IMPLEMENTED client-side with no single
                         backend capability (calibration sequences, motor detection,
                         parameter editing UX, health derivation). Name each as a
                         snake_case verb (e.g. calibrate_accelerometer,
                         calibrate_compass, detect_motor_directions,
                         edit_autopilot_parameters). [Asserted: rationale citing the component/lib]
```

Steps 1-5 are OBSERVED (need `file:line`). Steps 6-7 are ASSERTED (need rationale).

## Frozen conventions (calibrated on vehicle_setup / video_manager / disk)

The rubric was frozen after 3 exemplar pages spanning the complexity range (a
calibration hub, a streaming manager, a simple read-only tool). The `Page` type
held unchanged across all three. Apply these conventions verbatim when scaling:

- **Edge trigger.** Only calls the page (or its child components) actually triggers
  become `consumes` edges. Global background store fetches the page merely *reads*
  from (e.g. `ping`, `system`, `beacon`, `customization` stores polling on their own
  timers) are NOT edges — note them in a rationale/`notes` and move on. (A typed
  `trigger` field is deferred to the coupling-analysis milestone.)
- **Store naming.** Use the Vuex *module* name (the `@Module({name})`), not the file
  name, in `stores` and `ClientState.store` (e.g. module `autopilot` lives in
  `store/autopilot.ts`; module `system` in `store/system-information.ts`).
- **Component anchor.** `component` is the single view file (`views/<X>.vue`). The
  child-component tree is not modeled as a field; cite child components inside the
  relevant `consumes`/`client_state`/`frontend_features` evidence/rationale instead.
- **Capability namespace.** `frontend_features` reuse the shared `CapabilityId` verb
  space (a capability is the same concept wherever implemented). The frontend-vs-
  backend origin is disambiguated later by the Page itself; the feature-merge step
  resolves each feature's `Origin::{BackendService, FrontendPage}`. Do NOT invent a
  separate id type.
- **Backend-driven pages.** A page that only displays/relays backend data (like
  `disk`) gets `frontend_features: AssertedSet::established(&[])` with a rationale
  saying so. Client-side sorting / unit math / percentage display is derived
  `client_state` (Shared), NOT a feature.
- **Param plane.** Autopilot parameter read/write is via `mavlink2rest` (MAVLink
  PARAM protocol through `libs/MAVLink2Rest`), not a REST param service.

## Output contract

`catalog/src/pages/<id>.rs` exposing `pub const PAGE: Page`, registered in
`pages/mod.rs`. `route`/`name`/`component`/`menu_title`/`advanced_only`/`stores`/
`consumes` use `Observed`/`ObservedSet` with `Evidence`; `frontend_features`/
`client_state` use `AssertedSet` with rationale. Unset -> `Unknown{reason}`.

## Done-criteria (gate)

- Every observed field resolves to a real `core/frontend/src/...:LINE`.
- Every `consumes.service` is a cataloged service id or `"external"`; `Catalog::validate()` + `drift` + `bash gate.sh` pass.
- Client-state ownership is classified for every held state; no state left implicit.
- `frontend_features` are genuinely client-implemented (not backend passthrough).

## Forbidden

- Inventing routes, calls, or features not present in `core/frontend/src`.
- Recording a `frontend_feature` for something that is just a call to a backend route (that is a `consumes` edge).
- Setting observed fields from docs or assumption instead of the actual frontend source.
- Editing backend service cards, journeys, or other pages in the same task.
