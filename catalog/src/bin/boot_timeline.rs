//! First-boot timeline and resource capture for a BlueOS device.
//!
//! Subcommands:
//! - `monitor` (laptop): waits for the device to answer ping (phase 1), installs a static sampler
//!   on it over ssh, waits for nginx to answer `:80` with 200 (phase 2), keeps sampling for
//!   `--settle` seconds, then retrieves the samples and prints the report. Reboots done by the
//!   first-boot update scripts are detected and the sampler is brought back after each one.
//! - `sample` (device): appends CPU / RSS / swap samples to a JSONL file that survives reboots.
//!   This is the same binary, cross-built for the device and pushed by `monitor`.
//! - `retrieve`: fetch the samples again (and optionally uninstall) without a full monitor run.
//! - `report`: re-print the report from an output directory.
//!
//! Resource sampling can only start once sshd accepts connections, so the first seconds of the
//! first boot have ping/HTTP timing but no resource samples; later boots are covered from
//! `multi-user.target` onward.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use blueos_catalog::report::format_unix_utc_rfc3339;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

const REMOTE_DIR: &str = "/home/pi/boot-monitor";
const REMOTE_BIN: &str = "/home/pi/boot-monitor/boot_sampler";
const REMOTE_SAMPLES: &str = "/home/pi/boot-monitor/samples.jsonl";
const UNIT: &str = "blueos-boot-sampler.service";
const UNIT_PATH: &str = "/etc/systemd/system/blueos-boot-sampler.service";
const DEFAULT_TARGETS: [&str; 2] = [
    "armv7-unknown-linux-musleabihf",
    "aarch64-unknown-linux-musl",
];
const SSH_OPTS: [&str; 8] = [
    "-o",
    "StrictHostKeyChecking=no",
    "-o",
    "UserKnownHostsFile=/dev/null",
    "-o",
    "LogLevel=ERROR",
    "-o",
    "ConnectTimeout=6",
];
const MEM_KEYS: [&str; 11] = [
    "MemTotal",
    "MemFree",
    "MemAvailable",
    "Buffers",
    "Cached",
    "SwapTotal",
    "SwapFree",
    "SwapCached",
    "Dirty",
    "Shmem",
    "Slab",
];
const USAGE: &str = "\
boot_timeline monitor  [--host H] [--user U] [--password P] [--port N] [--path P]
                       [--interval S] [--poll S] [--settle S] [--max-wait S]
                       [--out-dir D] [--targets T,T] [--sampler FILE]
                       [--proc-swap] [--no-agent]
boot_timeline retrieve [--host H] [--user U] [--password P] [--out-dir D] [--uninstall]
boot_timeline sample   [--out FILE] [--interval S] [--duration S] [--proc-swap]
boot_timeline report   <out-dir|samples.jsonl>

Defaults: --host 192.168.2.2 --user pi --password raspberry --port 80 --path /
          --interval 1 --poll 1 --settle 120 --max-wait 3600
          --out-dir boot-timeline-<host>
";

/// One `/proc/<pid>/stat` row: pid, starttime, comm, cpu ticks (utime+stime), RSS kB, swap kB.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcRow {
    p: i64,
    s: u64,
    c: String,
    k: u64,
    r: u64,
    w: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "ev")]
enum Record {
    #[serde(rename = "start")]
    Start {
        t: f64,
        up: f64,
        boot_id: String,
        btime: f64,
        clk_tck: u64,
        page_kb: u64,
        ncpu: usize,
        interval: f64,
        uname: String,
    },
    #[serde(rename = "s")]
    Sample {
        t: f64,
        up: f64,
        cpu: Vec<u64>,
        mem: BTreeMap<String, u64>,
        load: Vec<f64>,
        p: Vec<ProcRow>,
    },
    #[serde(rename = "proc")]
    Proc {
        t: f64,
        pid: i64,
        st: u64,
        cmd: String,
    },
    #[serde(rename = "err")]
    Err { t: f64, msg: String },
}

#[derive(Debug, Clone)]
struct MonitorCfg {
    host: String,
    user: String,
    password: String,
    port: u16,
    path: String,
    interval: f64,
    poll: f64,
    settle: f64,
    max_wait: f64,
    out_dir: PathBuf,
    targets: Vec<String>,
    sampler: Option<PathBuf>,
    proc_swap: bool,
    no_agent: bool,
    uninstall: bool,
}

#[derive(Debug)]
struct SampleCfg {
    out: PathBuf,
    interval: f64,
    duration: f64,
    proc_swap: bool,
}

#[derive(Debug)]
enum Mode {
    Monitor(MonitorCfg),
    Retrieve(MonitorCfg),
    Sample(SampleCfg),
    Report(PathBuf),
}

#[derive(Debug, Clone)]
struct Health {
    boot_id: String,
    uptime: f64,
    unit: String,
    lines: u64,
}

#[derive(Debug, Default)]
struct Outcome {
    phase2: Option<f64>,
    reason: String,
}

/// Everything the monitor loop needs from the device, so the loop can be driven by a script in
/// tests instead of a live Pi.
trait Probe {
    fn ping(&mut self) -> bool;
    fn http(&mut self) -> Result<u16, String>;
    fn health(&mut self) -> Option<Health>;
    fn install(&mut self, rotate: bool) -> Result<String, String>;
}

struct LivePi {
    cfg: MonitorCfg,
    samplers: BTreeMap<String, PathBuf>,
}

struct EventLog {
    path: PathBuf,
    file: File,
}

#[derive(Debug)]
struct ProcStat {
    comm: String,
    first: u64,
    last: u64,
    peak: u64,
}

#[derive(Debug)]
struct Session {
    boot_id: String,
    btime: f64,
    clk_tck: u64,
    ncpu: usize,
    samples: Vec<Record>,
    cmds: BTreeMap<String, String>,
    procs: BTreeMap<String, ProcStat>,
}

impl Probe for LivePi {
    fn ping(&mut self) -> bool {
        Command::new("ping")
            .args(["-n", "-c", "1", "-W", "1", "-w", "2", &self.cfg.host])
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    fn http(&mut self) -> Result<u16, String> {
        let url = if self.cfg.port == 80 {
            format!("http://{}{}", self.cfg.host, self.cfg.path)
        } else {
            format!(
                "http://{}:{}{}",
                self.cfg.host, self.cfg.port, self.cfg.path
            )
        };
        let out = Command::new("curl")
            .args([
                "-s",
                "-o",
                "/dev/null",
                "-m",
                "3",
                "-w",
                "%{http_code}",
                &url,
            ])
            .output()
            .map_err(|error| error.to_string())?;
        let code = String::from_utf8_lossy(&out.stdout).trim().to_string();
        match code.parse::<u16>() {
            Ok(0) | Err(_) => Err(format!("no response ({code})")),
            Ok(status) => Ok(status),
        }
    }

    fn health(&mut self) -> Option<Health> {
        let script = format!(
            "cat /proc/sys/kernel/random/boot_id\nawk '{{print $1}}' /proc/uptime\n\
             systemctl is-active {UNIT} 2>/dev/null || true\nwc -l < {REMOTE_SAMPLES} 2>/dev/null || echo 0\n"
        );
        let (code, out, _) = self.ssh(&script, 25);
        if code != 0 {
            return None;
        }
        let lines: Vec<&str> = out.lines().map(str::trim).collect();
        if lines.len() < 4 {
            return None;
        }
        Some(Health {
            boot_id: lines[0].to_string(),
            uptime: lines[1].parse().unwrap_or(0.0),
            unit: lines[2].to_string(),
            lines: lines[3].parse().unwrap_or(0),
        })
    }

    fn install(&mut self, rotate: bool) -> Result<String, String> {
        let (code, _, err) = self.ssh(&format!("mkdir -p {REMOTE_DIR}"), 25);
        if code != 0 {
            return Err(format!("mkdir: {err}"));
        }
        if rotate {
            // keep samples from an earlier experiment out of this run's report
            self.ssh(
                &format!("test -f {REMOTE_SAMPLES} && mv {REMOTE_SAMPLES} {REMOTE_SAMPLES}.$(date +%s) || true"),
                25,
            );
        }
        let sampler = self.pick_sampler()?;
        let staged = format!("{REMOTE_BIN}.new");
        let (code, _, err) = self.scp(
            sampler.as_os_str().to_string_lossy().as_ref(),
            &format!("{}@{}:{staged}", self.cfg.user, self.cfg.host),
        );
        if code != 0 {
            return Err(format!("scp sampler: {err}"));
        }
        let unit = unit_text(self.cfg.interval, self.cfg.proc_swap);
        let (code, out, err) = self.ssh(&install_script(&self.cfg.password, &unit, &staged), 90);
        let state = out.lines().last().unwrap_or("").trim().to_string();
        if code != 0 || !unit_ok(&state) {
            return Err(format!("install: {out} {err}"));
        }
        Ok(state)
    }
}

impl LivePi {
    fn new(cfg: MonitorCfg, samplers: BTreeMap<String, PathBuf>) -> Self {
        Self { cfg, samplers }
    }

    fn ssh(&self, command: &str, timeout: u32) -> (i32, String, String) {
        let target = format!("{}@{}", self.cfg.user, self.cfg.host);
        let mut argv: Vec<String> = vec![timeout.to_string(), "ssh".into()];
        argv.extend(SSH_OPTS.iter().map(|opt| (*opt).to_string()));
        argv.push(target);
        argv.push(command.to_string());
        self.run("timeout", &argv)
    }

    fn scp(&self, from: &str, to: &str) -> (i32, String, String) {
        let mut argv: Vec<String> = vec!["180".into(), "scp".into()];
        argv.extend(SSH_OPTS.iter().map(|opt| (*opt).to_string()));
        argv.push(from.to_string());
        argv.push(to.to_string());
        self.run("timeout", &argv)
    }

    fn run(&self, program: &str, argv: &[String]) -> (i32, String, String) {
        let mut command;
        if self.cfg.password.is_empty() {
            command = Command::new(program);
            command.args(argv);
        } else {
            command = Command::new("sshpass");
            command.args(["-p", &self.cfg.password, program]);
            command.args(argv);
        }
        match command.output() {
            Ok(out) => (
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stdout).trim().to_string(),
                String::from_utf8_lossy(&out.stderr).trim().to_string(),
            ),
            Err(error) => (-1, String::new(), error.to_string()),
        }
    }

    /// The device tells us its arch; pushing a binary for the wrong one would only fail on exec.
    fn pick_sampler(&self) -> Result<PathBuf, String> {
        if let Some(path) = &self.cfg.sampler {
            return Ok(path.clone());
        }
        let (code, machine, err) = self.ssh("uname -m", 25);
        if code != 0 {
            return Err(format!("uname -m: {err}"));
        }
        let target = match machine.trim() {
            "armv6l" | "armv7l" | "armv8l" => "armv7-unknown-linux-musleabihf",
            "aarch64" | "arm64" => "aarch64-unknown-linux-musl",
            other => return Err(format!("unsupported device arch {other}; pass --sampler")),
        };
        self.samplers.get(target).cloned().ok_or_else(|| {
            format!(
                "device is {}, but no sampler built for {target}",
                machine.trim()
            )
        })
    }

    fn retrieve(&self, out_dir: &Path) -> Result<PathBuf, String> {
        let local = out_dir.join("samples.jsonl");
        let (code, _, err) = self.scp(
            &format!("{}@{}:{REMOTE_SAMPLES}", self.cfg.user, self.cfg.host),
            local.as_os_str().to_string_lossy().as_ref(),
        );
        if code != 0 {
            return Err(format!("scp samples: {err}"));
        }
        Ok(local)
    }

    fn uninstall(&self) -> (i32, String, String) {
        self.ssh(
            &format!(
                "export SUDO_ASKPASS={REMOTE_DIR}/askpass\n\
                 sudo -A systemctl disable --now {UNIT} > /dev/null 2>&1 || true\n\
                 sudo -A rm -f {UNIT_PATH} || true\n\
                 sudo -A systemctl daemon-reload || true\n\
                 rm -f {REMOTE_DIR}/askpass\n"
            ),
            60,
        )
    }
}

impl EventLog {
    fn open(path: PathBuf) -> Result<Self, String> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(Self { path, file })
    }

    fn event(&mut self, kind: &str, fields: Value) -> f64 {
        let now = epoch_now();
        let mut record = Map::new();
        record.insert("t".into(), json!(now));
        record.insert("iso".into(), json!(format_unix_utc_rfc3339(now as u64)));
        record.insert("ev".into(), json!(kind));
        if let Value::Object(extra) = fields {
            for (key, value) in extra {
                record.insert(key, value);
            }
        }
        let line = Value::Object(record).to_string();
        let _ = writeln!(self.file, "{line}");
        let _ = self.file.flush();
        now
    }
}

impl Session {
    fn new(boot_id: String, btime: f64, clk_tck: u64, ncpu: usize) -> Self {
        Self {
            boot_id,
            btime,
            clk_tck,
            ncpu,
            samples: Vec::new(),
            cmds: BTreeMap::new(),
            procs: BTreeMap::new(),
        }
    }

    fn label(&self, key: &str, stat: &ProcStat) -> String {
        let name = self
            .cmds
            .get(key)
            .filter(|cmd| !cmd.is_empty())
            .unwrap_or(&stat.comm);
        let pid = key.split(':').next().unwrap_or("?");
        let short: String = name.chars().take(48).collect();
        format!("{short}({pid})")
    }
}

fn main() {
    let code = match parse_args(std::env::args().skip(1).collect()) {
        Ok(Mode::Monitor(cfg)) => cmd_monitor(cfg),
        Ok(Mode::Retrieve(cfg)) => cmd_retrieve(cfg),
        Ok(Mode::Sample(cfg)) => cmd_sample(cfg),
        Ok(Mode::Report(path)) => cmd_report(&path),
        Err(message) => {
            eprintln!("error: {message}\n\n{USAGE}");
            2
        }
    };
    std::process::exit(code);
}

fn parse_args(args: Vec<String>) -> Result<Mode, String> {
    let mut cfg = MonitorCfg {
        host: "192.168.2.2".into(),
        user: "pi".into(),
        password: "raspberry".into(),
        port: 80,
        path: "/".into(),
        interval: 1.0,
        poll: 1.0,
        settle: 120.0,
        max_wait: 3600.0,
        out_dir: PathBuf::new(),
        targets: DEFAULT_TARGETS
            .iter()
            .map(|target| (*target).to_string())
            .collect(),
        sampler: None,
        proc_swap: false,
        no_agent: false,
        uninstall: false,
    };
    let mut sample = SampleCfg {
        out: PathBuf::from(REMOTE_SAMPLES),
        interval: 1.0,
        duration: 0.0,
        proc_swap: false,
    };
    let mut out_dir: Option<PathBuf> = None;
    let mode = args.first().cloned().unwrap_or_default();
    let mut report_path: Option<PathBuf> = None;
    let mut index = 1;
    while index < args.len() {
        let flag = args[index].as_str();
        let mut value = || -> Result<String, String> {
            index += 1;
            args.get(index)
                .cloned()
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag {
            "--host" => cfg.host = value()?,
            "--user" => cfg.user = value()?,
            "--password" => cfg.password = value()?,
            "--port" => cfg.port = value()?.parse().map_err(|_| "--port must be a number")?,
            "--path" => cfg.path = value()?,
            "--poll" => cfg.poll = parse_secs(&value()?, "--poll")?,
            "--settle" => cfg.settle = parse_secs(&value()?, "--settle")?,
            "--max-wait" => cfg.max_wait = parse_secs(&value()?, "--max-wait")?,
            "--out-dir" => out_dir = Some(PathBuf::from(value()?)),
            "--targets" => {
                cfg.targets = value()?
                    .split(',')
                    .map(|part| part.trim().to_string())
                    .collect();
            }
            "--sampler" => cfg.sampler = Some(PathBuf::from(value()?)),
            "--out" => sample.out = PathBuf::from(value()?),
            "--duration" => sample.duration = parse_secs(&value()?, "--duration")?,
            "--interval" => {
                let secs = parse_secs(&value()?, "--interval")?;
                cfg.interval = secs;
                sample.interval = secs;
            }
            "--proc-swap" => {
                cfg.proc_swap = true;
                sample.proc_swap = true;
            }
            "--no-agent" => cfg.no_agent = true,
            "--uninstall" => cfg.uninstall = true,
            other if mode == "report" && report_path.is_none() && !other.starts_with("--") => {
                report_path = Some(PathBuf::from(other));
            }
            other => return Err(format!("unknown argument {other}")),
        }
        index += 1;
    }
    cfg.out_dir = out_dir.unwrap_or_else(|| PathBuf::from(format!("boot-timeline-{}", cfg.host)));
    match mode.as_str() {
        "monitor" => Ok(Mode::Monitor(cfg)),
        "retrieve" => Ok(Mode::Retrieve(cfg)),
        "sample" => Ok(Mode::Sample(sample)),
        "report" => Ok(Mode::Report(report_path.ok_or("report needs a path")?)),
        other => Err(format!(
            "expected monitor|retrieve|sample|report, got '{other}'"
        )),
    }
}

fn parse_secs(raw: &str, flag: &str) -> Result<f64, String> {
    let secs: f64 = raw.parse().map_err(|_| format!("{flag} must be seconds"))?;
    if secs < 0.0 {
        return Err(format!("{flag} must not be negative"));
    }
    Ok(secs)
}

fn cmd_monitor(cfg: MonitorCfg) -> i32 {
    if !cfg.no_agent && !cfg.password.is_empty() && which("sshpass").is_none() {
        eprintln!(
            "error: sshpass not found (needed for password ssh); install it or use --no-agent"
        );
        return 2;
    }
    let mut samplers = BTreeMap::new();
    if !cfg.no_agent && cfg.sampler.is_none() {
        // Build before the device boots: a toolchain problem must not surface mid-capture.
        for target in &cfg.targets {
            print!("building sampler for {target} ... ");
            let _ = std::io::stdout().flush();
            match build_sampler(target) {
                Ok(path) => {
                    println!("{}", path.display());
                    samplers.insert(target.clone(), path);
                }
                Err(error) => {
                    println!("failed");
                    eprintln!("error: {error}");
                    eprintln!("hint: rustup target add {target}");
                    return 2;
                }
            }
        }
    }
    if let Err(error) = fs::create_dir_all(&cfg.out_dir) {
        eprintln!("error: {}: {error}", cfg.out_dir.display());
        return 2;
    }
    let mut log = match EventLog::open(cfg.out_dir.join("events.jsonl")) {
        Ok(log) => log,
        Err(error) => {
            eprintln!("error: {error}");
            return 2;
        }
    };
    let mut probe = LivePi::new(cfg.clone(), samplers);
    println!(
        "monitoring {} (out={}) - phase1=ping, phase2=http {}:{}{} -> 200, settle={}s",
        cfg.host,
        cfg.out_dir.display(),
        cfg.host,
        cfg.port,
        cfg.path,
        cfg.settle
    );
    let outcome = run_monitor(&cfg, &mut probe, &mut log);
    if !cfg.no_agent {
        if outcome.reason == "settled" {
            let (code, out, err) = probe.uninstall();
            log.event(
                "agent_uninstall",
                json!({"code": code, "detail": format!("{out} {err}")}),
            );
        }
        match probe.retrieve(&cfg.out_dir) {
            Ok(path) => println!("retrieved samples -> {}", path.display()),
            Err(error) => println!("warning: {error}"),
        }
    }
    println!();
    let code = cmd_report(&cfg.out_dir);
    if outcome.phase2.is_none() {
        eprintln!("phase 2 never reached ({})", outcome.reason);
        return 1;
    }
    code
}

fn cmd_retrieve(cfg: MonitorCfg) -> i32 {
    let probe = LivePi::new(cfg.clone(), BTreeMap::new());
    if cfg.uninstall {
        let (code, out, err) = probe.uninstall();
        println!("uninstall: code={code} {out} {err}");
    }
    match probe.retrieve(&cfg.out_dir) {
        Ok(path) => println!("retrieved samples -> {}\n", path.display()),
        Err(error) => {
            eprintln!("error: {error}");
            return 1;
        }
    }
    cmd_report(&cfg.out_dir)
}

fn run_monitor(cfg: &MonitorCfg, probe: &mut dyn Probe, log: &mut EventLog) -> Outcome {
    let mut phase1: Option<f64> = None;
    let mut phase2: Option<f64> = None;
    let mut reboots = 0usize;
    let mut boot_id: Option<String> = None;
    for event in load_events(&log.path) {
        match event.get("ev").and_then(Value::as_str) {
            Some("phase1") => phase1 = phase1.or_else(|| event.get("t").and_then(Value::as_f64)),
            Some("phase2") => phase2 = phase2.or_else(|| event.get("t").and_then(Value::as_f64)),
            Some("reboot") => reboots += 1,
            _ => {}
        }
        if let Some(seen) = event.get("boot_id").and_then(Value::as_str) {
            boot_id = Some(seen.to_string());
        }
    }
    let resumed = phase1.is_some();
    if let (Some(first), Some(ready)) = (phase1, phase2) {
        println!(
            "resuming: phase1 at {}, phase2 at {}, reboots so far={reboots}",
            format_unix_utc_rfc3339(first as u64),
            format_unix_utc_rfc3339(ready as u64)
        );
    } else if let Some(first) = phase1 {
        println!(
            "resuming: phase1 at {}, phase 2 not reached, reboots so far={reboots}",
            format_unix_utc_rfc3339(first as u64)
        );
    }
    log.event(
        "monitor_start",
        json!({"host": cfg.host, "port": cfg.port, "path": cfg.path, "settle": cfg.settle, "resumed": resumed}),
    );

    let started = Instant::now();
    let mut installs = 0u32;
    let mut prev_ping: Option<bool> = None;
    let mut prev_status = String::from("init");
    let mut agent_ok = cfg.no_agent;
    let mut next_agent_try = 0.0f64;
    let mut next_health = 0.0f64;
    let mut deadline = phase2.map(|ready| ready + cfg.settle);
    let mut down_since: Option<f64> = None;
    let mut reason = "settled";

    loop {
        let loop_started = Instant::now();
        let up = probe.ping();

        if up && prev_ping != Some(true) {
            log.event("ping_up", json!({}));
            if phase1.is_none() {
                let now = log.event("phase1", json!({}));
                phase1 = Some(now);
                println!("[{}] phase 1: {} answers ping", stamp(now), cfg.host);
            } else if let Some(down) = down_since {
                let info = if cfg.no_agent { None } else { probe.health() };
                let changed = info
                    .as_ref()
                    .zip(boot_id.as_ref())
                    .map(|(info, seen)| &info.boot_id != seen);
                if changed != Some(false) {
                    reboots += 1;
                    let elapsed = epoch_now() - down;
                    deadline = phase2.map(|_| epoch_now() + cfg.settle);
                    log.event(
                        "reboot",
                        json!({
                            "n": reboots,
                            "boot_id": info.as_ref().map(|info| info.boot_id.clone()),
                            "previous": boot_id,
                            "down_s": (elapsed * 10.0).round() / 10.0,
                        }),
                    );
                    println!(
                        "[{}] reboot #{reboots} detected (down {elapsed:.1}s)",
                        stamp(epoch_now())
                    );
                }
                if let Some(info) = info {
                    boot_id = Some(info.boot_id);
                    agent_ok = agent_ok && unit_ok(&info.unit);
                }
            }
            down_since = None;
            if !cfg.no_agent && !agent_ok {
                next_agent_try = 0.0;
            }
        } else if !up && prev_ping != Some(false) {
            log.event("ping_down", json!({}));
            if prev_ping == Some(true) {
                down_since = Some(epoch_now());
                println!("[{}] host down (reboot?)", stamp(epoch_now()));
            }
        }
        prev_ping = Some(up);

        if up && !cfg.no_agent && !agent_ok && epoch_now() >= next_agent_try {
            let rotate = installs == 0 && !resumed;
            installs += 1;
            match probe.install(rotate) {
                Ok(state) => {
                    agent_ok = true;
                    log.event("agent_install", json!({"ok": true, "detail": state}));
                    println!("[{}] sampler installed", stamp(epoch_now()));
                }
                Err(error) => {
                    log.event("agent_install", json!({"ok": false, "detail": error}));
                    println!("[{}] sampler install failed: {error}", stamp(epoch_now()));
                }
            }
            next_agent_try = epoch_now() + 5.0;
        }

        if up {
            let (status, key) = match probe.http() {
                Ok(status) => (Some(status), status.to_string()),
                Err(error) => (None, format!("err:{error}")),
            };
            if key != prev_status {
                log.event("http", json!({"status": status, "detail": key}));
                println!("[{}] http {key}", stamp(epoch_now()));
                prev_status = key;
            }
            if status == Some(200) && phase2.is_none() {
                let now = log.event("phase2", json!({}));
                phase2 = Some(now);
                deadline = Some(now + cfg.settle);
                let since = phase1.map(|first| now - first).unwrap_or_default();
                println!(
                    "[{}] phase 2: nginx 200 (+{since:.1}s after phase 1); sampling {}s more",
                    stamp(now),
                    cfg.settle
                );
            }
        }

        if up && !cfg.no_agent && agent_ok && epoch_now() >= next_health {
            next_health = epoch_now() + 30.0;
            match probe.health() {
                None => {
                    log.event("health_fail", json!({}));
                }
                Some(info) => {
                    if boot_id.as_ref().is_some_and(|seen| seen != &info.boot_id) {
                        reboots += 1;
                        deadline = phase2.map(|_| epoch_now() + cfg.settle);
                        log.event(
                            "reboot",
                            json!({"n": reboots, "boot_id": info.boot_id, "previous": boot_id, "down_s": Value::Null}),
                        );
                        println!(
                            "[{}] reboot #{reboots} detected (boot_id change)",
                            stamp(epoch_now())
                        );
                    }
                    boot_id = Some(info.boot_id.clone());
                    if unit_ok(&info.unit) {
                        log.event(
                            "health",
                            json!({"boot_id": info.boot_id, "uptime": info.uptime, "samples": info.lines}),
                        );
                    } else {
                        agent_ok = false;
                        next_agent_try = 0.0;
                        log.event("agent_lost", json!({"unit": info.unit}));
                        println!(
                            "[{}] sampler not active ({}), reinstalling",
                            stamp(epoch_now()),
                            info.unit
                        );
                    }
                }
            }
        }

        if deadline.is_some_and(|at| epoch_now() >= at) {
            break;
        }
        if started.elapsed().as_secs_f64() > cfg.max_wait {
            reason = "max_wait";
            log.event("max_wait", json!({}));
            println!(
                "giving up after {:.0}s (--max-wait)",
                started.elapsed().as_secs_f64()
            );
            break;
        }
        let spent = loop_started.elapsed().as_secs_f64();
        if spent < cfg.poll {
            sleep(Duration::from_secs_f64(cfg.poll - spent));
        }
    }

    log.event("monitor_stop", json!({"reason": reason}));
    Outcome {
        phase2,
        reason: reason.to_string(),
    }
}

fn cmd_sample(cfg: SampleCfg) -> i32 {
    if let Some(parent) = cfg.out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut file = match OpenOptions::new().create(true).append(true).open(&cfg.out) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("error: {}: {error}", cfg.out.display());
            return 1;
        }
    };
    let (page_kb, clk_tck) = auxv_page_and_tck();
    let uptime = read_uptime();
    let start = Record::Start {
        t: epoch_now(),
        up: uptime,
        boot_id: fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .unwrap_or_default()
            .trim()
            .to_string(),
        btime: epoch_now() - uptime,
        clk_tck,
        page_kb,
        ncpu: count_cpus(),
        interval: cfg.interval,
        uname: fs::read_to_string("/proc/version")
            .unwrap_or_default()
            .trim()
            .to_string(),
    };
    write_record(&mut file, &start);

    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    let started = Instant::now();
    let mut tick = 0u64;
    loop {
        let procs = read_procs(page_kb, cfg.proc_swap);
        let now = epoch_now();
        for row in &procs {
            let key = format!("{}:{}", row.p, row.s);
            if seen.insert(key, ()).is_none() {
                let cmd = read_cmdline(row.p).unwrap_or_else(|| row.c.clone());
                write_record(
                    &mut file,
                    &Record::Proc {
                        t: now,
                        pid: row.p,
                        st: row.s,
                        cmd,
                    },
                );
            }
        }
        write_record(
            &mut file,
            &Record::Sample {
                t: now,
                up: read_uptime(),
                cpu: read_cpu(),
                mem: read_meminfo(),
                load: read_loadavg(),
                p: procs,
            },
        );
        tick += 1;
        if tick.is_multiple_of(5) {
            let _ = file.sync_data();
        }
        if cfg.duration > 0.0 && started.elapsed().as_secs_f64() >= cfg.duration {
            break;
        }
        let target = cfg.interval * tick as f64;
        let spent = started.elapsed().as_secs_f64();
        if target > spent {
            sleep(Duration::from_secs_f64(target - spent));
        }
    }
    let _ = file.sync_data();
    0
}

fn cmd_report(path: &Path) -> i32 {
    let (events_path, samples_path, out_dir) = if path.is_dir() {
        (
            path.join("events.jsonl"),
            path.join("samples.jsonl"),
            path.to_path_buf(),
        )
    } else {
        let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        (dir.join("events.jsonl"), path.to_path_buf(), dir)
    };
    let events = load_events(&events_path);
    let reference = events
        .iter()
        .find(|event| event.get("ev").and_then(Value::as_str) == Some("phase1"))
        .and_then(|event| event.get("t").and_then(Value::as_f64));

    println!("=== timeline ===");
    if let Some(t0) = reference {
        println!(
            "T0 = {} (phase 1: first ping reply)",
            format_unix_utc_rfc3339(t0 as u64)
        );
    }
    let mut phase1 = None;
    let mut phase2 = None;
    let mut reboots = 0usize;
    let mut event_summary = Vec::new();
    for event in &events {
        let kind = event.get("ev").and_then(Value::as_str).unwrap_or("");
        let time = event.get("t").and_then(Value::as_f64).unwrap_or_default();
        let detail = match kind {
            "http" => event
                .get("detail")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            "reboot" => {
                reboots += 1;
                format!(
                    "#{} down={}s boot_id={}",
                    event.get("n").and_then(Value::as_u64).unwrap_or_default(),
                    event
                        .get("down_s")
                        .map(Value::to_string)
                        .unwrap_or_default(),
                    event
                        .get("boot_id")
                        .and_then(Value::as_str)
                        .map(|id| id.chars().take(8).collect::<String>())
                        .unwrap_or_default()
                )
            }
            "agent_install" => format!(
                "ok={}",
                event.get("ok").and_then(Value::as_bool).unwrap_or(false)
            ),
            "monitor_stop" => format!(
                "reason={}",
                event.get("reason").and_then(Value::as_str).unwrap_or("")
            ),
            "agent_lost" => format!(
                "unit={}",
                event.get("unit").and_then(Value::as_str).unwrap_or("")
            ),
            "phase1" | "phase2" | "ping_up" | "ping_down" | "monitor_start" | "health_fail"
            | "max_wait" => String::new(),
            _ => continue,
        };
        if kind == "phase1" {
            phase1 = Some(time);
        }
        if kind == "phase2" {
            phase2 = Some(time);
        }
        let offset = reference
            .map(|t0| format!("{:+.1}s", time - t0))
            .unwrap_or_else(|| "-".into());
        println!("{offset:<10} {kind:<14} {detail}");
        event_summary.push(json!({
            "offset_s": reference.map(|t0| round1(time - t0)),
            "ev": kind,
            "detail": detail,
        }));
    }

    println!();
    match phase1 {
        Some(first) => println!(
            "phase 1 (ping)      : {}",
            format_unix_utc_rfc3339(first as u64)
        ),
        None => println!("phase 1 (ping)      : not reached"),
    }
    match (phase1, phase2) {
        (Some(first), Some(ready)) => println!(
            "phase 2 (nginx 200) : {}  (+{:.1}s after phase 1)",
            format_unix_utc_rfc3339(ready as u64),
            ready - first
        ),
        _ => println!("phase 2 (nginx 200) : not reached"),
    }
    println!("reboots observed    : {reboots}");

    let sessions = if samples_path.exists() {
        report_samples(&samples_path, reference)
    } else {
        println!("\n(no samples.jsonl in {})", out_dir.display());
        Vec::new()
    };

    let summary = json!({
        "phase1": phase1,
        "phase2": phase2,
        "boot_seconds_phase1_to_phase2": match (phase1, phase2) {
            (Some(first), Some(ready)) => json!(((ready - first) * 10.0).round() / 10.0),
            _ => Value::Null,
        },
        "reboots": reboots,
        "events": event_summary,
        "sessions": sessions,
    });
    let summary_path = out_dir.join("summary.json");
    match fs::write(&summary_path, format!("{summary:#}\n")) {
        Ok(()) => println!("\nsummary -> {}", summary_path.display()),
        Err(error) => println!("\nwarning: {}: {error}", summary_path.display()),
    }
    0
}

fn report_samples(path: &Path, reference: Option<f64>) -> Vec<Value> {
    let mut sessions: Vec<Session> = Vec::new();
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            println!("\nwarning: {}: {error}", path.display());
            return Vec::new();
        }
    };
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        // a reboot can truncate the last line; skip whatever does not parse
        let record: Record = match serde_json::from_str(&line) {
            Ok(record) => record,
            Err(_) => continue,
        };
        match record {
            Record::Start {
                boot_id,
                btime,
                clk_tck,
                ncpu,
                ..
            } => sessions.push(Session::new(boot_id, btime, clk_tck, ncpu)),
            Record::Proc { pid, st, cmd, .. } => {
                if let Some(session) = sessions.last_mut() {
                    session.cmds.insert(format!("{pid}:{st}"), cmd);
                }
            }
            Record::Sample { .. } => {
                if let Some(session) = sessions.last_mut() {
                    if let Record::Sample { p, .. } = &record {
                        for row in p {
                            let key = format!("{}:{}", row.p, row.s);
                            session
                                .procs
                                .entry(key)
                                .and_modify(|stat| {
                                    stat.last = row.k;
                                    stat.peak = stat.peak.max(row.r);
                                })
                                .or_insert(ProcStat {
                                    comm: row.c.clone(),
                                    first: row.k,
                                    last: row.k,
                                    peak: row.r,
                                });
                        }
                    }
                    session.samples.push(record);
                }
            }
            Record::Err { .. } => {}
        }
    }

    println!("\n=== resource sessions (from the device) ===");
    let mut out = Vec::new();
    for (index, session) in sessions.iter().enumerate() {
        if session.samples.is_empty() {
            println!("session {}: no samples", index + 1);
            continue;
        }
        let mut cpu_series = Vec::new();
        let mut mem_used = Vec::new();
        let mut swap_used = Vec::new();
        let mut mem_total = 0.0;
        let mut swap_total = 0.0;
        let mut window = (0.0, 0.0);
        let mut previous: Option<&Vec<u64>> = None;
        for (position, record) in session.samples.iter().enumerate() {
            let Record::Sample { t, cpu, mem, .. } = record else {
                continue;
            };
            if position == 0 {
                window.0 = *t;
                mem_total = mem.get("MemTotal").copied().unwrap_or_default() as f64 / 1024.0;
                swap_total = mem.get("SwapTotal").copied().unwrap_or_default() as f64 / 1024.0;
            }
            window.1 = *t;
            let available = mem
                .get("MemAvailable")
                .or_else(|| mem.get("MemFree"))
                .copied()
                .unwrap_or_default();
            let total = mem.get("MemTotal").copied().unwrap_or_default();
            let swap_free = mem.get("SwapFree").copied().unwrap_or_default();
            let swap_size = mem.get("SwapTotal").copied().unwrap_or_default();
            mem_used.push(total.saturating_sub(available) as f64 / 1024.0);
            swap_used.push(swap_size.saturating_sub(swap_free) as f64 / 1024.0);
            if let Some(before) = previous {
                if let Some(busy) = cpu_percent(before, cpu) {
                    cpu_series.push(busy);
                }
            }
            previous = Some(cpu);
        }

        let mut by_rss: Vec<(&String, &ProcStat)> = session.procs.iter().collect();
        by_rss.sort_by_key(|(_, stat)| std::cmp::Reverse(stat.peak));
        let mut by_cpu: Vec<(&String, &ProcStat)> = session.procs.iter().collect();
        by_cpu.sort_by_key(|(_, stat)| std::cmp::Reverse(stat.last.saturating_sub(stat.first)));
        let ticks = session.clk_tck.max(1) as f64;
        let relative = reference
            .map(|t0| format!(" rel T0 {:+.1}s..{:+.1}s", window.0 - t0, window.1 - t0))
            .unwrap_or_default();

        println!(
            "\nsession {}  boot_id={}  kernel_up={}  samples={}  window={}..{}{relative}",
            index + 1,
            session.boot_id.chars().take(8).collect::<String>(),
            format_unix_utc_rfc3339(session.btime as u64),
            session.samples.len(),
            format_unix_utc_rfc3339(window.0 as u64),
            format_unix_utc_rfc3339(window.1 as u64),
        );
        println!(
            "  cpu%      mean {:.1}  p95 {:.1}  max {:.1}   (100 = all {} cores busy)",
            mean(&cpu_series),
            percentile(&cpu_series, 0.95),
            cpu_series.iter().copied().fold(0.0, f64::max),
            session.ncpu
        );
        println!(
            "  mem MB    mean {:.0}  max {:.0}  of {mem_total:.0} total",
            mean(&mem_used),
            mem_used.iter().copied().fold(0.0, f64::max)
        );
        println!(
            "  swap MB   mean {:.1}  max {:.1}  of {swap_total:.0} total",
            mean(&swap_used),
            swap_used.iter().copied().fold(0.0, f64::max)
        );
        println!(
            "  peak RSS: {}",
            by_rss
                .iter()
                .take(6)
                .map(|(key, stat)| format!(
                    "{} {:.0}MB",
                    session.label(key, stat),
                    stat.peak as f64 / 1024.0
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "  cpu time: {}",
            by_cpu
                .iter()
                .take(6)
                .map(|(key, stat)| format!(
                    "{} {:.1}s",
                    session.label(key, stat),
                    stat.last.saturating_sub(stat.first) as f64 / ticks
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );

        out.push(json!({
            "boot_id": session.boot_id,
            "kernel_boot": session.btime,
            "samples": session.samples.len(),
            "window": [window.0, window.1],
            "cpu_percent": {
                "mean": round1(mean(&cpu_series)),
                "p95": round1(percentile(&cpu_series, 0.95)),
                "max": round1(cpu_series.iter().copied().fold(0.0, f64::max)),
            },
            "mem_mb": {
                "mean": round1(mean(&mem_used)),
                "max": round1(mem_used.iter().copied().fold(0.0, f64::max)),
                "total": round1(mem_total),
            },
            "swap_mb": {
                "mean": round1(mean(&swap_used)),
                "max": round1(swap_used.iter().copied().fold(0.0, f64::max)),
                "total": round1(swap_total),
            },
            "top_rss_mb": by_rss.iter().take(10).map(|(key, stat)| json!({
                "proc": session.label(key, stat),
                "peak_mb": round1(stat.peak as f64 / 1024.0),
            })).collect::<Vec<_>>(),
            "top_cpu_s": by_cpu.iter().take(10).map(|(key, stat)| json!({
                "proc": session.label(key, stat),
                "cpu_s": round1(stat.last.saturating_sub(stat.first) as f64 / ticks),
            })).collect::<Vec<_>>(),
        }));
    }
    out
}

fn build_sampler(target: &str) -> Result<PathBuf, String> {
    let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let out = Command::new("cargo")
        .args([
            "build",
            "--profile",
            "release-device",
            "--bin",
            "boot_timeline",
            "--target",
            target,
            "--manifest-path",
            manifest,
            "--message-format=json-render-diagnostics",
        ])
        .env("RUSTFLAGS", "-C linker=rust-lld")
        .output()
        .map_err(|error| format!("cargo: {error}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo build for {target} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    // the target directory is configurable, so take the path cargo reports
    let mut executable = None;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if message.get("reason").and_then(Value::as_str) == Some("compiler-artifact") {
            if let Some(path) = message.get("executable").and_then(Value::as_str) {
                executable = Some(PathBuf::from(path));
            }
        }
    }
    executable.ok_or_else(|| format!("cargo reported no executable for {target}"))
}

fn read_procs(page_kb: u64, proc_swap: bool) -> Vec<ProcRow> {
    let mut rows = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return rows;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Ok(pid) = name.to_string_lossy().parse::<i64>() else {
            continue;
        };
        let Ok(raw) = fs::read_to_string(format!("/proc/{pid}/stat")) else {
            continue;
        };
        let (Some(open), Some(close)) = (raw.find('('), raw.rfind(')')) else {
            continue;
        };
        let comm = raw[open + 1..close].to_string();
        let rest: Vec<&str> = raw[close + 1..].split_whitespace().collect();
        // rest[0] is field 3 (state), so field N is rest[N - 3]
        if rest.len() < 22 {
            continue;
        }
        let rss = field(&rest, 24) * page_kb;
        if rss == 0 {
            continue;
        }
        rows.push(ProcRow {
            p: pid,
            s: field(&rest, 22),
            c: comm,
            k: field(&rest, 14) + field(&rest, 15),
            r: rss,
            w: if proc_swap { read_proc_swap(pid) } else { 0 },
        });
    }
    rows
}

fn field(rest: &[&str], one_based: usize) -> u64 {
    rest.get(one_based - 3)
        .and_then(|raw| raw.parse().ok())
        .unwrap_or_default()
}

fn read_proc_swap(pid: i64) -> u64 {
    let Ok(raw) = fs::read_to_string(format!("/proc/{pid}/smaps_rollup")) else {
        return 0;
    };
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("Swap:") {
            return rest
                .split_whitespace()
                .next()
                .and_then(|kb| kb.parse().ok())
                .unwrap_or(0);
        }
    }
    0
}

fn read_cmdline(pid: i64) -> Option<String> {
    let raw = fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let text = String::from_utf8_lossy(&raw)
        .replace('\0', " ")
        .trim()
        .to_string();
    if text.is_empty() {
        return None;
    }
    Some(text.chars().take(200).collect())
}

fn read_cpu() -> Vec<u64> {
    fs::read_to_string("/proc/stat")
        .unwrap_or_default()
        .lines()
        .next()
        .map(|line| {
            line.split_whitespace()
                .skip(1)
                .take(8)
                .map(|raw| raw.parse().unwrap_or_default())
                .collect()
        })
        .unwrap_or_default()
}

fn read_meminfo() -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for line in fs::read_to_string("/proc/meminfo")
        .unwrap_or_default()
        .lines()
    {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        if MEM_KEYS.contains(&key) {
            if let Some(kb) = rest
                .split_whitespace()
                .next()
                .and_then(|raw| raw.parse().ok())
            {
                out.insert(key.to_string(), kb);
            }
        }
    }
    out
}

fn read_loadavg() -> Vec<f64> {
    fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .take(3)
        .map(|raw| raw.parse().unwrap_or_default())
        .collect()
}

fn read_uptime() -> f64 {
    fs::read_to_string("/proc/uptime")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or_default()
}

fn count_cpus() -> usize {
    fs::read_to_string("/proc/stat")
        .unwrap_or_default()
        .lines()
        .filter(|line| line.starts_with("cpu") && !line.starts_with("cpu "))
        .count()
        .max(1)
}

/// Page size and USER_HZ come from the auxiliary vector, so RSS and CPU ticks stay right on a
/// device with a different page size than the host running this build.
fn auxv_page_and_tck() -> (u64, u64) {
    const AT_CLKTCK: u64 = 17;
    const AT_PAGESZ: u64 = 6;
    let mut page_kb = 4;
    let mut clk_tck = 100;
    if let Ok(raw) = fs::read("/proc/self/auxv") {
        let width = std::mem::size_of::<usize>();
        for pair in raw.chunks_exact(width * 2) {
            let key = usize_from(&pair[..width]);
            let value = usize_from(&pair[width..]);
            // reject nonsense so a garbled read cannot silently rescale every RSS and CPU number
            match key {
                AT_PAGESZ if value.is_power_of_two() && (1024..=65536).contains(&value) => {
                    page_kb = value / 1024
                }
                AT_CLKTCK if (24..=10_000).contains(&value) => clk_tck = value,
                _ => {}
            }
        }
    }
    (page_kb.max(1), clk_tck)
}

fn usize_from(bytes: &[u8]) -> u64 {
    let mut value = 0u64;
    for (index, byte) in bytes.iter().enumerate().take(8) {
        value |= (*byte as u64) << (8 * index);
    }
    value
}

fn write_record(file: &mut File, record: &Record) {
    match serde_json::to_string(record) {
        Ok(line) => {
            let _ = writeln!(file, "{line}");
            let _ = file.flush();
        }
        Err(error) => eprintln!("serialize: {error}"),
    }
}

fn load_events(path: &Path) -> Vec<Value> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect()
}

fn cpu_percent(before: &[u64], after: &[u64]) -> Option<f64> {
    if before.len() < 5 || after.len() < 5 {
        return None;
    }
    let total: u64 = after
        .iter()
        .sum::<u64>()
        .saturating_sub(before.iter().sum::<u64>());
    if total == 0 {
        return None;
    }
    let idle = after[3].saturating_sub(before[3]) + after[4].saturating_sub(before[4]);
    Some(100.0 * (total.saturating_sub(idle)) as f64 / total as f64)
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn percentile(values: &[f64], fraction: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(|left, right| left.total_cmp(right));
    let index = ((fraction * (ordered.len() - 1) as f64).round() as usize).min(ordered.len() - 1);
    ordered[index]
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn unit_text(interval: f64, proc_swap: bool) -> String {
    let extra = if proc_swap { " --proc-swap" } else { "" };
    format!(
        "[Unit]\n\
         Description=BlueOS boot resource sampler\n\
         After=local-fs.target\n\
         Wants=local-fs.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={REMOTE_BIN} sample --out {REMOTE_SAMPLES} --interval {interval}{extra}\n\
         Restart=always\n\
         RestartSec=1\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n"
    )
}

/// `sudo -A` reads the password from an askpass helper, so it works whether or not the device has
/// passwordless sudo, and leaves stdin free for the heredoc.
fn install_script(password: &str, unit: &str, staged: &str) -> String {
    format!(
        "set -e\n\
         printf '#!/bin/sh\\necho %s\\n' {pw} > {REMOTE_DIR}/askpass\n\
         chmod 700 {REMOTE_DIR}/askpass\n\
         export SUDO_ASKPASS={REMOTE_DIR}/askpass\n\
         cat > {REMOTE_DIR}/unit <<'BOOTUNIT'\n{unit}BOOTUNIT\n\
         sudo -A systemctl stop {UNIT} > /dev/null 2>&1 || true\n\
         mv {staged} {REMOTE_BIN}\n\
         chmod +x {REMOTE_BIN}\n\
         sudo -A cp {REMOTE_DIR}/unit {UNIT_PATH}\n\
         sudo -A systemctl daemon-reload\n\
         sudo -A systemctl enable {UNIT} > /dev/null 2>&1 || true\n\
         sudo -A systemctl restart {UNIT}\n\
         sleep 1\n\
         systemctl is-active {UNIT}\n",
        pw = sh_quote(password),
    )
}

fn unit_ok(state: &str) -> bool {
    // "active" / "activating", but not "inactive" or "failed"
    state.starts_with("activ")
}

fn sh_quote(raw: &str) -> String {
    format!("'{}'", raw.replace('\'', "'\\''"))
}

fn stamp(epoch: f64) -> String {
    format_unix_utc_rfc3339(epoch as u64)
}

fn epoch_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs_f64())
        .unwrap_or_default()
}

fn which(program: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(program))
            .find(|candidate| candidate.is_file())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (ping up, http status, boot_id); the last row repeats once exhausted.
    const TICKS: [(bool, Option<u16>, &str); 10] = [
        (false, None, "b1"),
        (false, None, "b1"),
        (true, None, "b1"),
        (true, Some(502), "b1"),
        (false, None, "b2"),
        (false, None, "b2"),
        (true, Some(502), "b2"),
        (true, Some(200), "b2"),
        (false, None, "b3"),
        (true, Some(200), "b3"),
    ];

    struct Scripted {
        tick: usize,
        installs: u32,
    }

    impl Probe for Scripted {
        fn ping(&mut self) -> bool {
            let up = self.spec().0;
            self.tick += 1;
            up
        }

        fn http(&mut self) -> Result<u16, String> {
            self.spec().1.ok_or_else(|| "no response".to_string())
        }

        fn health(&mut self) -> Option<Health> {
            let spec = self.spec();
            spec.0.then(|| Health {
                boot_id: spec.2.to_string(),
                uptime: 10.0,
                unit: "active".into(),
                lines: 42,
            })
        }

        fn install(&mut self, _rotate: bool) -> Result<String, String> {
            self.installs += 1;
            Ok("active".into())
        }
    }

    impl Scripted {
        fn spec(&self) -> (bool, Option<u16>, &'static str) {
            TICKS[self.tick.min(TICKS.len() - 1)]
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("boot_timeline_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn test_cfg(out_dir: PathBuf) -> MonitorCfg {
        MonitorCfg {
            host: "10.0.0.1".into(),
            user: "pi".into(),
            password: "raspberry".into(),
            port: 80,
            path: "/".into(),
            interval: 1.0,
            poll: 0.05,
            settle: 0.5,
            max_wait: 30.0,
            out_dir,
            targets: Vec::new(),
            sampler: None,
            proc_swap: false,
            no_agent: false,
            uninstall: false,
        }
    }

    #[test]
    fn monitor_tracks_phases_reboots_and_restarts_the_settle_window() {
        let dir = temp_dir("phases");
        let cfg = test_cfg(dir.clone());
        let mut log = EventLog::open(dir.join("events.jsonl")).expect("log");
        let mut probe = Scripted {
            tick: 0,
            installs: 0,
        };
        let outcome = run_monitor(&cfg, &mut probe, &mut log);

        assert_eq!(outcome.reason, "settled");
        assert!(outcome.phase2.is_some(), "phase 2 must be reached");
        assert!(
            probe.installs >= 1,
            "sampler must be installed once reachable"
        );

        let events = load_events(&dir.join("events.jsonl"));
        let stamp_of = |kind: &str| -> Option<f64> {
            events
                .iter()
                .find(|event| event.get("ev").and_then(Value::as_str) == Some(kind))
                .and_then(|event| event.get("t").and_then(Value::as_f64))
        };
        assert!(
            stamp_of("phase2") >= stamp_of("phase1"),
            "phase 2 follows phase 1"
        );
        let kinds: Vec<&str> = events
            .iter()
            .filter_map(|event| event.get("ev").and_then(Value::as_str))
            .collect();
        assert_eq!(kinds.iter().filter(|kind| **kind == "phase1").count(), 1);
        assert_eq!(kinds.iter().filter(|kind| **kind == "phase2").count(), 1);
        let reboot_previous: Vec<String> = events
            .iter()
            .filter(|event| event.get("ev").and_then(Value::as_str) == Some("reboot"))
            .map(|event| {
                event
                    .get("previous")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();
        assert_eq!(
            reboot_previous,
            vec!["b1".to_string(), "b2".to_string()],
            "one reboot before and one after phase 2"
        );
        let last_reboot = events
            .iter()
            .filter(|event| event.get("ev").and_then(Value::as_str) == Some("reboot"))
            .filter_map(|event| event.get("t").and_then(Value::as_f64))
            .next_back()
            .expect("reboot");
        let stop = events
            .iter()
            .filter(|event| event.get("ev").and_then(Value::as_str) == Some("monitor_stop"))
            .filter_map(|event| event.get("t").and_then(Value::as_f64))
            .next_back()
            .expect("stop");
        assert!(
            stop - last_reboot >= cfg.settle,
            "settle window must restart after a late reboot"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn report_groups_sessions_and_tolerates_a_truncated_line() {
        let dir = temp_dir("report");
        let samples = dir.join("samples.jsonl");
        let mut lines = Vec::new();
        for (session, boot) in ["b1", "b2"].iter().enumerate() {
            lines.push(
                json!({"ev": "start", "t": 100.0 + session as f64, "up": 5.0, "boot_id": boot, "btime": 95.0,
                       "clk_tck": 100, "page_kb": 4, "ncpu": 4, "interval": 1.0, "uname": "Linux"})
                .to_string(),
            );
            lines.push(
                json!({"ev": "proc", "t": 100.0, "pid": 7, "st": 1, "cmd": "/usr/bin/dockerd"})
                    .to_string(),
            );
            for step in 0..3u64 {
                lines.push(
                    json!({"ev": "s", "t": 100.0 + step as f64, "up": 5.0 + step as f64,
                           "cpu": [100 * (step + 1), 0, 0, 100, 0, 0, 0, 0],
                           "mem": {"MemTotal": 4_000_000, "MemAvailable": 3_000_000 - 100_000 * step,
                                   "SwapTotal": 102_400, "SwapFree": 102_400 - 1024 * step},
                           "load": [1.0, 0.5, 0.2],
                           "p": [{"p": 7, "s": 1, "c": "dockerd", "k": 50 * (step + 1), "r": 90_112 + 1024 * step, "w": 0}]})
                    .to_string(),
                );
            }
        }
        lines.push("{\"ev\":\"s\",\"t\":123,\"cpu\":[1,2".to_string());
        fs::write(&samples, lines.join("\n") + "\n").expect("write samples");

        let sessions = report_samples(&samples, Some(100.0));
        assert_eq!(sessions.len(), 2, "one session per sampler start");
        let first = &sessions[0];
        assert_eq!(first["samples"], json!(3));
        // busy grows 100 ticks per sample while idle grows 0 -> 100% busy
        assert_eq!(first["cpu_percent"]["max"], json!(100.0));
        assert_eq!(first["mem_mb"]["max"], json!(1171.9));
        assert_eq!(first["swap_mb"]["max"], json!(2.0));
        assert_eq!(first["top_rss_mb"][0]["proc"], json!("/usr/bin/dockerd(7)"));
        assert_eq!(first["top_cpu_s"][0]["cpu_s"], json!(1.0));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unit_state_active_is_not_confused_with_inactive() {
        assert!(unit_ok("active"));
        assert!(unit_ok("activating"));
        assert!(!unit_ok("inactive"));
        assert!(!unit_ok("failed"));
    }

    /// The remote script and unit only ever run on the device, so check them with the local
    /// shell and systemd parsers rather than finding out during a one-shot first boot.
    #[test]
    fn generated_unit_and_install_script_parse() {
        let dir = temp_dir("remote");
        let unit = unit_text(1.0, true);
        let unit_path = dir.join(UNIT);
        fs::write(&unit_path, &unit).expect("write unit");
        assert!(unit.contains("ExecStart=/home/pi/boot-monitor/boot_sampler sample --out"));
        assert!(unit.contains("--proc-swap"));

        let script = install_script("it's", &unit, "/home/pi/boot-monitor/boot_sampler.new");
        assert!(
            script.contains("'it'\\''s'"),
            "password must be shell quoted"
        );
        assert!(
            script.contains("<<'BOOTUNIT'"),
            "heredoc keeps the unit unescaped"
        );

        if which("bash").is_some() {
            let script_path = dir.join("install.sh");
            fs::write(&script_path, &script).expect("write script");
            let out = Command::new("bash")
                .arg("-n")
                .arg(&script_path)
                .output()
                .expect("bash");
            assert!(
                out.status.success(),
                "bash -n: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        if which("systemd-analyze").is_some() {
            let out = Command::new("systemd-analyze")
                .arg("verify")
                .arg(&unit_path)
                .output()
                .expect("systemd-analyze");
            // ExecStart points at the device's filesystem, so ignore that one local-only complaint
            let stderr = String::from_utf8_lossy(&out.stderr);
            let complaints: Vec<&str> = stderr
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.contains("is not executable"))
                .collect();
            assert!(
                complaints.is_empty(),
                "systemd-analyze verify: {complaints:?}"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn password_is_quoted_for_the_remote_shell() {
        assert_eq!(sh_quote("raspberry"), "'raspberry'");
        assert_eq!(sh_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn proc_fields_are_read_by_their_documented_offsets() {
        let rows = read_procs(auxv_page_and_tck().0, false);
        let me = rows
            .iter()
            .find(|row| row.p == std::process::id() as i64)
            .expect("self");
        assert!(me.r > 0, "our own RSS must be positive");
        assert!(!me.c.is_empty());
    }
}
