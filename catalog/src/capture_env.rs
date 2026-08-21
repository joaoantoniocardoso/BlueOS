//! Runtime capture image identity for catalog provenance.
//!
//! The floating Docker tag `master` is **not** a pin — the digest is. The DUT
//! image may move while the tag stays the same; captures must record the digest.

pub const RUNTIME_CAPTURE_CORE_DIGEST: &str =
    "sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e";
/// `bluerobotics/blueos-core:master` on the Tier-2 smoke play Pi when smoke was last authored.
pub const TIER2_SMOKE_DUT_CORE_DIGEST: &str =
    "sha256:ae50d2d1d5935db0d764e2f14837039ff498d78abf22eb3c8f3fd68aa3fe4b76";
pub const RUNTIME_CAPTURE_CORE_REPO_DIGEST: &str =
    "sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a";
pub const RUNTIME_CAPTURE_CORE_TAG: &str = "master";
pub const RUNTIME_CAPTURE_CORE_REPO: &str = "bluerobotics/blueos-core";

/// Canonical environment string used in `Provenance::runtime` environment fields.
pub const RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub const RUNTIME_CAPTURE_ENV_PI4: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4";

pub const RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e; RepoDigest sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

/// Pi 4 SITL capture environment (same image digest as Navigator captures).
pub const RUNTIME_CAPTURE_ENV_PI4_SITL: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, SITL";

pub const RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_ARDUSUB: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator, ArduSub 4.5.3 STABLE";

pub const RUNTIME_CAPTURE_CORE_DIGEST_1_4_DEV: &str =
    "sha256:5b50dfafc3114651d459993c0c65639d7205634613fb62e0dea5ee04de8eebb1";
pub const RUNTIME_CAPTURE_CORE_TAG_1_4_DEV: &str = "1.4-dev";

pub const RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_1_4_DEV: &str =
    "BlueOS 1.4-dev (bluerobotics/blueos-core:1.4-dev @ sha256:5b50dfafc3114651d459993c0c65639d7205634613fb62e0dea5ee04de8eebb1), Raspberry Pi 4, Navigator";

pub fn format_runtime_env(board_notes: &str) -> String {
    format!(
        "BlueOS {} ({}:{} @ {}), Raspberry Pi 4, {}",
        RUNTIME_CAPTURE_CORE_TAG,
        RUNTIME_CAPTURE_CORE_REPO,
        RUNTIME_CAPTURE_CORE_TAG,
        RUNTIME_CAPTURE_CORE_DIGEST,
        board_notes,
    )
}
