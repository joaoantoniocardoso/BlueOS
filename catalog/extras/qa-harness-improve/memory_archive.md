# Archive — QA-HARNESS-IMPROVE

## Iteration 0 — bootstrap
- **Hypothesis**: cold start; H0 crate types first
- **Agent tasks**: none yet
- **Decision**: wait for orchestrator chat with `QAHARNESS CONTINUE`

## Iteration 1 — H0 Crate Engineer
- **Hypothesis**: H0 types + validate stubs can land without H1 annotations
- **Agent tasks**: composer-2.5 Crate Engineer (H0 only)
- **Raw results**: PLAN found at ~/.cursor/plans/QA harness improvement-ccf7f122.plan.md. Types present (BlastRadius, ConflictKind, DutProfile, body_kind, effect_read, derive_oracle_class, validate stubs). cargo fmt/clippy/test PASS. gate.sh FAIL pre-existing feature-traces drift. READY_FOR_QA yes.
- **Decision**: spawn fresh Opus-5 QA Reviewer; no H1 until ACCEPT

## Iteration 2 — H0 QA ACCEPT
- **Hypothesis**: H0 slice matches plan invariants
- **Agent tasks**: Opus-5 QA Reviewer (fresh)
- **Raw results**: VERDICT ACCEPT. fmt/clippy/test PASS. gate.sh fail pre-existing. Invariants HOLD. OVERBUILD none for H0. Notes: fake BLAST_RADIUS_UNKNOWN provenance until H1; H2 skip/sacrificial; H4 derive_oracle_class extra trigger; qa-1.4 file entanglement.
- **Decision**: H0 merged (no commit). Extract H1 worker split, then parallel annotators. No H2 until H1 ACCEPT.

## Iteration 3 — H1 split + spawn batch 1–3
- **Hypothesis**: H1 is BlastRadius + optional Hardware/NetworkResource per journey module; 10 disjoint batches
- **Agent tasks**: composer-2.5 extract H1 split; then spawn h1-ardupilot, h1-strand-risk, h1-commander
- **Raw results**: 10 batches listed; shared files off-limits; first 3 in flight
- **Decision**: annotate first 3 modules in parallel; QA after they return

## Iteration 4 — H1 annotators 1–3 returned
- **Hypothesis**: first 3 modules can be QA'd independently
- **Agent tasks**: composer-2.5 h1-ardupilot, h1-strand-risk, h1-commander
- **Raw results**: all READY_FOR_QA. ardupilot 9 journeys (4 Destructive firmware). strand-risk 12 none Destructive. commander reboot/shutdown/eeprom/reset Destructive; RunHostCommand Unknown. Possible H0 all-Unknown stub test fail.
- **Decision**: spawn fresh Opus-5 QA with per-batch verdicts

## Iteration 5 — H1 QA batches 1–3
- **Hypothesis**: per-batch merge; Hardware on mutating journeys is the ardupilot defect
- **Agent tasks**: Opus-5 QA
- **Raw results**: strand ACCEPT, commander ACCEPT, ardupilot BOUNCE. Root cause: Precondition::Hardware → Automatable::Hardware → dropped from Tier-2 (62→53) and journey_http. BlastRadius values themselves OK. SHARED_VALIDATE_STUB fail: relax all-Unknown now.
- **Decision**: bounce ardupilot (drop Hardware, keep Other). Crate-side validate stub relax. Spawn h1-customization. No eligibility rewrite in H1.

## Iteration 6 — H1 bounce+stub+customization returned
- **Hypothesis**: dropping Hardware restores Tier-2; stub relax unblocks cargo test
- **Agent tasks**: composer-2.5 ardupilot bounce, validate.rs stub, h1-customization
- **Raw results**: ardupilot Hardware gone, radii kept. validate ledger counts Unknowns. customization 8 Reversible. Stub author cargo test blocked mid-flight by customization compile (should be fixed now).
- **Decision**: fresh Opus-5 QA on these three

## Iteration 7 — H1 QA ardupilot/stub/customization
- **Hypothesis**: annotation batches clean; stub overbuilt
- **Agent tasks**: Opus-5 QA
- **Raw results**: ardupilot ACCEPT, customization ACCEPT, validate-stub BOUNCE (dead_code clippy, tautology, inverse ratchet). cargo test 256 pass. mutating_smoke 62 Http restored.
- **Decision**: bounce validate.rs; spawn h1-wifi + h1-kraken-vc

## Iteration 8 — stub bounce + wifi + kraken-vc returned
- **Hypothesis**: smallest stub + wifi NetworkResource does not drop Tier-2
- **Agent tasks**: composer-2.5 validate-stub bounce, h1-wifi, h1-kraken-vc
- **Raw results**: stub READY. wifi 12 annotated, RF unchanged, KnownWifiNetwork/HotspotCapable. kraken-vc Update/Switch Destructive, bootstrap Disruptive.
- **Decision**: fresh Opus-5 QA

## Iteration 9 — H1 QA stub/wifi/kraken ACCEPT
- **Hypothesis**: all three match H1 invariants
- **Agent tasks**: Opus-5 QA
- **Raw results**: all ACCEPT. NetworkResource ≠ Automatable flip. tier2 62/62. wifi_rf.rs clean. H2: wifi fixtures needed for live.
- **Decision**: spawn h1-net-tools, h1-storage, h1-frontend-mavlink

## Iteration 10 — H1 net-tools/storage/frontend-mavlink returned
- **Hypothesis**: remaining service modules annotatable without Hardware
- **Agent tasks**: composer-2.5 three annotators
- **Raw results**: all READY_FOR_QA. iperf3 empty. DetectMotorDirections Disruptive judgment. No new Hardware.
- **Decision**: fresh Opus-5 QA

## Iteration 11 — H1 QA net/storage/frontend ACCEPT
- **Hypothesis**: three batches match H1
- **Agent tasks**: Opus-5 QA
- **Raw results**: all ACCEPT. iperf3 empty OK. ApplyParameterFile Disruptive vs EEPROM wording is a note.
- **Decision**: spawn remainder split a/b/c

## Iteration 12 — H1 remainder annotators returned
- **Hypothesis**: leftover modules annotatable; empty arrays stay empty
- **Agent tasks**: composer-2.5 remainder-a/b/c
- **Raw results**: all READY. recorder.rs + user_terminal.rs empty. AccessWebTerminal Disruptive. No new Hardware/JourneyId.
- **Decision**: fresh Opus-5 QA remainder then H2

## Iteration 13 — H1 remainder QA ACCEPT; H1 complete (session interrupted mid H2 extract)
- **Hypothesis**: remainder matches H1; campaign advances to H2 runner
- **Agent tasks**: Opus-5 QA remainder; H2 extract spawn interrupted
- **Raw results**: A/B/C ACCEPT. 1 remaining Unknown blast (RunHostCommand). tier2 62.
- **Decision**: CONTINUE resumes H2 extract; no H3 until H2 ACCEPT

## Iteration 14 — H2 split extracted; spawn implementers
- **Hypothesis**: skip mapping and ghost wiring are disjoint (runner.rs vs journey_http.rs)
- **Agent tasks**: extract H2; spawn h2-runner-skip + h2-journey-http-ghost
- **Raw results**: sacrificial=177 only; strand IDs listed; ghost GET advisory on --smoke, never 2.2
- **Decision**: implement both; QA after return

## Iteration 15 — H2 implementers returned
- **Hypothesis**: skip+ghost compile together after parallel edits
- **Agent tasks**: composer-2.5 h2-runner-skip, h2-journey-http-ghost
- **Raw results**: runner skip+tests 258 pass. journey_http skip-before-run + ghost; dry-run 81 Http. Extra files: report.rs, lib.rs.
- **Decision**: fresh Opus-5 QA

## Iteration 16 — H2 QA: runner ACCEPT, ghost BOUNCE
- **Hypothesis**: skip mapping correct; ghost missing tests + two dry-run bugs
- **Agent tasks**: Opus-5 QA
- **Raw results**: invariants HOLD. Bounce: (1) no ghost classifier tests (2) dry-run ghost listing unreachable (3) dry-run 2.2 skip double-count
- **Decision**: new ghost author; do not edit runner.rs

## Iteration 17 — H2 ghost bounce returned
- **Hypothesis**: tests + dead-code delete + always-continue fix the bounce list
- **Agent tasks**: composer-2.5 h2-ghost-bounce
- **Raw results**: 3 bin tests pass; dry-run skipped=118 default and 2.2; runner.rs untouched
- **Decision**: fresh Opus-5 QA

## Iteration 18 — H2 ghost QA BOUNCE clippy
- **Hypothesis**: ghost semantics OK; test module placement fails gate clippy
- **Agent tasks**: Opus-5 QA
- **Raw results**: BOUNCE items_after_test_module. Tests/invariants/double-count HOLD. Move mod tests to EOF.
- **Decision**: new author; no H3

## Iteration 19 — H2 ghost clippy bounce returned
- **Hypothesis**: EOF tests + drop _journey green clippy
- **Agent tasks**: composer-2.5 h2-ghost-clippy
- **Raw results**: clippy --all-targets -D warnings pass; bin 3/3; lib 258
- **Decision**: fresh Opus-5 QA

## Iteration 20 — H2 ACCEPT complete
- **Hypothesis**: clippy bounce sufficient
- **Agent tasks**: Opus-5 QA
- **Raw results**: VERDICT_GHOST ACCEPT. Invariants HOLD. gate.sh clippy green; feature-traces drift only.
- **Decision**: H2 merged. Extract H3 effect-read. No H4 until H3 ACCEPT.

## Iteration 21 — H3 split; spawn annotator
- **Hypothesis**: one author on mutating_smoke.rs avoids 3-way same-file merge
- **Agent tasks**: extract H3; spawn h3-effect-read annotator (A+B+C ids)
- **Raw results**: Some(EffectReadRef) only for real GET steps; else None + HarnessGap notes. Runner after ACCEPT.
- **Decision**: annotate then QA; no runner until ACCEPT

## Iteration 22 — H3 annotator returned
- **Hypothesis**: no in-journey GET → None is correct; do not invent endpoints
- **Agent tasks**: composer-2.5 h3-effect-read-annotate
- **Raw results**: 2 Some (/status wifi); A/B all HarnessGap; wifi RF untouched; tests 258
- **Decision**: fresh Opus-5 QA

## Iteration 23 — H3 annotate ACCEPT; spawn runner
- **Hypothesis**: plan permits None+HarnessGap; runner next
- **Agent tasks**: Opus-5 QA annotate
- **Raw results**: ACCEPT. step_index 0 is GET /status on both Some ids. validate counts None, returns empty.
- **Decision**: spawn H3 runner engineer

## Iteration 24 — H3 runner returned
- **Hypothesis**: unit tests can prove order and EffectNotApplied without live DUT
- **Agent tasks**: composer-2.5 h3-effect-read-runner
- **Raw results**: runner.rs + journey_http.rs; 264 tests; clippy pass; RF skips HTTP effect-read
- **Decision**: fresh Opus-5 QA

## Iteration 25 — H3 runner QA BOUNCE
- **Hypothesis**: tests green but don't prove the wired path
- **Agent tasks**: Opus-5 QA
- **Raw results**: BOUNCE 5: contains: always EffectNotApplied; only Some are RF-skipped; phase helper unused by bin; stale test doesn't call effect_read_after; probe inflates passed
- **Decision**: new runner author

## Iteration 26 — H3 runner bounce returned
- **Hypothesis**: status-only probes + lib phases + after_observed + synthetic fixture close bounce
- **Agent tasks**: composer-2.5 h3-runner-bounce
- **Raw results**: 265 tests; clippy pass; READY_FOR_QA
- **Decision**: fresh Opus-5 QA

## Iteration 27 — H3 runner ACCEPT; H3 complete
- **Hypothesis**: bounce list actually fixed
- **Agent tasks**: Opus-5 QA
- **Raw results**: ACCEPT. 265 tests, clippy, fmt. Live Some still RF-only (annotate ACCEPT).
- **Decision**: extract H4 UI oracle. No H5 until H4 ACCEPT.

## Iteration 28 — H4 split; spawn classifier
- **Hypothesis**: 3rd ClientOrchestrated trigger must be confirmed or dropped before ui_plan authors
- **Agent tasks**: extract H4; spawn h4-oracle-classifier
- **Raw results**: serial classifier then two ui.rs clusters. Camera plans qa-1.4 leave intact.
- **Decision**: classifier then QA; no parallel ui authors yet

## Iteration 29 — H4 classifier returned
- **Hypothesis**: 3rd trigger is duplicate; DROP
- **Agent tasks**: composer-2.5 h4-oracle-classifier
- **Raw results**: DROP; 30 orchestrated / 47 composed / 23 passthrough; 17 missing ui_plan; H4_ORACLE_COUNTS.md
- **Decision**: fresh Opus-5 QA

## Iteration 30 — H4 classifier ACCEPT; spawn ui.rs author
- **Hypothesis**: DROP aligns with plan; same-file ui.rs clusters serialized as one author
- **Agent tasks**: Opus-5 QA classifier
- **Raw results**: ACCEPT. Counts 30/47/23 match. 17 missing list matches. Trigger3 DROP correct.
- **Decision**: spawn h4-ui-plan-author; no H5 until H4 ACCEPT

## Iteration 31 — H4 ui-plan author returned
- **Hypothesis**: extensions+version plans + typed skips for VehicleSetup over-classify
- **Agent tasks**: composer-2.5 h4-ui-plan-author
- **Raw results**: 12 planned; typed-skip firmware/reboot/shutdown/delete/bootstrap; camera untouched; 274 tests
- **Decision**: fresh Opus-5 QA

## Iteration 32 — H4 ui-plan ACCEPT; H4 complete
- **Hypothesis**: plans are landmarks not HTTP clones
- **Agent tasks**: Opus-5 QA
- **Raw results**: ACCEPT. 274 tests, clippy, fmt, journey_matrix blank-both 0. Camera provenance via 04:30 live artifacts.
- **Decision**: extract H5 body_kind. No H6 until H5 ACCEPT.

## Iteration 33 — H5 split; spawn 3 fillers
- **Hypothesis**: disjoint journey modules can fill body_kind in parallel; runner in remainder batch
- **Agent tasks**: extract H5; spawn h5-network, h5-vehicle, h5-remainder-runner
- **Raw results**: evidence-only labeling; Unknown OK; ProductMissingReject in batch 3
- **Decision**: fill then QA

## Iteration 34 — H5 fillers returned
- **Hypothesis**: merged tree compiles; labels evidence-only
- **Agent tasks**: composer-2.5 three fillers
- **Raw results**: network labeled helper/pardal/bridget/kraken/vc; vehicle labeled some Payload/Empty; remainder disk/nginx/linux2rest/recorder + ProductMissingReject. Parallel signature risk.
- **Decision**: fresh Opus-5 QA

## Iteration 35 — H5 QA: network+vehicle ACCEPT, remainder BOUNCE
- **Hypothesis**: labels evidence-only; remainder wiring dead
- **Agent tasks**: Opus-5 QA
- **Raw results**: ProductMissingReject never reaches report (run_http_step drops conflict). Tests/clippy green. Feature-traces drift + GH_RATE_LIMIT flake not H5.
- **Decision**: bounce remainder journey_http wiring only

## Iteration 36 — H5 remainder bounce returned
- **Hypothesis**: detailed run + push conflict + wave-path test close bounce
- **Agent tasks**: composer-2.5 h5-remainder-bounce
- **Raw results**: finalize_http_step_run; bin test 4; lib 275; clippy pass
- **Decision**: fresh Opus-5 QA

## Iteration 37 — H5 remainder ACCEPT; H5 complete
- **Hypothesis**: wave path now attaches ProductMissingReject
- **Agent tasks**: Opus-5 QA
- **Raw results**: ACCEPT. lib 275, bin 4, clippy, fmt. Wave loop uses run_http_step_detailed.
- **Decision**: extract H6 ratchet. No H7 until H6 ACCEPT.

## Iteration 38 — H6 split; spawn engineer
- **Hypothesis**: one author for metrics+baseline+gate avoids shared-file merge
- **Agent tasks**: extract H6; spawn h6-ratchet
- **Raw results**: fail only worsening; baseline in extras/qa-harness-improve/ratchet_baseline.json
- **Decision**: implement then QA

## Iteration 39 — H6 engineer returned
- **Hypothesis**: snapshot+compare+gate tests prove worsening-only
- **Agent tasks**: composer-2.5 h6-ratchet
- **Raw results**: baseline counts listed; bin tests for effect_read and ui_plan worsen; validate() still green
- **Decision**: fresh Opus-5 QA

## Iteration 40 — H6 ACCEPT; spawn H7
- **Hypothesis**: ratchet matches plan; campaign advances to live 177 or typed skip
- **Agent tasks**: Opus-5 QA H6
- **Raw results**: ACCEPT. Fail only > baseline. validate stubs still empty. Synthetic worsen exit 1.
- **Decision**: H7 live 177; skip if unreachable; then DONE

## Iteration 41 — H7 RAN; campaign DONE
- **Hypothesis**: reports + FINDINGS + ratchet green meet Done protocol
- **Agent tasks**: H7 closer (smoke/mutating/ui on 177); ratchet verify
- **Raw results**: STATUS RAN. smoke 26/0/64; mutating 83/39/18; ui 6/5/0. FINDINGS 12 rows. harness_ratchet exit 0.
- **Decision**: write DONE.md; memory next_steps STOP. HALT. No further agents.





























## Iteration 13 — H1 remainder QA ACCEPT; H1 complete
- **Hypothesis**: remainder matches H1; campaign advances to H2 runner
- **Agent tasks**: Opus-5 QA remainder
- **Raw results**: A/B/C ACCEPT. 1 remaining Unknown blast (RunHostCommand). tier2 62. One pre-existing shell retry flake, not remainder.
- **Decision**: extract H2 from plan; no H3 until H2 ACCEPT


