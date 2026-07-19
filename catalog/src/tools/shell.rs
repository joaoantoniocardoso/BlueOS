use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("catalog is nested under repo root")
        .to_path_buf()
}

pub fn catalog_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn run(cmd: &[&str], cwd: &Path) -> Result<String, String> {
    let output = Command::new(cmd[0])
        .args(&cmd[1..])
        .current_dir(cwd)
        .output()
        .map_err(|err| format!("failed to spawn {}: {err}", cmd[0]))?;
    if !output.status.success() {
        return Err(format!(
            "cmd failed ({}): {}\n{}",
            output.status.code().unwrap_or(-1),
            cmd.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

pub fn run_ok(cmd: &[&str], cwd: &Path) -> Option<String> {
    run(cmd, cwd).ok().filter(|s| !s.is_empty())
}

pub fn sleep_cold() {
    std::thread::sleep(Duration::from_millis(50));
}
