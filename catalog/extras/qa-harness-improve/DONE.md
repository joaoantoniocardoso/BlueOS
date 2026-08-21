# QA harness improvement — DONE

Phases ACCEPTed: H0 crate types, H1 journey annotate, H2 runner/DutProfile/ghost, H3 effect-read, H4 UI oracle, H5 body_kind, H6 ratchet.

H7 status: RAN on 192.168.0.177 (not skipped). Waves: smoke (pass=26 fail=0 skip=64), mutating-smoke (pass=83 fail=39 skip=18), ui (pass=6 fail=5 skip=0). Reports: `catalog/extras/qa-harness-improve/reports/192.168.0.177/`. Fails cataloged in `catalog/extras/qa-harness-improve/FINDINGS.md` (12 rows). Ratchet `cargo run -q --bin harness_ratchet` exit 0.

Remaining Unknowns/HarnessGaps (ratchet baseline, not worsening):
- unknown_blast_radius=1 (RunHostCommand)
- unknown_body_kind=60
- unprobed_failure_modes=45
- mutating_smoke_missing_effect_read=60
- client_orchestrated_missing_ui_plan=17
- open_harness_gap=24

next_steps: STOP
