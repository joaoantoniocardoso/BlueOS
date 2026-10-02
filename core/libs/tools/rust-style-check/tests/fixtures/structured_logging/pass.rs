use tracing::warn;

pub fn example() {
    let error = "failed";
    warn!(message = %error, "Operation failed");
}
