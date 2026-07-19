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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FeatureAvailability {
    pub introduced_in: Option<VersionBound>,
    pub removed_in: Option<VersionBound>,
}

impl FeatureAvailability {
    pub const fn unknown() -> Self {
        Self {
            introduced_in: None,
            removed_in: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub enum BlueOsChannel {
    Numbered { major: u32, minor: u32, patch: u32 },
    Master,
    Other(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub enum AvailabilitySkip {
    NotYetIntroduced { introduced_in: &'static str },
    RemovedInVersion { removed_in: &'static str },
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
        (BlueOsChannel::Other(a_tag), BlueOsChannel::Other(b_tag)) if a_tag == b_tag => {
            Some(Ordering::Equal)
        }
        (BlueOsChannel::Other(_), BlueOsChannel::Other(_)) => None,
        _ => None,
    }
}

pub fn format_availability_skip_reason(skip: &AvailabilitySkip, dut_tag: &str) -> String {
    match skip {
        AvailabilitySkip::NotYetIntroduced { introduced_in } => {
            format!("not introduced until {introduced_in} (dut {dut_tag})")
        }
        AvailabilitySkip::RemovedInVersion { removed_in } => {
            format!("removed in {removed_in} (dut {dut_tag})")
        }
    }
}

pub fn availability_skip(
    dut_tag: &str,
    availability: &FeatureAvailability,
) -> Option<AvailabilitySkip> {
    if let Some(introduced) = availability.introduced_in.as_ref() {
        match cmp_release_tags(dut_tag, introduced.release_tag) {
            Some(Ordering::Less) => {
                return Some(AvailabilitySkip::NotYetIntroduced {
                    introduced_in: introduced.release_tag,
                });
            }
            None => {}
            Some(_) => {}
        }
    }

    if let Some(removed) = availability.removed_in.as_ref() {
        match cmp_release_tags(dut_tag, removed.release_tag) {
            Some(Ordering::Greater) | Some(Ordering::Equal) => {
                return Some(AvailabilitySkip::RemovedInVersion {
                    removed_in: removed.release_tag,
                });
            }
            None => {}
            Some(Ordering::Less) => {}
        }
    }

    None
}

pub fn availability_is_valid(a: &FeatureAvailability) -> Result<(), &'static str> {
    let (Some(introduced), Some(removed)) = (&a.introduced_in, &a.removed_in) else {
        return Ok(());
    };

    match cmp_release_tags(introduced.release_tag, removed.release_tag) {
        Some(Ordering::Less) => Ok(()),
        Some(Ordering::Equal) => Err("introduced_in must be strictly before removed_in"),
        Some(Ordering::Greater) => Err("introduced_in must be strictly before removed_in"),
        None => Err("introduced_in and removed_in are not comparable"),
    }
}

const UNPARSED_TAG: &str = "";
const KNOWN_OTHER_TAGS: &[&str] = &[];

fn cmp_release_tags(a: &str, b: &str) -> Option<Ordering> {
    let a = a.strip_prefix('v').unwrap_or(a);
    let b = b.strip_prefix('v').unwrap_or(b);

    if a == "master" && b == "master" {
        return Some(Ordering::Equal);
    }
    if a == "master" {
        return Some(Ordering::Greater);
    }
    if b == "master" {
        return Some(Ordering::Less);
    }

    if let (Some(na), Some(nb)) = (parse_numbered(a), parse_numbered(b)) {
        return Some(cmp_numbered(na, nb));
    }

    if parse_numbered(a).is_some() || parse_numbered(b).is_some() {
        return None;
    }

    if a == b {
        Some(Ordering::Equal)
    } else {
        None
    }
}

fn parse_numbered(tag: &str) -> Option<(u32, u32, u32)> {
    let mut parts = tag.split('.');
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
        assert_eq!(
            parse_release_tag("v1.5.0"),
            BlueOsChannel::Numbered {
                major: 1,
                minor: 5,
                patch: 0,
            }
        );
        assert_eq!(parse_release_tag("master"), BlueOsChannel::Master);
    }

    #[test]
    fn channel_ordering() {
        let a = parse_release_tag("1.4.4");
        let b = parse_release_tag("1.5.0");
        let m = parse_release_tag("master");
        assert_eq!(cmp_channels(&a, &b), Some(Ordering::Less));
        assert_eq!(cmp_channels(&b, &m), Some(Ordering::Less));
        assert_eq!(cmp_channels(&a, &m), Some(Ordering::Less));
    }

    #[test]
    fn format_availability_skip_reason_strings() {
        assert_eq!(
            format_availability_skip_reason(
                &AvailabilitySkip::NotYetIntroduced {
                    introduced_in: "1.5.0",
                },
                "1.4.4",
            ),
            "not introduced until 1.5.0 (dut 1.4.4)"
        );
        assert_eq!(
            format_availability_skip_reason(
                &AvailabilitySkip::RemovedInVersion {
                    removed_in: "1.5.0",
                },
                "1.5.1",
            ),
            "removed in 1.5.0 (dut 1.5.1)"
        );
    }

    #[test]
    fn availability_skip_not_introduced() {
        let availability = FeatureAvailability {
            introduced_in: Some(bound_tag("1.5.0")),
            removed_in: None,
        };
        assert_eq!(
            availability_skip("1.4.4", &availability),
            Some(AvailabilitySkip::NotYetIntroduced {
                introduced_in: "1.5.0",
            })
        );
    }

    #[test]
    fn availability_skip_removed() {
        let availability = FeatureAvailability {
            introduced_in: None,
            removed_in: Some(bound_tag("1.5.0")),
        };
        assert_eq!(
            availability_skip("1.5.0", &availability),
            Some(AvailabilitySkip::RemovedInVersion {
                removed_in: "1.5.0",
            })
        );
        assert_eq!(
            availability_skip("1.6.0", &availability),
            Some(AvailabilitySkip::RemovedInVersion {
                removed_in: "1.5.0",
            })
        );
    }

    #[test]
    fn availability_skip_always_available() {
        let availability = FeatureAvailability::unknown();
        assert_eq!(availability_skip("1.4.4", &availability), None);
        assert_eq!(availability_skip("master", &availability), None);
    }

    #[test]
    fn availability_skip_master_has_introduced_feature() {
        let availability = FeatureAvailability {
            introduced_in: Some(bound_tag("1.5.0")),
            removed_in: None,
        };
        assert_eq!(availability_skip("master", &availability), None);
    }

    #[test]
    fn invalid_introduced_not_before_removed() {
        let availability = FeatureAvailability {
            introduced_in: Some(bound_tag("1.5.0")),
            removed_in: Some(bound_tag("1.4.0")),
        };
        assert!(availability_is_valid(&availability).is_err());

        let equal = FeatureAvailability {
            introduced_in: Some(bound_tag("1.5.0")),
            removed_in: Some(bound_tag("1.5.0")),
        };
        assert!(availability_is_valid(&equal).is_err());

        let valid = FeatureAvailability {
            introduced_in: Some(bound_tag("1.4.0")),
            removed_in: Some(bound_tag("1.5.0")),
        };
        assert!(availability_is_valid(&valid).is_ok());
    }
}
