use tracing::warn;

pub fn example() {
    let error = "failed";
    warn!("Operation failed: {error}");
}
