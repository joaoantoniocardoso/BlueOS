use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Backoff schedule (seconds) between retries of a transient `gh` failure.
/// Total attempts = `RETRY_BACKOFFS_SECS.len() + 1`.
const RETRY_BACKOFFS_SECS: [u64; 3] = [5, 15, 30];

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

/// Transient `gh`/network failures worth retrying: 5xx gateway errors and timeouts.
/// Anything else (404, auth, bad args) is a permanent failure — retrying wastes time.
pub fn is_transient(err: &str) -> bool {
    let lower = err.to_lowercase();
    const NEEDLES: &[&str] = &[
        "502",
        "503",
        "504",
        "bad gateway",
        "gateway timeout",
        "service unavailable",
        "timed out",
        "timeout",
        "connection reset",
        "temporarily unavailable",
    ];
    NEEDLES.iter().any(|needle| lower.contains(needle))
}

/// Retries `attempt` on [`is_transient`] errors with the [`RETRY_BACKOFFS_SECS`]
/// schedule; non-transient errors return immediately without retrying.
pub fn run_with_retry<F: FnMut() -> Result<String, String>>(attempt: F) -> Result<String, String> {
    run_with_retry_using(attempt, |secs| {
        std::thread::sleep(Duration::from_secs(secs))
    })
}

fn run_with_retry_using<F, S>(mut attempt: F, sleep: S) -> Result<String, String>
where
    F: FnMut() -> Result<String, String>,
    S: Fn(u64),
{
    for backoff in RETRY_BACKOFFS_SECS {
        match attempt() {
            Ok(out) => return Ok(out),
            Err(err) if is_transient(&err) => sleep(backoff),
            Err(err) => return Err(err),
        }
    }
    attempt()
}

#[cfg(test)]
mod retry_tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn is_transient_matches_5xx_and_timeouts() {
        assert!(is_transient(
            "cmd failed (1): gh pr view\nHTTP 502: Bad Gateway"
        ));
        assert!(is_transient(
            "error: context deadline exceeded (Client.Timeout exceeded)"
        ));
        assert!(is_transient("gh: Gateway Timeout (HTTP 504)"));
        assert!(!is_transient("HTTP 404: Not Found"));
        assert!(!is_transient("HTTP 401: Bad credentials"));
    }

    #[test]
    fn run_with_retry_retries_transient_then_succeeds_without_real_sleep() {
        let calls = RefCell::new(0);
        let sleeps: RefCell<Vec<u64>> = RefCell::new(vec![]);
        let attempt = || {
            *calls.borrow_mut() += 1;
            if *calls.borrow() < 3 {
                Err("HTTP 503: Service Unavailable".to_string())
            } else {
                Ok("ok".to_string())
            }
        };
        let result = run_with_retry_using(attempt, |secs| sleeps.borrow_mut().push(secs));
        assert_eq!(result, Ok("ok".to_string()));
        assert_eq!(*calls.borrow(), 3);
        assert_eq!(*sleeps.borrow(), vec![5, 15]);
    }

    #[test]
    fn run_with_retry_does_not_retry_non_transient() {
        let calls = RefCell::new(0);
        let attempt = || {
            *calls.borrow_mut() += 1;
            Err::<String, String>("HTTP 404: Not Found".to_string())
        };
        let result = run_with_retry_using(attempt, |_| {
            panic!("should not sleep on non-transient error")
        });
        assert!(result.is_err());
        assert_eq!(*calls.borrow(), 1);
    }

    #[test]
    fn run_with_retry_gives_up_after_max_attempts() {
        let calls = RefCell::new(0);
        let attempt = || {
            *calls.borrow_mut() += 1;
            Err::<String, String>("HTTP 503: Service Unavailable".to_string())
        };
        let result = run_with_retry_using(attempt, |_| {});
        assert!(result.is_err());
        assert_eq!(*calls.borrow(), RETRY_BACKOFFS_SECS.len() + 1);
    }
}
