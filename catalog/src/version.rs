use std::cmp::Ordering;

use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct VersionBound {
    pub release_tag: &'static str,
    pub git_sha: Option<&'static str>,
    pub image_digest: Option<&'static str>,
    pub approx_date: Option<&'static str>,
}

/// Where a catalog feature exists across BlueOS releases.
///
/// Features land on `master` first, then appear on release tags (and sometimes
/// backports). Presence is the full `git tag --contains <intro_commit>` set plus
/// floating channel tips (`master`, `1.4-dev`) recorded at seed time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FeatureAvailability {
    /// Commit that introduced the feature (usually first landed on master).
    pub intro_commit: &'static str,
    /// Every version tag that contains `intro_commit` (includes backports).
    pub present_in_tags: &'static [&'static str],
    /// `git merge-base --is-ancestor intro_commit master` at seed time.
    pub present_on_master: bool,
    /// `git merge-base --is-ancestor intro_commit 1.4-dev` at seed time.
    /// Also used for DUT tags like `1.4-dev-next` (patched 1.4-dev channel).
    pub present_on_1_4_dev: bool,
}

impl FeatureAvailability {
    /// Empty / unset — **not valid** on catalog journeys.
    pub const fn unknown() -> Self {
        Self {
            intro_commit: "",
            present_in_tags: &[],
            present_on_master: false,
            present_on_1_4_dev: false,
        }
    }

    pub fn first_tag(self) -> Option<&'static str> {
        self.present_in_tags.first().copied()
    }

    pub fn present_on_dut(self, dut_tag: &str) -> bool {
        feature_present_on(dut_tag, &self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub enum BlueOsChannel {
    Numbered { major: u32, minor: u32, patch: u32 },
    Master,
    Dev { major: u32, minor: u32 },
    Other(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub enum AvailabilitySkip {
    /// DUT tag/channel does not contain the feature's intro commit.
    NotPresentOnDut {
        dut_tag: String,
        intro_commit: &'static str,
        first_tag: Option<&'static str>,
    },
}

pub const fn bound_tag(tag: &'static str) -> VersionBound {
    VersionBound {
        release_tag: tag,
        git_sha: None,
        image_digest: None,
        approx_date: None,
    }
}

pub fn parse_release_tag(tag: &str) -> BlueOsChannel {
    let tag = tag.strip_prefix('v').unwrap_or(tag);
    if tag == "master" {
        return BlueOsChannel::Master;
    }
    // `1.4-dev`, `1.4-dev-next`, … — floating channel tip + optional suffix.
    if let Some(dev_idx) = tag.find("-dev") {
        let base = &tag[..dev_idx];
        let after_dev = &tag[dev_idx + 4..];
        if after_dev.is_empty() || after_dev.starts_with('-') {
            let mut parts = base.split('.');
            if let (Some(major), Some(minor), None) = (
                parts.next().and_then(|s| s.parse().ok()),
                parts.next().and_then(|s| s.parse().ok()),
                parts.next(),
            ) {
                return BlueOsChannel::Dev { major, minor };
            }
        }
    }
    if let Some((major, minor, patch)) = parse_numbered(tag) {
        return BlueOsChannel::Numbered {
            major,
            minor,
            patch,
        };
    }
    for &known in KNOWN_OTHER_TAGS {
        if tag == known {
            return BlueOsChannel::Other(known);
        }
    }
    BlueOsChannel::Other(UNPARSED_TAG)
}

pub fn cmp_channels(a: &BlueOsChannel, b: &BlueOsChannel) -> Option<Ordering> {
    match (a, b) {
        (
            BlueOsChannel::Numbered {
                major: am,
                minor: ai,
                patch: ap,
            },
            BlueOsChannel::Numbered {
                major: bm,
                minor: bi,
                patch: bp,
            },
        ) => Some(cmp_numbered((*am, *ai, *ap), (*bm, *bi, *bp))),
        (BlueOsChannel::Master, BlueOsChannel::Master) => Some(Ordering::Equal),
        (BlueOsChannel::Master, BlueOsChannel::Numbered { .. }) => Some(Ordering::Greater),
        (BlueOsChannel::Numbered { .. }, BlueOsChannel::Master) => Some(Ordering::Less),
        (BlueOsChannel::Master, BlueOsChannel::Dev { .. }) => Some(Ordering::Greater),
        (BlueOsChannel::Dev { .. }, BlueOsChannel::Master) => Some(Ordering::Less),
        (
            BlueOsChannel::Dev {
                major: am,
                minor: ai,
            },
            BlueOsChannel::Dev {
                major: bm,
                minor: bi,
            },
        ) => Some((am, ai).cmp(&(bm, bi))),
        (BlueOsChannel::Other(a_tag), BlueOsChannel::Other(b_tag)) if a_tag == b_tag => {
            Some(Ordering::Equal)
        }
        (BlueOsChannel::Other(_), BlueOsChannel::Other(_)) => None,
        _ => None,
    }
}

pub fn format_availability_skip_reason(skip: &AvailabilitySkip, dut_tag: &str) -> String {
    match skip {
        AvailabilitySkip::NotPresentOnDut {
            intro_commit,
            first_tag,
            ..
        } => {
            let first = first_tag.unwrap_or("(none)");
            let short = intro_commit.get(..12).unwrap_or(intro_commit);
            format!("not present on {dut_tag} (intro {short}; first tag {first})")
        }
    }
}

/// Whether the feature is present on the DUT's reported version tag.
///
/// - `master` → `present_on_master`
/// - `1.4-dev` / `1.4-dev-*` (e.g. `1.4-dev-next`) → `present_on_1_4_dev`
/// - any other tag → exact membership in `present_in_tags` (covers backports)
pub fn feature_present_on(dut_tag: &str, availability: &FeatureAvailability) -> bool {
    let tag = dut_tag.strip_prefix('v').unwrap_or(dut_tag);
    match parse_release_tag(tag) {
        BlueOsChannel::Master => availability.present_on_master,
        BlueOsChannel::Dev { major: 1, minor: 4 } => availability.present_on_1_4_dev,
        _ => availability.present_in_tags.contains(&tag),
    }
}

pub fn availability_skip(
    dut_tag: &str,
    availability: &FeatureAvailability,
) -> Option<AvailabilitySkip> {
    if feature_present_on(dut_tag, availability) {
        return None;
    }
    Some(AvailabilitySkip::NotPresentOnDut {
        dut_tag: dut_tag.to_string(),
        intro_commit: availability.intro_commit,
        first_tag: availability.first_tag(),
    })
}

pub fn availability_is_valid(a: &FeatureAvailability) -> Result<(), &'static str> {
    if a.intro_commit.is_empty() {
        return Err("intro_commit is required (unknown availability is not allowed)");
    }
    if a.present_in_tags.is_empty() && !a.present_on_master && !a.present_on_1_4_dev {
        return Err("feature must be present on at least one tag or channel tip");
    }
    Ok(())
}

/// Journey ids (as strings) present on a given DUT tag / channel tip.
pub fn journeys_present_on(
    dut_tag: &str,
    presence: &[(&'static str, FeatureAvailability)],
) -> Vec<&'static str> {
    presence
        .iter()
        .filter(|(_, avail)| feature_present_on(dut_tag, avail))
        .map(|(id, _)| *id)
        .collect()
}

const UNPARSED_TAG: &str = "";
const KNOWN_OTHER_TAGS: &[&str] = &[];

fn parse_numbered(tag: &str) -> Option<(u32, u32, u32)> {
    // Strip trailing -beta.N / .betaN for ordering helpers that need X.Y.Z only.
    let base = tag.split("-beta").next().unwrap_or(tag);
    let base = base.split(".beta").next().unwrap_or(base);
    let mut parts = base.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn cmp_numbered(a: (u32, u32, u32), b: (u32, u32, u32)) -> Ordering {
    a.0.cmp(&b.0)
        .then_with(|| a.1.cmp(&b.1))
        .then_with(|| a.2.cmp(&b.2))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TAGS: &[&str] = &[
        "1.4.4-beta.16",
        "1.4.4-beta.17",
        "1.5.0-beta.2",
        "1.5.0-beta.22",
    ];

    const ZENOH_LIKE: FeatureAvailability = FeatureAvailability {
        intro_commit: "127f885b2daf",
        present_in_tags: SAMPLE_TAGS,
        present_on_master: true,
        present_on_1_4_dev: false,
    };

    #[test]
    fn parse_release_tags() {
        assert_eq!(
            parse_release_tag("1.4.4"),
            BlueOsChannel::Numbered {
                major: 1,
                minor: 4,
                patch: 4,
            }
        );
        assert_eq!(parse_release_tag("master"), BlueOsChannel::Master);
        assert_eq!(
            parse_release_tag("1.4-dev"),
            BlueOsChannel::Dev { major: 1, minor: 4 }
        );
        assert_eq!(
            parse_release_tag("1.4-dev-next"),
            BlueOsChannel::Dev { major: 1, minor: 4 }
        );
    }

    #[test]
    fn presence_includes_backports_and_excludes_missing_channels() {
        assert!(feature_present_on("1.4.4-beta.16", &ZENOH_LIKE));
        assert!(feature_present_on("1.5.0-beta.2", &ZENOH_LIKE));
        assert!(feature_present_on("master", &ZENOH_LIKE));
        assert!(!feature_present_on("1.4-dev", &ZENOH_LIKE));
        assert!(!feature_present_on("1.4-dev-next", &ZENOH_LIKE));
        assert!(!feature_present_on("1.4.0", &ZENOH_LIKE));
    }

    #[test]
    fn presence_1_4_dev_next_follows_1_4_dev_tip() {
        let wifi_like = FeatureAvailability {
            intro_commit: "732b3ac2997b",
            present_in_tags: &["1.0.0.beta1"],
            present_on_master: true,
            present_on_1_4_dev: true,
        };
        assert!(feature_present_on("1.4-dev", &wifi_like));
        assert!(feature_present_on("1.4-dev-next", &wifi_like));
        assert!(availability_skip("1.4-dev-next", &wifi_like).is_none());
    }

    #[test]
    fn availability_skip_when_absent() {
        let skip = availability_skip("1.4-dev", &ZENOH_LIKE).expect("skip");
        match skip {
            AvailabilitySkip::NotPresentOnDut { dut_tag, .. } => assert_eq!(dut_tag, "1.4-dev"),
        }
        assert!(availability_skip("master", &ZENOH_LIKE).is_none());
        assert!(availability_skip("1.5.0-beta.22", &ZENOH_LIKE).is_none());
    }

    #[test]
    fn availability_is_valid_rejects_unknown() {
        assert!(availability_is_valid(&FeatureAvailability::unknown()).is_err());
        assert!(availability_is_valid(&ZENOH_LIKE).is_ok());
    }

    #[test]
    fn journeys_present_on_filters() {
        let table = [("InspectZenohNetwork", ZENOH_LIKE), ("Other", ZENOH_LIKE)];
        let on_14dev = journeys_present_on("1.4-dev", &table);
        assert!(on_14dev.is_empty());
        let on_master = journeys_present_on("master", &table);
        assert_eq!(on_master.len(), 2);
    }
}
