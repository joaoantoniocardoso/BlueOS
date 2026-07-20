use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

/// Backoff schedule (seconds) between retries of a transient `gh` failure.
/// Total attempts = `RETRY_BACKOFFS_SECS.len() + 1`.
const RETRY_BACKOFFS_SECS: [u64; 3] = [5, 15, 30];

/// Process-wide backoff when GitHub returns 403 / rate-limit errors so parallel
/// `--jobs N` workers serialize waits instead of stampeding retries.
const RATE_LIMIT_BACKOFFS_SECS: [u64; 4] = [10, 30, 60, 120];

struct GhRateLimitState {
    backoff_index: usize,
}

static GH_RATE_LIMIT: Mutex<GhRateLimitState> = Mutex::new(GhRateLimitState { backoff_index: 0 });

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

/// GitHub 403 / abuse / rate-limit failures retried with [`GH_RATE_LIMIT`] coordination.
pub fn is_github_rate_limited(err: &str) -> bool {
    let lower = err.to_lowercase();
    const NEEDLES: &[&str] = &[
        "403",
        "rate limit",
        "rate_limit",
        "ratelimit",
        "abuse",
        "secondary rate limit",
    ];
    NEEDLES.iter().any(|needle| lower.contains(needle))
}

/// Transient `gh`/network failures worth retrying: 5xx gateway errors, timeouts,
/// and GitHub rate limits (coordinated via [`GH_RATE_LIMIT`]).
/// Anything else (404, auth, bad args) is a permanent failure — retrying wastes time.
pub fn is_transient(err: &str) -> bool {
    if is_github_rate_limited(err) {
        return true;
    }
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

fn wait_on_shared_rate_limit<S: Fn(u64)>(sleep: &S) {
    let mut guard = GH_RATE_LIMIT.lock().unwrap_or_else(|e| e.into_inner());
    let idx = guard.backoff_index.min(RATE_LIMIT_BACKOFFS_SECS.len() - 1);
    let secs = RATE_LIMIT_BACKOFFS_SECS[idx];
    if guard.backoff_index < RATE_LIMIT_BACKOFFS_SECS.len() - 1 {
        guard.backoff_index += 1;
    }
    sleep(secs);
}

fn reset_shared_rate_limit() {
    if let Ok(mut guard) = GH_RATE_LIMIT.lock() {
        guard.backoff_index = 0;
    }
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
            Ok(out) => {
                reset_shared_rate_limit();
                return Ok(out);
            }
            Err(err) if is_github_rate_limited(&err) => wait_on_shared_rate_limit(&sleep),
            Err(err) if is_transient(&err) => sleep(backoff),
            Err(err) => return Err(err),
        }
    }
    match attempt() {
        Ok(out) => {
            reset_shared_rate_limit();
            Ok(out)
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod retry_tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn is_transient_matches_5xx_and_timeouts() {
        reset_shared_rate_limit();
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
    fn is_github_rate_limited_matches_403_and_abuse() {
        reset_shared_rate_limit();
        assert!(is_github_rate_limited("HTTP 403: Forbidden"));
        assert!(is_github_rate_limited("API rate limit exceeded"));
        assert!(is_github_rate_limited("secondary rate limit"));
        assert!(is_github_rate_limited("abuse detection mechanism"));
        assert!(!is_github_rate_limited("HTTP 404: Not Found"));
        assert!(!is_github_rate_limited("HTTP 401: Bad credentials"));
        assert!(is_transient("HTTP 403: rate limit exceeded"));
    }

    #[test]
    fn run_with_retry_retries_rate_limit_with_shared_backoff() {
        reset_shared_rate_limit();
        let calls = RefCell::new(0);
        let sleeps: RefCell<Vec<u64>> = RefCell::new(vec![]);
        let attempt = || {
            *calls.borrow_mut() += 1;
            if *calls.borrow() < 3 {
                Err("HTTP 403: rate limit exceeded".to_string())
            } else {
                Ok("ok".to_string())
            }
        };
        let result = run_with_retry_using(attempt, |secs| sleeps.borrow_mut().push(secs));
        assert_eq!(result, Ok("ok".to_string()));
        assert_eq!(*calls.borrow(), 3);
        assert_eq!(*sleeps.borrow(), vec![10, 30]);
    }

    #[test]
    fn parallel_workers_retry_rate_limit_without_panic() {
        reset_shared_rate_limit();
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    let calls = RefCell::new(0);
                    let result = run_with_retry_using(
                        || {
                            *calls.borrow_mut() += 1;
                            if *calls.borrow() < 2 {
                                Err("secondary rate limit".to_string())
                            } else {
                                Ok("ok".to_string())
                            }
                        },
                        |_| {},
                    );
                    assert_eq!(result, Ok("ok".to_string()));
                });
            }
        });
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
