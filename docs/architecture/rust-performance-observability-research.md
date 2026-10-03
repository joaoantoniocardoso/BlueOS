# Rust performance and observability: research note

Status: research input for a grill session and later decision entries. It decides nothing. Written 2026-10-02
against rustc 1.99.0 and the repository state at commit `714d8d589`. Versions and dates were read from
crates.io and GitHub on that day.

## Summary

Recommended default per area, one line each:

1. **Benchmarking:** Criterion for wall-clock microbenchmarks run locally and on a Pi; Gungraun (the renamed
   iai-callgrind) for instruction-count benchmarks in CI; hyperfine for whole-binary measurements such as start-up
   time. Defer a tracking service (Bencher or CodSpeed) until benchmarks exist.
2. **Regression detection and property testing:** gate CI on Gungraun soft limits (instruction counts are stable
   on shared runners) and keep wall-clock statistics report-only; adopt proptest, plus `proptest-state-machine`
   for Domains, for property-based tests in test layers L1 and L2.
3. **Profiling:** `perf` with Hotspot for CPU on the Pi (samply fails on the Pi 5's 16 KiB pages), Callgrind through Gungraun for deterministic
   counts, heaptrack for heap (on a dynamically linked glibc build), tokio-console only in a development build.
4. **Profiling build:** a `[profile.profiling]` that inherits `release` with `debug = "line-tables-only"` and
   `strip = false`, plus `-C force-frame-pointers=yes` for armv7 and x86_64 passed through `RUSTFLAGS`
   (aarch64 Linux already defaults to non-leaf frame pointers, and the shipped standard library is built with
   them). The v0 symbol mangling the perf book suggests is the default since Rust 1.97.
5. **Compilation time:** measure first with `cargo build --timings`; fix the workspace-wide build the user saw
   (a virtual workspace builds every member unless `-p` is given) before switching linkers; evaluate mold only
   for the host development loop, where rust-lld is already the default.
6. **Binary size:** keep `panic = "unwind"` (D-29 recovery depends on it), measure `codegen-units = 1`,
   `lto = "fat"` and `opt-level = "s"` one at a time with `cargo bloat`; do not adopt UPX without the
   measurement plan in that section, because UPX-packed processes stop sharing the multicall binary's pages.
7. **Service observability:** keep `tracing` as the one instrumentation API, add the `metrics` facade with a
   recorder that publishes over Zenoh (next to the D-12 standard keys) instead of an HTTP scrape endpoint, and
   expose the Zenoh router's own OpenMetrics output; do not adopt jamesgober/rust-benchmark.

Open questions to grill the user on:

1. "Hypothesis testing" is ambiguous. Did you mean statistical significance of benchmark changes (regression
   detection), property-based testing, or both? This note covers both.
2. What is the performance budget: target boards (Pi 3, Pi 4, Pi 5, and is armv7 still a shipping target for
   new Rust services), maximum resident memory per service, start-up time, and CPU headroom?
3. May CI depend on a hosted service (Bencher Cloud, CodSpeed) that stores benchmark history outside GitHub, or
   must everything stay in GitHub Actions artifacts or a self-hosted server?
4. Should performance gates ever fail a pull request, or stay report-only like `cargo bloat` in D-30? If they
   gate, on which metric (instruction counts only, or also binary size)?
5. Are free GitHub `ubuntu-24.04-arm` runners acceptable for aarch64 benchmarks, given they are 4-core Arm VMs and
   not Pi hardware?
6. Is binary size a real constraint (flash wear, update download size, SD card space) or a nice-to-have? The answer
   decides whether `opt-level = "s"` and UPX are even in scope.
7. Which processes run from the `blueos` multicall binary at the same time on a vehicle, now and after more
   services migrate? This decides whether page sharing (and so UPX) matters.
8. Who consumes service metrics: the BlueOS frontend, the Recorder (MCAP files), a user's Prometheus or Grafana,
   or a future fleet backend? This decides between Zenoh-native metrics and OpenTelemetry or Prometheus exporters.
9. Should profiling tools (perf, samply, heaptrack, valgrind) ship in the core image, in an optional debug
   image, or be installed by hand when needed?
10. Is it acceptable to pin the Rust toolchain (a `rust-toolchain.toml`) so benchmark history is not confounded by
    compiler upgrades? CI currently floats on `stable`.
11. Is a nightly toolchain acceptable for size experiments (`build-std`, `-Zlocation-detail=none`) and for the
    `-Z self-profile` compile-time measurements, given D-30 already has a pinned nightly job?

## Repo baseline

What the repository has today, with file paths.

- **Workspace.** `core/Cargo.toml` is a virtual workspace (no root package) with `resolver = "2"`, edition 2024,
  and 23 members. There is no `default-members`, no `[workspace.lints]` table yet (D-30 describes it as the
  target), no Cargo config file under `core/`, and no `rust-toolchain.toml`.
- **Profiles** (`core/Cargo.toml`):
  - `[profile.dev] debug = "line-tables-only"` and `[profile.dev.package."*"] debug = false`.
  - `[profile.release] lto = "thin"`, `panic = "unwind"`, `strip = true`. Everything else is the Cargo default:
    `opt-level = 3`, `codegen-units = 16`, `debug = false`
    ([Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)).
  - No custom profile (no `profiling`, no `bench` override).
- **Multicall binary.** `core/app/blueos/Cargo.toml` defines the `blueos` binary with features `example` and
  `recorder`, and `default = ["recorder"]`, so `--features=recorder` repeats the default. `core/Dockerfile`
  installs it as `/usr/bin/blueos` with a `recorder` symlink, in the last layer (D-16).
- **Release build and linker.** `core/build_cross.sh` runs
  `cross build --release --locked --features recorder -p blueos --target <triple>` for
  `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and `armv7-unknown-linux-musleabihf`, each with its own
  `CARGO_TARGET_DIR`. The Cross config passes `RUSTFLAGS` through to the container. In rustc 1.99.0 the target
  specifications of `aarch64-unknown-linux-musl` and `armv7-unknown-linux-musleabihf` use the `gnu-cc` linker
  flavor, and so does `x86_64-unknown-linux-musl` in the local 1.100 nightly (read locally with
  `rustc -Z unstable-options --print target-spec-json`), so the linker is whatever the cross image's GCC driver
  invokes; the cross Dockerfile sets `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER` to the
  `aarch64-linux-musl-gcc` wrapper
  ([cross Dockerfile](https://github.com/cross-rs/cross/blob/main/docker/Dockerfile.aarch64-unknown-linux-musl)).
  CI installs cross with `cargo install cross --locked`, which resolves to cross 0.2.5, released 2023-02-04
  ([cross releases](https://github.com/cross-rs/cross/releases)). D-16 records release binaries of about 14 MB per
  target.
- **Host linker.** Local and CI host builds on `x86_64-unknown-linux-gnu` link with rust-lld by default since
  Rust 1.90 ([Rust blog](https://blog.rust-lang.org/2025/09/01/rust-lld-on-1.90.0-stable/),
  [rust#140525](https://github.com/rust-lang/rust/pull/140525)). The local 1.99.0 target specifications also show
  `gnu-lld-cc` for `aarch64-unknown-linux-gnu` and `armv7-unknown-linux-gnueabihf`; the release note for that
  change was not found (unverified which release introduced it).
- **Frame pointers in the targets.** The 1.99.0 target specifications show `frame-pointer: non-leaf` for both
  aarch64 Linux targets and no default for armv7 and x86_64, matching
  [rust#140832](https://github.com/rust-lang/rust/pull/140832) (Rust 1.89).
- **Tracing stack.** `tracing` 0.1.44, `tracing-subscriber` 0.3.23 with `env-filter`, and `tracing-log` 0.2
  (`core/Cargo.lock`). `core/libs/adapters/logging` adds a Zenoh layer that publishes each record as a foxglove
  `Log` message on the service's `log` key (D-13). There is no metrics crate, no OpenTelemetry, and no
  tokio-console instrumentation.
- **Zenoh.** `zenoh = "=1.9.0"` with `default-features = false` and features `shared-memory` and `transport_tcp`
  (no `stats` feature).
- **Gates.** `.hooks/pre-push` calls `run_rust_checks` in `.hooks/lib/rust_checks.sh`: the style-checklist drift
  check, `cargo fmt`, `cargo clippy --workspace --all-targets --all-features`, `cargo test --workspace`,
  `cargo deny check bans`, the crate-folder rules, `no_std` and wasm32 checks of logic crates, and
  `cargo semver-checks` when installed. `core/deny.toml` sets `exclude-dev = true`, so dev-dependencies such as
  benchmark harnesses are outside cargo-deny's graph.
- **CI.** `.github/workflows/test-and-deploy.yml` has two Rust jobs: `rust-tests` (the same checks, then
  `cargo test`) and `rust-cross-build` (a matrix of the three musl targets that prints `ls -lah` of the binary
  and uploads it). Both run on `ubuntu-latest` with `Swatinem/rust-cache` and `dtolnay/rust-toolchain@stable`.
  D-30 describes the target of seven Rust jobs, including a report-only `cargo bloat`; the binary size is printed
  today but not recorded or compared.
- **Constraints from existing decisions.** D-29 recovers from panics in the Inbox loop and in Tasks, which needs
  unwinding, so `panic = "abort"` is out unless D-29 changes. D-12 reserves the standard per-service keys
  (liveliness, `info`, `status`, `settings`, `jobs`, `log`), which is the natural place for a metrics key. D-30
  keeps coverage out of the pre-push hook because changing `RUSTFLAGS` invalidates the target folder; the same
  applies to frame-pointer and `tokio_unstable` flags.
- **Container.** The core container is privileged with host networking (`bootstrap/startup.json.default`).
  Docker documents that `--privileged` gives all capabilities to the container
  ([Docker run reference](https://docs.docker.com/engine/containers/run/)), which includes `CAP_PERFMON`.
- **Device under test** (read on 2026-10-02 from a Raspberry Pi 5, 4 GB, Raspberry Pi OS bookworm 64-bit, kernel
  6.6.31, running the integration image). The core container is Debian 12 with glibc 2.36, runs as root with every
  capability and `Seccomp: 0`, so `perf_event_open` succeeds inside it for software, hardware and system-wide
  events even with `perf_event_paranoid` at 2. Neither the host nor the container has `perf` installed, and the
  `linux-perf` candidate (6.12) does not match the running kernel. `CONFIG_UPROBE_EVENTS` is not set. The
  `blueos` binary is 16.2 MB, static musl, stripped, with `.eh_frame` kept; its image layer is 16.2 MB against
  66.1 MB for the Python virtual environment. The Recorder runs at 11.5 MB RSS (2.3 MB anonymous, 9.2 MB
  file-backed, 12 threads); each Python service runs at about 75 to 100 MB RSS. The root partition has 101 GB free.

## 1. Benchmarking frameworks

| Name | What it measures | Maturity, last release | Pi and aarch64 fit | CI fit |
|---|---|---|---|---|
| Criterion | Wall-clock time per iteration, with statistics | 0.8.2, 2026-02-04; active | Runs on any target | Noisy on shared runners |
| Divan | Wall-clock time, allocation counts | 0.1.21, 2025-04-10 | Runs on any target | Noisy on shared runners |
| Gungraun (was iai-callgrind) | Instruction counts, estimated cycles, cache and heap metrics (Valgrind), or perf counters | 0.20.0, 2026-09-26; active | Valgrind supports ARM64 and ARMv7 Linux | Deterministic, one run |
| hyperfine | Wall-clock time of whole commands | 1.20.0, 2025-11-18 | Runs on the Pi | Noisy on shared runners |
| tango-bench | Paired wall-clock comparison of two builds | 0.8.0, 2026-08-21 | Runs on any target | Less sensitive to noise by design |
| Bencher | History and thresholds (not a harness) | CLI v0.6.13, 2026-09-28 | Not applicable | Hosted or self-hosted |
| CodSpeed | Simulated CPU cycles or bare-metal wall time (not a harness) | 5.0.2, 2026-09-17 | Not applicable | Hosted service |

**Criterion** is the most used Rust benchmark harness (295 million downloads on
[crates.io](https://crates.io/crates/criterion)); the repository moved to the `criterion-rs` organization and had
commits on 2026-10-02 ([repository](https://github.com/criterion-rs/criterion.rs)). It runs on stable Rust and
reports statistically whether performance changed since the last run
([README](https://github.com/criterion-rs/criterion.rs/blob/master/README.md)). It supports async benchmarks
through `to_async` with an executor ([async guide](https://github.com/criterion-rs/criterion.rs/blob/master/book/src/user_guide/benchmarking_async.md)),
and its features include `async_tokio` ([Cargo.toml](https://github.com/criterion-rs/criterion.rs/blob/master/Cargo.toml)).
Its own FAQ says not to rely on its results on cloud CI such as GitHub Actions, because virtualization adds
noise its statistics cannot remove, and points to Cachegrind-based instruction counting instead
([FAQ](https://github.com/criterion-rs/criterion.rs/blob/master/book/src/faq.md)).

**Divan** has a lighter API but is still at 0.1.x, with the last release on 2025-04-10 and the last commit on
2026-07-19 ([repository](https://github.com/nvzqz/divan)); async functions are an open feature request
([divan#39](https://github.com/nvzqz/divan/issues/39)). Since the services are tokio-based outside the sans-IO
logic crates, Criterion's async support is the deciding difference.

**Gungraun** is iai-callgrind renamed: the changelog says the project was renamed with version 0.17.0 and the
crates moved from `iai-callgrind` to `gungraun`
([CHANGELOG](https://github.com/gungraun/gungraun/blob/main/CHANGELOG.md)), and the old GitHub repository
redirects to `gungraun/gungraun`. It runs each benchmark once under Callgrind, Cachegrind, DHAT or Linux perf, and
its README states the measurements stay comparable in virtualized CI; it also states it cannot give wall-clock
times ([README](https://github.com/gungraun/gungraun/blob/main/README.md)). It needs Valgrind installed where the
benchmark runs, or perf 5.9 or later for perf-only benchmarks
([prerequisites](https://gungraun.github.io/gungraun/latest/html/installation/prerequisites.html)). Valgrind
supports ARM64 Linux (ARMv8) and ARM Linux from ARMv7
([Valgrind platforms](https://valgrind.org/info/platforms.html)), so the same benchmarks can run natively on an
`ubuntu-24.04-arm` runner, which GitHub provides free for public repositories
([GitHub changelog](https://github.blog/changelog/2025-08-07-arm64-hosted-runners-for-public-repositories-are-now-generally-available/),
[runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)). Instruction
counts depend on the architecture, so x86_64 counts are only a proxy for the Pi.

**hyperfine** benchmarks whole commands with warm-up runs, preparation commands (for example dropping the page
cache), outlier detection and JSON export ([README](https://github.com/sharkdp/hyperfine/blob/master/README.md)).
It is the right tool for start-up time of the `blueos` binary and for the UPX measurement in section 6.

**tango-bench** measures two builds in alternation (paired benchmarking) and claims higher sensitivity than
measuring them one after the other; it supports tokio ([README](https://github.com/bazhenov/tango/blob/master/README.md)).
It is a credible option for wall-clock comparison of a pull request against its base, but it has a much smaller
user base (918 thousand downloads, [crates.io](https://crates.io/crates/tango-bench)).

**Bencher** is a continuous benchmarking service that stores results per branch and testbed and raises alerts
from statistical thresholds; it is open source and self-hostable
([self-hosting](https://bencher.dev/docs/tutorial/docker/)); the code is Apache-2.0 or MIT except directories and
features named "plus" ([LICENSE](https://github.com/bencherdev/bencher/blob/devel/LICENSE.md)). The perf book
lists it for continuous benchmarking on CI ([perf book, Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)).

**CodSpeed** provides compatibility crates for Criterion, Divan and the `bencher` crate API
([codspeed-rust](https://github.com/CodSpeedHQ/codspeed-rust)), a CPU-simulation instrument that runs a benchmark
once and estimates cycles including cache behaviour ([CPU simulation](https://codspeed.io/docs/instruments/cpu)),
and wall-time measurement on bare-metal runners it manages ([wall time](https://codspeed.io/docs/instruments/walltime)).
Its pricing page lists a free plan, unlimited for open-source projects ([pricing](https://codspeed.io/pricing)).
Whether its CPU simulation supports aarch64 was not found (unverified).

Rust's built-in `#[bench]` needs nightly ([perf book, Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)),
so it is not a candidate.

Recommendation. Use three tools for three questions. Criterion answers "how fast is this function on this machine"
and runs locally and on a Pi. Gungraun answers "did this pull request change the work done" in CI, on
`ubuntu-latest` and `ubuntu-24.04-arm`. hyperfine answers "how long does the binary take to start". The sans-IO
logic crates (D-03) are the cheapest place to start, because a Domain step is a pure function. Keep benchmarks out
of the pre-push hook, like coverage. A tracking service is a separate decision (open question 3); Gungraun can
already compare against a baseline saved from the base branch inside one CI job, which needs no service.

## 2. Hypothesis testing and regression detection

The phrase has two readings. Both are covered; open question 1 asks which was meant.

### 2a. Statistical significance of benchmark changes

| Name | Method | Maturity, last release | Pi and aarch64 fit | CI fit |
|---|---|---|---|---|
| Criterion statistics | Bootstrap confidence intervals and a t-test against the saved baseline | 0.8.2, 2026-02-04 | Good on the device | Noisy on shared runners |
| Gungraun limits | Soft limits (percentage) and hard limits (absolute) on instruction counts | 0.20.0, 2026-09-26 | Native on Arm runners | Deterministic |
| Bencher thresholds | Percentage, z-score, t-test, log-normal, IQR, delta IQR, static | v0.6.13, 2026-09-28 | Not applicable | Needs history storage |
| tango-bench | Paired comparison | 0.8.0, 2026-08-21 | Good on the device | Better than pointwise |

Criterion classifies outliers with a modified Tukey method, estimates time per iteration by linear regression with
bootstrap confidence intervals, and compares two runs with a t-test over bootstrap samples
([analysis](https://github.com/criterion-rs/criterion.rs/blob/master/book/src/analysis.md)). Gungraun has no
regression checks by default; soft and hard limits are opt-in, a regression fails the benchmark with exit code 3,
and comparing against a baseline also detects regressions, which its guide suggests for failing a pull request
against the main branch ([regressions](https://gungraun.github.io/gungraun/latest/html/regressions.html)). Bencher
checks each new metric against limits computed by one of seven tests over a window of historical results and can
fail the job with `--error-on-alert` ([thresholds](https://bencher.dev/docs/explanation/thresholds/)). The perf
book notes that wall time has high variance, that small memory-layout changes cause large but ephemeral
fluctuations, and that cycles or instruction counts are lower-variance alternatives
([perf book, Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)).

Recommendation. On shared runners, use instruction counts with Gungraun soft limits as the only candidate gate,
first report-only (D-30 style) until the noise floor is known. Treat Criterion's p-values as local evidence. A
wall-clock gate needs dedicated hardware (a Pi as a self-hosted runner, or CodSpeed's bare-metal runners); open
question 4 decides whether that is wanted.

### 2b. Property-based testing

| Name | What it does | Maturity, last release | Pi and aarch64 fit | CI fit |
|---|---|---|---|---|
| proptest | Strategy-based generation with shrinking; state-machine testing in a companion crate | 1.11.0, 2026-03-24; active | Host tests only | Good |
| quickcheck | Type-based generation with shrinking | 1.1.0, 2026-02-10 | Host tests only | Good |
| bolero | One test body for property testing and fuzzing engines | 0.13.6, 2026-09-30; active | Host tests only | Good |
| arbtest | Minimal property testing over `arbitrary` | 0.3.2, 2024-12-18 | Host tests only | Good |

The proptest book says both proptest and QuickCheck generate inputs and shrink failures, and argues that
QuickCheck's one generator per type makes custom strategies and composition harder
([proptest vs QuickCheck](https://github.com/proptest-rs/proptest/blob/main/book/src/proptest/vs-quickcheck.md);
this is the proptest authors' view). `proptest-state-machine` checks a system against a reference state machine and
shrinks to a minimal transition sequence
([state machine testing](https://github.com/proptest-rs/proptest/blob/main/book/src/proptest/state-machine.md)),
which matches the Domain and Block design of D-25 and D-27. bolero runs the same `check!` body as a property test
or under a fuzzing engine ([README](https://github.com/camshaft/bolero/blob/master/README.md)), which could serve
the hostile-CDR-input tests of layer L2 later.

Recommendation. proptest for L1 (Domain and Block invariants, with `proptest-state-machine`) and for L2 round
trips of the CDR codec. Because `deny.toml` excludes dev-dependencies and `no_std` checks do not build tests,
proptest does not touch the `no_std` gate. bolero is the candidate if fuzzing is added.

## 3. Profiling

| Name | What it measures | Maturity, last release | Pi and aarch64 fit | CI fit |
|---|---|---|---|---|
| perf | Sampled stacks and hardware counters | Kernel tool | Runs on the Pi; needs perf access | Not for gating |
| cargo-flamegraph | perf wrapped into a flame graph SVG | 0.6.14, 2026-08-12 | Linux only; `linux-tools-raspi` documented | Not for gating |
| inferno | Flame graph rendering library | 0.12.8, 2026-07-18; license CDDL-1.0 | Any | Not applicable |
| samply | Sampled stacks, viewed in Firefox Profiler | 0.13.1, 2025-02-01; commits 2026-09-30 | Prebuilt `aarch64-unknown-linux-gnu` binary | Not for gating |
| Hotspot | GUI for perf data | v1.6.0, 2026-02-27 | Prebuilt AppImage is x86_64 only; analyse off-device | Not applicable |
| Valgrind Callgrind and Cachegrind | Exact instruction counts, simulated caches | Valgrind 3.27.1 | ARM64 and ARMv7 supported | Deterministic (via Gungraun) |
| DHAT (Valgrind) | Allocation sites, peak heap | Part of Valgrind | ARM64 and ARMv7 supported | Possible via Gungraun |
| dhat-rs | Heap profiling through a global allocator | 0.3.3, 2024-02-04; author calls it experimental | Any | Heap usage tests |
| heaptrack | Every allocation with stacks, GUI | v1.3.0 (2021); commits 2026-09-24 | Builds for embedded; needs dynamic linking | Not applicable |
| bytehound | Every allocation with stacks | 0.11.0, 2022-11-23; no commits since 2023-07 | AArch64 and ARM supported | Not applicable |
| counts | Frequency tallies of log lines (ad hoc profiling) | 1.0.7, 2026-04-14 | Any | Not applicable |
| coz | Causal profiling (predicted speed-up per line) | coz v0.2.5, 2026-02-16; Rust crate 0.1.3 from 2020 | arm64 packages; needs perf events | Not applicable |
| tokio-console | Live async task states | console-subscriber 0.5.0, 2025-10-30 | Runs anywhere; needs `tokio_unstable` | Not applicable |

The perf book lists perf (with Hotspot or Firefox Profiler as viewers), samply, flamegraph, Cachegrind and
Callgrind, DHAT and dhat-rs, heaptrack and bytehound, counts, and Coz as profilers used successfully on Rust
([perf book, Profiling](https://nnethercote.github.io/perf-book/profiling.html)).

**Access to perf events.** `kernel.perf_event_paranoid` defaults to 2; at 2 or more users without `CAP_PERFMON`
cannot profile the kernel, at 1 or more they cannot access CPU events, and -1 allows almost all events
([kernel sysctl documentation](https://github.com/torvalds/linux/blob/master/Documentation/admin-guide/sysctl/kernel.rst)).
Since Linux 5.9, `CAP_PERFMON` alone is enough for a process to use perf events
([perf security](https://github.com/torvalds/linux/blob/master/Documentation/admin-guide/perf-security.rst)).
samply's README asks for `perf_event_paranoid` 1 (or -1) or `CAP_PERFMON`
([samply README](https://github.com/mstange/samply/blob/main/README.md)), and Coz asks for it to be relaxed too
([coz README](https://github.com/plasma-umass/coz/blob/master/README.md)). The sysctl is host-wide, so it is set
on the Pi host, not per container (unverified for containers that do not mount `/proc/sys` writable).

**Docker.** Docker's default seccomp profile lists `perf_event_open` among the syscalls it blocks
([Docker seccomp](https://docs.docker.com/engine/security/seccomp/)), and `ptrace` is limited by the dropped
`CAP_SYS_PTRACE`. The core container is privileged, so it has all capabilities
([Docker run reference](https://docs.docker.com/engine/containers/run/)). On the device under test the core
container has no seccomp filter and `perf_event_open` succeeds inside it (Repo baseline). Extensions are not
privileged and would need explicit permissions.

**Linker interaction.** cargo-flamegraph's README says that with lld (the default since Rust 1.90) or mold, perf
cannot produce accurate stacks unless the binary is linked with `--no-rosegment`
([flamegraph README](https://github.com/flamegraph-rs/flamegraph/blob/main/README.md)). This matters for host
profiling and for any future switch of the cross build to lld or mold. The README also documents
installing perf on Raspberry Pi with the `linux-tools-raspi` package.

**Heap profilers and static musl binaries.** heaptrack's launcher injects its library with `LD_PRELOAD`, or with
GDB when attaching to a running process ([heaptrack.sh](https://github.com/KDE/heaptrack/blob/master/src/track/heaptrack.sh.cmake)).
`LD_PRELOAD` relies on the dynamic loader, so it cannot intercept allocations in the statically linked musl
release binary; heap profiling needs a dynamically linked build such as `aarch64-unknown-linux-gnu` (inference
from how `LD_PRELOAD` works, not a heaptrack statement). Valgrind intercepts allocation functions in a statically
linked executable only if they are exported global symbols, and offers `--soname-synonyms=somalloc=NONE` for that
case ([Valgrind manual](https://valgrind.org/docs/manual/manual-core.html)); with `strip = true` those symbols are
gone, which is one more reason for an unstripped profiling profile. The heaptrack README describes building only
the recorder on an embedded device and analysing the data on another machine
([heaptrack README](https://github.com/KDE/heaptrack/blob/master/README.md)). dhat-rs works on any platform through a
global allocator, but its documentation says it is experimental, may crash or hang, and that maintenance is not a
priority for its author ([dhat docs](https://docs.rs/dhat/latest/dhat/)); its last commit was 2025-02-20. bytehound
supports AMD64, ARM, AArch64 and MIPS64 ([README](https://github.com/koute/bytehound/blob/master/README.md)) but has
had no commits since 2023-07-28 ([repository](https://github.com/koute/bytehound)), so it should be treated as
unmaintained.

**Async.** console-subscriber needs tokio's `tracing` feature and the `tokio_unstable` cfg, serves a gRPC endpoint
(port 6669 by default) and is a `tracing-subscriber` layer
([console-subscriber README](https://github.com/tokio-rs/console/blob/main/console-subscriber/README.md),
[console README](https://github.com/tokio-rs/console/blob/main/README.md)). The cfg flag changes `RUSTFLAGS`, so it
belongs in a development build only.

**Causal profiling.** Coz ships amd64 and arm64 packages and supports Rust
([coz README](https://github.com/plasma-umass/coz/blob/master/README.md)); the Rust crate needs source annotations
(`coz::progress!`, `coz::scope!`) and DWARF debug info ([coz-rs README](https://github.com/plasma-umass/coz/blob/master/rust/README.md)),
and its last crate release was 0.1.3 in 2020 ([crates.io](https://crates.io/crates/coz)). It is a specialist tool,
not a default.

**Ad hoc profiling.** counts tallies line frequencies, so `eprintln!` or `tracing` output can be turned into
profiles ([counts README](https://github.com/nnethercote/counts/blob/master/README.md)). Structured `tracing`
fields (D-13) make this almost free.

**Tried on the device under test** (Raspberry Pi 5, kernel 6.6.31, 2026-10-02):

- samply 0.13.1 panics at start on the host and inside `blueos-core` and records nothing. The Pi 5 kernel uses
  16 KiB pages (`getconf PAGESIZE` is 16384), and samply hard-codes a 4096-byte page when reading the perf ring
  buffer; its main branch still does.
- Debian's `linux-perf` 6.12.109, unpacked into `/tmp` with `apt-get download` and `dpkg -x` plus `libopencsd1` and
  `libtraceevent1`, ran against the 6.6.31 kernel with no version warning. `perf stat` read hardware counters and
  `perf record -g` captured 453 samples of the Recorder with none lost.
- Kernel frames were named; Recorder frames were raw addresses, because the binary is stripped and has no build ID,
  so perf cannot match it to an unstripped copy automatically.

Recommendation. On a Pi: `perf record`, unpacked into `/tmp` as above, with analysis in Hotspot or Firefox Profiler
on a workstation. samply only on kernels with 4 KiB pages (unverified on a Pi 4). In CI and for deterministic comparisons: Callgrind through Gungraun. Heap: heaptrack on a
glibc build, DHAT through Gungraun for benchmarks. Async: tokio-console behind a development feature. Skip
bytehound. Whether the tools ship in an image is open question 9. If `pprof` with its `flamegraph` feature is ever
added as a dependency, note that inferno is CDDL-1.0 ([crates.io](https://crates.io/crates/inferno)), which the
D-30 `cargo deny check licenses` allowlist would have to accept.

## 4. Cargo profile for profiling

What the sources say:

- The perf book recommends `debug = "line-tables-only"` in the release profile to get source lines, and notes the
  shipped standard library has no debug info; `build-std` can rebuild it but its paths do not point to source files
  ([perf book, Profiling](https://nnethercote.github.io/perf-book/profiling.html)).
- Cargo's `debug` accepts `"line-tables-only"` (minimal, for backtraces with file and line), `"limited"` (no type or
  variable information) and `true` or `"full"`; custom profiles must set `inherits`
  ([Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)).
- `strip = true` equals `strip = "symbols"`; the rustc book warns that stripping symbols can make traces
  incomprehensible on platforms that use the symbol table for backtraces and profiling
  ([rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html)). The current release
  profile strips symbols.
- `split-debuginfo`: `packed` is the default only on Windows MSVC and macOS, so on Linux debug info stays in the
  binary by default; `packed` writes a `.dwp` file and `unpacked` writes `.dwo` files
  ([rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html)).
- `-C force-frame-pointers=yes` forces frame pointers; without it the behaviour depends on the target
  ([rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html)). The perf book sets it with
  `RUSTFLAGS` or `[build] rustflags` ([perf book, Profiling](https://nnethercote.github.io/perf-book/profiling.html)).
  A `rustflags` key inside a Cargo profile is still unstable (`profile-rustflags`)
  ([Cargo unstable features](https://doc.rust-lang.org/cargo/reference/unstable.html)), so on stable the flag cannot
  live in `[profile.profiling]`.
- aarch64 Linux targets default to non-leaf frame pointers since Rust 1.89
  ([release notes](https://github.com/rust-lang/rust/blob/master/RELEASES.md),
  [rust#140832](https://github.com/rust-lang/rust/pull/140832)); confirmed for `aarch64-unknown-linux-musl` in the
  local 1.99.0 target specification. armv7 and x86_64 have no default.
- The prebuilt standard library has been built with frame pointers since Rust 1.79
  ([rust#122646](https://github.com/rust-lang/rust/pull/122646)), switched to non-leaf frame pointers in 2025
  ([rust#141800](https://github.com/rust-lang/rust/pull/141800)), and the flag now lives in the `dist` profile of
  `library/Cargo.toml` (`-Cforce-frame-pointers=non-leaf`) after
  [rust#149514](https://github.com/rust-lang/rust/pull/149514)
  ([library/Cargo.toml](https://github.com/rust-lang/rust/blob/main/library/Cargo.toml)). So the standard library
  does not need rebuilding for frame-pointer unwinding on any of the three targets.
- Symbol mangling: the perf book suggests `-C symbol-mangling-version=v0`
  ([perf book, Profiling](https://nnethercote.github.io/perf-book/profiling.html)), but Rust 1.97 made v0 the
  default and warns that old debuggers and profilers may fail to demangle it
  ([release notes](https://github.com/rust-lang/rust/blob/master/RELEASES.md),
  [rust#151994](https://github.com/rust-lang/rust/pull/151994)); legacy mangling is now nightly-only
  ([symbol mangling](https://doc.rust-lang.org/rustc/symbol-mangling/index.html)). The work flips from enabling v0 to
  checking that the perf, Valgrind, heaptrack and Hotspot versions used on the Pi demangle v0. heaptrack loads
  `rustc_demangle` at runtime when available ([heaptrack README](https://github.com/KDE/heaptrack/blob/master/README.md));
  `rustc-demangle` 0.1.28 is current ([crates.io](https://crates.io/crates/rustc-demangle)).

Candidate profile (for discussion, not applied):

```toml
[profile.profiling]
inherits = "release"
debug = "line-tables-only"
strip = false
```

Frame pointers for armv7 and x86_64 would come from the build command, for example
`RUSTFLAGS="-C force-frame-pointers=yes" ./build_cross.sh --profile profiling` (`Cross.toml` already passes
`RUSTFLAGS`). Putting the flag in a Cargo config file under `core/` would apply it to every profile, including release,
and change the cache key of every developer build. Open choices for the grill: `"line-tables-only"` versus
`"limited"` (Coz needs DWARF; richer info costs build time), whether to keep `lto = "thin"` in the profiling build
so it matches production, and whether to ship a stripped release binary plus a separate `.dwp` so production
crashes can be symbolized later. If the cross build ever moves to lld or mold, add `-Wl,--no-rosegment` to the
profiling build ([flamegraph README](https://github.com/flamegraph-rs/flamegraph/blob/main/README.md)).

## 5. Compilation time

| Name | What it measures or changes | Maturity, last release | Pi and aarch64 fit | CI fit |
|---|---|---|---|---|
| `cargo build --timings` | Per-unit timeline and critical path | Stable since Rust 1.60 | Host tool | Upload the HTML report |
| `-Z self-profile` and measureme | Time per compiler query | measureme 12.0.3; nightly flag | Host tool | Nightly job |
| cargo-llvm-lines | LLVM IR lines per generic function | 0.4.48, 2026-08-19 | Host tool | Report only |
| `-Zmacro-stats` | Code generated by macros | Nightly flag | Host tool | Nightly job |
| rust-lld | Linker | Default on x86_64 Linux since 1.90 | Self-contained for gnu targets | Already used on host |
| mold | Linker | v2.42.1, 2026-09-11 | Supports ARM 32 and 64 | Needs install in cross image |
| sccache | Compiler output cache | 0.18.0, 2026-09-16 | Host tool | Overlaps rust-cache |
| Cranelift backend | Faster debug code generation | Nightly rustup component | Linux x86_64 and AArch64 only | Debug builds only |
| cargo-hakari | Workspace-hack crate against feature unification rebuilds | 0.9.39, 2026-09-19 | Host tool | Adds a crate |

**Measuring.** `cargo build --timings` writes an HTML report with a unit timeline, concurrency and codegen time
([Cargo timings](https://doc.rust-lang.org/cargo/reference/timings.html)); it was stabilized in Rust 1.60
([release notes](https://github.com/rust-lang/rust/blob/master/RELEASES.md)). The perf book adds `-Zmacro-stats` for
macro-generated code and `cargo llvm-lines` for generic functions that produce a lot of LLVM IR
([perf book, Compile Times](https://nnethercote.github.io/perf-book/compile-times.html)). measureme is the
`-Z self-profile` tooling ([repository](https://github.com/rust-lang/measureme)).

**The unexpected workspace-wide build.** `core/` is a virtual workspace without `default-members`. Cargo's
documentation says that in a virtual workspace root, without `-p` or `--workspace`, all members are used, as if
`--workspace` were given ([Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)). That fully
explains why `cargo build --release --features=recorder` run in `core/` compiles the example service and every
other member; `build_cross.sh` avoids it with `-p blueos`. A second, smaller effect: when several packages are built
together, Cargo builds each shared dependency once with the union of the features all of them enable
([Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)), so a workspace-wide build and a
`-p blueos` build can compile dependencies with different features and invalidate each other's cache. Per-package
unification is still unstable (`-Z feature-unification` with `resolver.feature-unification`,
[Cargo unstable features](https://doc.rust-lang.org/cargo/reference/unstable.html)). cargo-hakari solves the rebuilds
with a generated workspace-hack crate ([cargo-hakari README](https://github.com/guppy-rs/guppy/blob/main/tools/cargo-hakari/README.md)),
but that crate would sit outside `logic/`, `adapters/` and `app/` and fail the folder check in
`.hooks/lib/rust_checks.sh`. Cheapest fixes to discuss: set `default-members = ["app/blueos"]`, or document
`-p blueos` for release builds.

**Linkers.** Rust 1.90 made rust-lld the default on `x86_64-unknown-linux-gnu`; the announcement reports a 7x
faster link and 40 percent shorter end-to-end time for an incremental ripgrep debug build, and the opt-out is
`-C linker-features=-lld` ([Rust blog](https://blog.rust-lang.org/2025/09/01/rust-lld-on-1.90.0-stable/)). The
announcement and the release note name only `x86_64-unknown-linux-gnu`
([release notes](https://github.com/rust-lang/rust/blob/master/RELEASES.md)); the perf book's wording "the default
linker on Linux" ([perf book, Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html))
is broader than the source. The musl targets used for releases still link through the cross image's GCC (Repo
baseline). mold supports ARM 32 and 64, and its README benchmarks show it 4 to 16 times faster than lld on outputs
from 0.15 to 9.5 GiB ([mold README](https://github.com/rui314/mold/blob/main/README.md)); the `blueos` binary is
about 14 MB, so the absolute saving is unknown and must be measured. mold is selected with `-fuse-ld=mold` on GCC
12.1 or later, or with `-B<mold libexec dir>` on older GCC ([mold README](https://github.com/rui314/mold/blob/main/README.md)).
cross supports `pre-build` commands and custom images per target
([cross configuration](https://github.com/cross-rs/cross/blob/main/docs/config_file.md)), which is how mold would get
into the image. The perf book says an alternative linker has no downside if it links correctly
([perf book, Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html)), but the
flamegraph README documents the `--no-rosegment` caveat for perf with lld and mold (section 3).

**Caching and backends.** sccache cannot cache crates that invoke the system linker (binaries, proc-macros) and
requires incremental compilation to be off ([sccache Rust notes](https://github.com/mozilla/sccache/blob/main/docs/Rust.md));
CI already caches the target folder with `Swatinem/rust-cache`, so sccache's gain there is unproven. The Cranelift
backend is a nightly rustup component for Linux x86_64 and AArch64, meant to speed up debug builds
([rustc_codegen_cranelift README](https://github.com/rust-lang/rustc_codegen_cranelift/blob/main/Readme.md)); it
does not apply to release cross builds or armv7.

**Debug info.** The perf book reports 20 to 40 percent faster dev builds without debug info, with
`line-tables-only` keeping most of the benefit ([perf book, Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html));
the dev profile already uses `line-tables-only` for workspace crates and no debug info for dependencies.

Recommendation. Fix the workspace-wide build first (it is the observed pain). Add `cargo build --timings` to CI as
an uploaded artifact, report-only. Try mold for the host development loop only if `--timings` shows linking on the
critical path; leave the cross release build on the image's linker unless a measurement shows a real gain.

## 6. Binary size

| Name | What it measures or changes | Maturity, last release | Pi and aarch64 fit | CI fit |
|---|---|---|---|---|
| cargo-bloat | Size per function and per crate | 0.12.1, 2024-05-10; no commits since | ELF supported | Already chosen in D-16 and D-30 |
| cargo-llvm-lines | IR lines per generic function | 0.4.48, 2026-08-19 | Host tool | Report only |
| twiggy | Code size profiler for Wasm | 0.8.0; repository archived | Not applicable to ELF | Not applicable |
| `opt-level = "s"` or `"z"` | Optimize for size | Stable | Slower code on a slow CPU | Measure |
| `codegen-units = 1` | Better optimization, slower build | Stable | Neutral | Longer CI build |
| `lto = "fat"` | Whole-program optimization | Stable | Neutral | Longer CI build |
| `panic = "abort"` | Removes unwinding code | Stable | Neutral | Blocked by D-29 |
| `build-std` and `-Zlocation-detail=none` | Size-optimized std, no panic locations | Nightly only | Neutral | Nightly job |
| UPX | Self-extracting compressed executable | v5.2.1, 2026-08-27 | ELF arm64 and arm32 supported | Measure |

**Compiler settings.** min-sized-rust recommends, in order, release mode, `strip`, `opt-level = "z"` (noting `"s"`
can be smaller), `lto = true`, `codegen-units = 1`, `panic = "abort"`, then nightly-only options such as
`-Zlocation-detail=none`, `build-std` and `panic=immediate-abort`, and finally UPX
([min-sized-rust](https://github.com/johnthagen/min-sized-rust)). It marks `panic = "abort"` as the first option that
changes program behaviour. The perf book's size summary is `opt-level = "z"`, `codegen-units = 1`, `lto = "fat"`,
`panic = "abort"` and `strip = "symbols"`, and it says to benchmark each change one at a time
([perf book, Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html)). It also notes
that thin LTO improves speed and size over the default thin local LTO, and fat LTO may improve them further "but
not always". The release profile already has `strip = true` and `lto = "thin"`; `panic = "abort"` conflicts with
D-29.

**Analysis tools.** cargo-bloat reports size per function and per crate for ELF, Mach-O and PE
([cargo-bloat README](https://github.com/RazrFalcon/cargo-bloat/blob/master/README.md)); its last release was
2024-05-10 ([crates.io](https://crates.io/crates/cargo-bloat)), which is a maintenance risk for a tool D-16 and D-30
already depend on. cargo-llvm-lines is actively released ([crates.io](https://crates.io/crates/cargo-llvm-lines)).
twiggy targets Wasm ([min-sized-rust](https://github.com/johnthagen/min-sized-rust)) and its repository is archived
([repository](https://github.com/AlexEne/twiggy)), so it does not apply.

**UPX.** UPX supports Linux ELF executables; its packer has classes for 64-bit Arm and 32-bit little-endian Arm
ELF ([p_lx_elf.h](https://github.com/upx/upx/blob/devel/src/p_lx_elf.h)), so aarch64 and armv7 are covered. UPX
5.0.0 uses `memfd_create` and can avoid `/proc/self/exe` ([NEWS](https://github.com/upx/upx/blob/devel/NEWS)).
For Linux ELF, UPX decompresses the whole program into memory at start, and the process relies on RAM and swap to
hold the decompressed program for its lifetime; the UPX documentation also says packed executables "do not share RAM
at runtime in the way that executables mapped from a file system do", and that big programs benefit less because
UPX decompresses all of the program even when only a fraction is used
([UPX manual](https://github.com/upx/upx/blob/devel/doc/upx-doc.txt)). The README claims 50 to 70 percent smaller
files ([UPX README](https://github.com/upx/upx/blob/devel/README)). min-sized-rust warns that UPX-packed binaries
have been flagged by heuristic antivirus software ([min-sized-rust](https://github.com/johnthagen/min-sized-rust)).
Two facts specific to BlueOS reduce the gain: the binary is a multicall binary meant to run as several processes
(D-15, D-16), and each packed process would hold its own decompressed copy instead of sharing page-cache pages; and
image layers are already transferred gzip- or zstd-compressed ([OCI layer specification](https://github.com/opencontainers/image-spec/blob/main/layer.md)),
so UPX's download saving is smaller than its on-disk saving.

Measurement plan (no verdict without data):

1. Build `blueos` for the three targets with the current release profile, then with each of `codegen-units = 1`,
   `lto = "fat"`, `opt-level = "s"` and `opt-level = "z"` applied one at a time. Record file size, `cargo bloat
   --crates` top entries, and release build time on CI.
2. For each binary, also record its gzip and zstd compressed size, which approximates the image download delta.
3. Pack the current binary with `upx --best --lzma` and record size and compressed size.
4. On a Pi 4 (and a Pi 3 if armv7 still ships): measure start-up with
   `hyperfine --warmup 3 'recorder --help'` and with a page-cache drop as `--prepare` for cold start, for the plain
   and the UPX binary.
5. With the Recorder running under load, read `RssAnon` and `RssFile` from `/proc/<pid>/status` for the plain and
   the UPX binary, and repeat with two processes from the same binary to see the lost sharing.
6. For the opt-level variants, run the Criterion benchmarks of section 1 on the Pi to price the speed cost.
7. Decide with the numbers and with open questions 6 and 7.

## 7. Service observability at runtime

| Name | What it provides | Maturity, last release | Pi and aarch64 fit | Fit with BlueOS |
|---|---|---|---|---|
| tracing and tracing-subscriber | Spans and events | 0.1.44 and 0.3.23; 895 million downloads | Already in use | Already the logging API (D-13) |
| metrics (facade) and metrics-util | Counters, gauges, histograms behind a pluggable recorder | 0.24.6, 2026-05-13 | Light | A Zenoh recorder fits D-12 |
| metrics-exporter-prometheus | HTTP scrape endpoint for `metrics` | 0.18.3, 2026-04-30 | Adds an HTTP server | Optional bridge |
| prometheus-client | Official OpenMetrics client | 0.25.1, 2026-09-01 | Light | Already used inside Zenoh `stats` |
| prometheus (tikv) | Prometheus client | 0.14.0, 2025-03-27 | Light | Less active |
| OpenTelemetry (`opentelemetry`, SDK, OTLP) | Traces, metrics, logs, OTLP export | 0.33.0, 2026-09-18 | Heavier; needs a collector | For a fleet backend |
| tracing-opentelemetry | Bridge from `tracing` spans to OpenTelemetry | 0.34.0, 2026-09-23 | Same | Same |
| tokio-metrics | Runtime and per-task metrics | 0.5.2, 2026-08-28 | Light | Exports through `metrics` |
| tokio-console | Live task debugger | 0.5.0, 2025-10-30 | Needs `tokio_unstable` | Development only |
| pyroscope (Grafana) | Continuous profiling, push to a server | 2.1.1, 2026-07-21 | Needs a server | Out of scope on the vehicle |
| pprof (tikv) | In-process sampling profiler | 0.15.0, 2025-05-27 | Frame-pointer feature is nightly-only | On-demand profiles |
| Parca Agent | eBPF continuous profiler | v0.50.0, 2026-09-21 | aarch64 and x86_64 only; kernel 5.3+ with BTF | Out of scope |
| Zenoh admin space metrics | OpenMetrics text from the Zenoh runtime | In zenoh 1.9.0 | Already on the device | First-party, no new crate |
| jamesgober/rust-benchmark (`benchmark`) | Benchmark suite plus timers for production | 0.8.0, 2025-09-04; 3 stars | Light | Not recommended |

**tracing** is already the instrumentation API and D-13 routes its events to the `log` key. It has 895 million
downloads ([crates.io](https://crates.io/crates/tracing)).

**metrics** describes itself as a lightweight metrics facade similar to `log`: libraries record, and the
application installs an exporter; the project ships a Prometheus exporter, a TCP exporter and
`metrics-tracing-context` ([metrics README](https://github.com/metrics-rs/metrics/blob/main/README.md)). The
recorder is the seam: a BlueOS recorder could aggregate in-process and publish snapshots on a per-service Zenoh key,
encoded with the IDL, the same way the logging adapter publishes `log`. That keeps one transport, lets the Recorder
store metrics in MCAP next to logs, and lets the frontend read them through `blueos-api`. The exact key and message
type would be a decision under D-12, and the cookbook (D-20) would show it.

**Prometheus clients.** prometheus-client is the Prometheus organization's OpenMetrics client
([README](https://github.com/prometheus/client_rust/blob/master/README.md)). The tikv `prometheus` crate's last
release was 2025-03-27 ([crates.io](https://crates.io/crates/prometheus)). Either implies an HTTP endpoint per
service (or a push gateway); BlueOS has no Prometheus server on the vehicle today.

**OpenTelemetry.** The Rust implementation's status table lists Logs and Metrics API and SDK as Stable, the OTLP
exporters for logs and metrics as RC, the Prometheus exporter as Beta, and Traces API, SDK and OTLP exporter as
Beta ([opentelemetry-rust README](https://github.com/open-telemetry/opentelemetry-rust/blob/main/README.md)). It
offers two bridges from `tracing`: `opentelemetry-appender-tracing` for logs and `tracing-opentelemetry` for spans
([appender README](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-appender-tracing/README.md)).
It needs a collector or backend to be useful. Because instrumentation is written against `tracing` (and `metrics`),
an OpenTelemetry exporter can be added later as one more layer without touching services.

**tokio runtime metrics.** tokio-metrics provides task metrics through `TaskMonitor` and runtime metrics, where
unstable runtime metrics need `tokio_unstable`, and its `metrics-rs-integration` feature exports them through
`metrics` exporters ([tokio-metrics README](https://github.com/tokio-rs/tokio-metrics/blob/main/README.md)). The
stable subset fits the facade approach without changing `RUSTFLAGS`.

**Continuous profiling.** pyroscope pushes profiles to a Pyroscope server URL and uses a pprof-rs backend
([pyroscope docs](https://docs.rs/pyroscope/latest/pyroscope/)). pprof-rs's frame-pointer backtrace feature is
nightly-only ([pprof-rs README](https://github.com/tikv/pprof-rs/blob/master/README.md)). Parca Agent requires Linux
5.3 or later with BTF ([Parca Agent README](https://github.com/parca-dev/parca-agent/blob/main/README.md)) and
publishes only aarch64 and x86_64 binaries ([releases](https://github.com/parca-dev/parca-agent/releases)), so armv7
is excluded. All three need infrastructure off the vehicle; on-demand profiling (section 3) covers the near term.

**Zenoh's own metrics.** In zenoh 1.9.0 the admin space registers a `metrics` handler under
`@/{zid}/{whatami}/metrics` that replies with OpenMetrics text (encoding
`application/openmetrics-text; version=1.0.0`): only a build-information metric without the `stats` feature, and
the runtime's statistics encoded by its stats registry with it ([adminspace.rs at 1.9.0](https://github.com/eclipse-zenoh/zenoh/blob/1.9.0/zenoh/src/net/runtime/adminspace.rs),
[zenoh Cargo.toml](https://github.com/eclipse-zenoh/zenoh/blob/main/zenoh/Cargo.toml)). The `stats` feature pulls in
`zenoh-stats`, which uses `prometheus-client`
([zenoh-stats Cargo.toml](https://github.com/eclipse-zenoh/zenoh/blob/main/commons/zenoh-stats/Cargo.toml)), after
the stats rework in [zenoh#2260](https://github.com/eclipse-zenoh/zenoh/pull/2260). The default configuration keeps
the admin space disabled and labels its configuration unstable, and offers per-key-expression stats filters
([DEFAULT_CONFIG.json5](https://github.com/eclipse-zenoh/zenoh/blob/main/DEFAULT_CONFIG.json5)). Enabling this on
the BlueOS `zenohd` (D-09 already plans a custom build) gives bus-level metrics with no new service code. Whether
the REST plugin serves the same key over HTTP for a Prometheus scrape was not verified.

Cost and control of `stats`, read from the zenoh source at tag 1.9.0 on 2026-10-02:

- Each routed message pays a few relaxed atomic additions per direction, with no locks or allocations once a label
  set exists. Nothing scales with key expressions unless `stats.filters` is configured, and the default has none.
  Shared-memory messages are counted the same way, with an extra `shm` label. No benchmark of the overhead was
  found; the only maintainer remark is qualitative.
- There is no runtime switch for the counting. The admin space settings, read permissions and filters only change
  what is exposed; building without the feature is the only way to stop counting.
- `zenohd` turns the admin space on whatever the configuration says, and its read permission is all or nothing. So
  anything on the bus can already read the router's admin space today, with or without `stats`. Narrowing it to the
  metrics key needs `access_control` with default deny, an allow rule for `@/*/router/metrics`, and explicit allow
  rules for all normal traffic, because `**` never matches `@/` keys.
- In 1.9.0 the payload and per-key histograms are never recorded for unicast transports, which is how services
  connect; [zenoh#2636](https://github.com/eclipse-zenoh/zenoh/pull/2636) fixes it in 1.10.0. Entries for
  disconnected clients are only cleaned up when the `metrics` key is read (inferred from the code, not tested).

**jamesgober/rust-benchmark.** The crate is `benchmark` 0.8.0, released 2025-09-04, with about 4,500 total downloads
([crates.io](https://crates.io/crates/benchmark)); the repository has 3 stars and no commits since 2025-09-04
([repository](https://github.com/jamesgober/rust-benchmark)). Its README presents it as both a statistical
benchmarking suite and a lightweight production performance monitor with timers, percentiles and spans
([README](https://github.com/jamesgober/rust-benchmark/blob/main/README.md)). Each half is covered by a far more
used project (Criterion for benchmarks, `tracing` plus `metrics` for production), so it does not meet D-32's "reuse
mature projects" bar.

**Resource constraints.** No source measured the overhead of these crates on a Pi; the cost of an HTTP server per
service, a gRPC server (tokio-console) or an OTLP exporter has to be measured with the section 6 method (RSS and
binary size deltas). The Zenoh path adds no server, because every service already holds a Zenoh session (D-10).

Recommendation. Keep `tracing` for events and spans. Add the `metrics` facade for counters, gauges and histograms,
with a BlueOS recorder that publishes on Zenoh under the D-12 standard keys, plus the stable tokio-metrics subset.
Enable the `stats` feature and the admin space on `zenohd` for bus metrics. Keep Prometheus and OpenTelemetry
exporters as optional bridges decided by open question 8. tokio-console stays a development feature.

## Candidate ADRs

Titles only, each with the question it would answer.

1. **Benchmark harnesses and where benchmarks live.** Which harness for wall-clock, which for instruction counts,
   which for whole-binary timing, and which crates must carry benchmarks?
2. **Performance regression gate.** Which metric (instruction counts, binary size, wall time), on which runners
   (x86_64, `ubuntu-24.04-arm`, a Pi), report-only or gating, and with what thresholds?
3. **Benchmark history storage.** Artifacts only, self-hosted Bencher, Bencher Cloud, or CodSpeed?
4. **Property-based testing.** Is proptest (with `proptest-state-machine`) the standard for L1 and L2, and is
   fuzzing (bolero) in scope?
5. **Profiling build profile.** The exact `[profile.profiling]`, how frame pointers are passed per target, and
   whether release binaries keep symbols or ship a separate debug file.
6. **Profiling tools on the device.** Which tools are supported, where they are installed (core image, debug image,
   by hand), and how perf access is granted inside containers.
7. **Workspace build selection.** `default-members`, `-p blueos`, or a workspace-hack, to stop workspace-wide and
   feature-unification rebuilds.
8. **Linker choice.** Keep rust-lld on the host and the image linker for cross builds, or adopt mold, and where?
9. **Release profile for size and speed.** Which of `codegen-units`, `lto`, `opt-level` change, given D-29 rules out
   `panic = "abort"`?
10. **Executable compression.** Adopt UPX or not, based on the section 6 measurements.
11. **Service metrics.** The `metrics` facade with a Zenoh recorder: key layout under D-12, IDL message type, and
    whether the Recorder stores metrics.
12. **Bus observability.** Enable Zenoh `stats` and the admin space on `zenohd`, and who may read it.
13. **External telemetry bridges.** Whether and when to add Prometheus or OpenTelemetry exporters.
14. **Toolchain pinning for measurements.** Pin the Rust toolchain so performance history is comparable?

## Sources

Rust project documentation:

- Rust Performance Book: [Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html),
  [Profiling](https://nnethercote.github.io/perf-book/profiling.html),
  [Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html),
  [Compile Times](https://nnethercote.github.io/perf-book/compile-times.html)
  (sources at [nnethercote/perf-book](https://github.com/nnethercote/perf-book))
- Cargo Book: [Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html),
  [Features](https://doc.rust-lang.org/cargo/reference/features.html),
  [Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html),
  [Timings](https://doc.rust-lang.org/cargo/reference/timings.html),
  [Unstable features](https://doc.rust-lang.org/cargo/reference/unstable.html),
  [Configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- rustc book: [Codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html),
  [Symbol mangling](https://doc.rust-lang.org/rustc/symbol-mangling/index.html),
  [Linker-plugin-based LTO](https://doc.rust-lang.org/rustc/linker-plugin-lto.html)
- Rust [RELEASES.md](https://github.com/rust-lang/rust/blob/master/RELEASES.md) (1.60 timings, 1.79 std frame
  pointers, 1.89 aarch64 frame pointers, 1.90 lld, 1.97 v0 mangling)
- Rust pull requests: [#122646](https://github.com/rust-lang/rust/pull/122646),
  [#140525](https://github.com/rust-lang/rust/pull/140525), [#140832](https://github.com/rust-lang/rust/pull/140832),
  [#141800](https://github.com/rust-lang/rust/pull/141800), [#149514](https://github.com/rust-lang/rust/pull/149514),
  [#151994](https://github.com/rust-lang/rust/pull/151994);
  [library/Cargo.toml](https://github.com/rust-lang/rust/blob/main/library/Cargo.toml)
- Rust blog: [Faster linking times with 1.90.0 stable on Linux using the LLD linker](https://blog.rust-lang.org/2025/09/01/rust-lld-on-1.90.0-stable/)
- [min-sized-rust](https://github.com/johnthagen/min-sized-rust)
- [rustc_codegen_cranelift README](https://github.com/rust-lang/rustc_codegen_cranelift/blob/main/Readme.md)
- [measureme](https://github.com/rust-lang/measureme)

Benchmarking and testing tools:

- Criterion: [repository](https://github.com/criterion-rs/criterion.rs),
  [README](https://github.com/criterion-rs/criterion.rs/blob/master/README.md),
  [FAQ](https://github.com/criterion-rs/criterion.rs/blob/master/book/src/faq.md),
  [analysis](https://github.com/criterion-rs/criterion.rs/blob/master/book/src/analysis.md),
  [async guide](https://github.com/criterion-rs/criterion.rs/blob/master/book/src/user_guide/benchmarking_async.md),
  [Cargo.toml](https://github.com/criterion-rs/criterion.rs/blob/master/Cargo.toml)
- Divan: [repository](https://github.com/nvzqz/divan), [async issue](https://github.com/nvzqz/divan/issues/39)
- Gungraun: [README](https://github.com/gungraun/gungraun/blob/main/README.md),
  [CHANGELOG](https://github.com/gungraun/gungraun/blob/main/CHANGELOG.md),
  [prerequisites](https://gungraun.github.io/gungraun/latest/html/installation/prerequisites.html),
  [regressions](https://gungraun.github.io/gungraun/latest/html/regressions.html)
- [hyperfine README](https://github.com/sharkdp/hyperfine/blob/master/README.md)
- [tango README](https://github.com/bazhenov/tango/blob/master/README.md)
- Bencher: [thresholds](https://bencher.dev/docs/explanation/thresholds/),
  [self-hosting](https://bencher.dev/docs/tutorial/docker/),
  [LICENSE](https://github.com/bencherdev/bencher/blob/devel/LICENSE.md)
- CodSpeed: [codspeed-rust](https://github.com/CodSpeedHQ/codspeed-rust),
  [CPU simulation](https://codspeed.io/docs/instruments/cpu), [wall time](https://codspeed.io/docs/instruments/walltime),
  [pricing](https://codspeed.io/pricing)
- proptest: [QuickCheck comparison](https://github.com/proptest-rs/proptest/blob/main/book/src/proptest/vs-quickcheck.md),
  [state machine testing](https://github.com/proptest-rs/proptest/blob/main/book/src/proptest/state-machine.md)
- [bolero README](https://github.com/camshaft/bolero/blob/master/README.md)
- GitHub: [arm64 runners for public repositories](https://github.blog/changelog/2025-08-07-arm64-hosted-runners-for-public-repositories-are-now-generally-available/),
  [GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)

Profiling tools and platform:

- Linux kernel: [sysctl kernel.rst](https://github.com/torvalds/linux/blob/master/Documentation/admin-guide/sysctl/kernel.rst),
  [perf-security.rst](https://github.com/torvalds/linux/blob/master/Documentation/admin-guide/perf-security.rst)
- Docker: [seccomp](https://docs.docker.com/engine/security/seccomp/),
  [running containers](https://docs.docker.com/engine/containers/run/)
- [samply README](https://github.com/mstange/samply/blob/main/README.md), [samply releases](https://github.com/mstange/samply/releases)
- [flamegraph README](https://github.com/flamegraph-rs/flamegraph/blob/main/README.md)
- [Hotspot](https://github.com/KDAB/hotspot)
- Valgrind: [supported platforms](https://valgrind.org/info/platforms.html),
  [core manual](https://valgrind.org/docs/manual/manual-core.html)
- [dhat crate documentation](https://docs.rs/dhat/latest/dhat/), [dhat-rs](https://github.com/nnethercote/dhat-rs)
- heaptrack: [README](https://github.com/KDE/heaptrack/blob/master/README.md),
  [heaptrack.sh](https://github.com/KDE/heaptrack/blob/master/src/track/heaptrack.sh.cmake)
- bytehound: [README](https://github.com/koute/bytehound/blob/master/README.md), [repository](https://github.com/koute/bytehound)
- [counts README](https://github.com/nnethercote/counts/blob/master/README.md)
- Coz: [README](https://github.com/plasma-umass/coz/blob/master/README.md),
  [Rust README](https://github.com/plasma-umass/coz/blob/master/rust/README.md)
- tokio console: [README](https://github.com/tokio-rs/console/blob/main/README.md),
  [console-subscriber README](https://github.com/tokio-rs/console/blob/main/console-subscriber/README.md)

Build, linking and size tools:

- mold: [README](https://github.com/rui314/mold/blob/main/README.md)
- cross: [configuration](https://github.com/cross-rs/cross/blob/main/docs/config_file.md),
  [aarch64 musl Dockerfile](https://github.com/cross-rs/cross/blob/main/docker/Dockerfile.aarch64-unknown-linux-musl),
  [releases](https://github.com/cross-rs/cross/releases)
- [sccache Rust notes](https://github.com/mozilla/sccache/blob/main/docs/Rust.md)
- [cargo-hakari README](https://github.com/guppy-rs/guppy/blob/main/tools/cargo-hakari/README.md)
- [cargo-bloat README](https://github.com/RazrFalcon/cargo-bloat/blob/master/README.md)
- [twiggy repository](https://github.com/AlexEne/twiggy)
- UPX: [README](https://github.com/upx/upx/blob/devel/README), [manual](https://github.com/upx/upx/blob/devel/doc/upx-doc.txt),
  [NEWS](https://github.com/upx/upx/blob/devel/NEWS), [p_lx_elf.h](https://github.com/upx/upx/blob/devel/src/p_lx_elf.h)
- [OCI image layer specification](https://github.com/opencontainers/image-spec/blob/main/layer.md)

Observability:

- [metrics README](https://github.com/metrics-rs/metrics/blob/main/README.md)
- [prometheus-client README](https://github.com/prometheus/client_rust/blob/master/README.md)
- [tikv rust-prometheus](https://github.com/tikv/rust-prometheus)
- OpenTelemetry Rust: [README](https://github.com/open-telemetry/opentelemetry-rust/blob/main/README.md),
  [appender-tracing README](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-appender-tracing/README.md),
  [tracing-opentelemetry](https://github.com/tokio-rs/tracing-opentelemetry)
- [tokio-metrics README](https://github.com/tokio-rs/tokio-metrics/blob/main/README.md)
- [pyroscope documentation](https://docs.rs/pyroscope/latest/pyroscope/), [pyroscope-rs](https://github.com/grafana/pyroscope-rs)
- [pprof-rs README](https://github.com/tikv/pprof-rs/blob/master/README.md)
- Parca Agent: [README](https://github.com/parca-dev/parca-agent/blob/main/README.md),
  [releases](https://github.com/parca-dev/parca-agent/releases)
- Zenoh: [adminspace.rs at 1.9.0](https://github.com/eclipse-zenoh/zenoh/blob/1.9.0/zenoh/src/net/runtime/adminspace.rs),
  [zenoh Cargo.toml](https://github.com/eclipse-zenoh/zenoh/blob/main/zenoh/Cargo.toml),
  [zenoh-stats Cargo.toml](https://github.com/eclipse-zenoh/zenoh/blob/main/commons/zenoh-stats/Cargo.toml),
  [stats rework, zenoh#2260](https://github.com/eclipse-zenoh/zenoh/pull/2260),
  [DEFAULT_CONFIG.json5](https://github.com/eclipse-zenoh/zenoh/blob/main/DEFAULT_CONFIG.json5)
- jamesgober/rust-benchmark: [repository](https://github.com/jamesgober/rust-benchmark),
  [README](https://github.com/jamesgober/rust-benchmark/blob/main/README.md)

Versions, release dates, download counts and licenses in the tables come from the crates.io API
(`https://crates.io/api/v1/crates/<name>`) and from GitHub repository and release metadata, both read on
2026-10-02.

Unverified items, collected:

- Which Rust release made rust-lld the default for `aarch64-unknown-linux-gnu` and `armv7-unknown-linux-gnueabihf`
  (observed in the 1.99.0 target specification only).
- The GCC version in the cross 0.2.5 images (it decides `-fuse-ld=mold` versus `-B`).
- Whether CodSpeed's CPU simulation supports aarch64.
- Whether the Zenoh REST plugin exposes the admin space `metrics` key over HTTP.
- v0 demangling support in the perf, Valgrind and Hotspot versions available on the Pi.
- Runtime overhead of any observability crate on a Pi (no primary source measured it).
