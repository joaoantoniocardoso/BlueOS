# G6 Track A — promotion decision (Opus-5)

**SKIP_WITH_REASON — zero new `JourneyId`s promoted.**

Scope: only the five candidates listed in `catalog/extras/qa-1.4-full/PLAN.md:44-48`.
Template: `RejectInvalidWifiCredentials` (`catalog/src/journeys/wifi.rs:336-378`).
Gate applied: `PLAN.md:33-38` criteria 1-4, all four required.

## Candidate ledger

| Candidate | Probe | Live capture | Verdict |
|---|---|---|---|
| Reject unconfirmed dangerous op | NP-02/03/05 | 400 Pass ×4 DUTs | reject — criterion 1 |
| Reject invalid theme color | NP-12/13 | 405 Fail ×4 DUTs | reject — `not_on_1.4-dev` |
| Reject disallowed upload suffix | NP-15/16/17 | none (never ran) | reject — `not_on_1.4-dev` |
| Reject missing bag path | NP-24 | 400 Pass ×4 DUTs | reject — criterion 1 |
| Reject host command without ack | NP-01 | 400 Pass ×4 DUTs | reject — criterion 1 |

Capture evidence: `catalog/extras/qa-1.4-full/reports/{192.168.0.177,192.168.0.87,192.168.2.2,192.168.0.124}/w2-negative.{json,log}`
(177 log lines 6,7,8,10,21), summarised in `W2_SUMMARY.md:25-37`.

## Why the three captured candidates still fail

Criterion 1 requires the operator to see a **named failure UX**. The wifi template earns
its journey because the connection dialog surfaces a check-password error to the operator
(`core/frontend/src/.../ConnectionDialog` at the provenance line cited in `wifi.rs:339,366`).
The commander and bag rejections have no such surface:

- **Commander gate (NP-01/02/03/05).** Every frontend call site hardcodes the ack:
  `core/frontend/src/store/commander.ts:46,68,92,160,205,226`,
  `core/frontend/src/views/SettingsView.vue:672,704,752`,
  `core/frontend/src/utils/update_time.ts:13`. A repo-wide search finds **no** call site
  sending `i_know_what_i_am_doing=false`, so the 400 raised at
  `core/services/commander/main.py:52` is unreachable from the UI. It is an API-level
  guard, correctly covered as a Track B probe, not an operator journey.
- **Missing bag path (NP-24).** `core/frontend/src/store/bag.ts:69-70` catches the 400 and
  returns `null` — the documented "path doesn't exist" signal (`bag.ts:57-60`). The 400 from
  `core/services/bag_of_holding/main.py:102` is normal internal control flow; no notifier
  fires, so there is no named failure UX to model.

Promoting any of these would assert an operator-visible failure contract that the frontend
provably cannot produce — the exact quality drift the Track A gate exists to prevent.

## Why the other two are environment-blocked

`change_ui_theme_color` PUT and the customization upload routes are absent on the pinned
1.4-dev core (`W2_SUMMARY.md:25-33`, finding F-022): NP-12/13 returned **405**, and
NP-15/16/17 never executed on any DUT. Their status is `not_on_1.4-dev`, so there is no live
capture to pin and criterion 3 (contract, not environment artefact) cannot be evaluated.
Revisit on a 1.5.0+ core.

## Consequence

No change to `catalog/src/**`; `JourneyId::ALL`, presence generation, and coverage mappings
are untouched. Per `ORCHESTRATOR_PROMPT.xml:86`, G6 skip-with-reason satisfies the done
criteria. Track B keeps all five as negative probes in `NEGATIVE_PROBES.md`.

Follow-up for the ledger, not for G6: NP-09 captured **500** (`w2-negative.log:14`) where a
framework rejection was expected — commander's custom handler masking a 422 is a product
finding worth a finding id, and is further reason not to model commander rejections as
contract journeys yet.
