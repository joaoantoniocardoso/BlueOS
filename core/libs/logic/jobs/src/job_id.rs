use alloc::string::{String, ToString};
use core::{fmt, str::FromStr};

const JOB_ID_TEXT_LEN: usize = 36;
const JOB_ID_HEX_RADIX: u32 = 16;
const JOB_ID_HEX_SHIFT_BITS: u32 = 4;
const JOB_ID_HYPHEN_INDEXES: [usize; 4] = [8, 13, 18, 23];

/// Identifies a Job: the UUID the client generated when it submitted it, written as text on the wire.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "String", into = "String")
)]
pub struct JobId(u128);

/// The text is not a UUID such as `0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the Job id is not a UUID")]
pub struct InvalidJobId;

impl fmt::Display for JobId {
    /// The UUID in its hyphenated lowercase form.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.0;
        write!(
            formatter,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            value >> 96,
            (value >> 80) & 0xffff,
            (value >> 64) & 0xffff,
            (value >> 48) & 0xffff,
            value & 0xffff_ffff_ffff,
        )
    }
}

impl FromStr for JobId {
    type Err = InvalidJobId;

    /// Parses the hyphenated form of a UUID, in either case.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.len() != JOB_ID_TEXT_LEN {
            return Err(InvalidJobId);
        }
        let mut value = 0_u128;
        for (index, character) in text.chars().enumerate() {
            if JOB_ID_HYPHEN_INDEXES.contains(&index) {
                if character != '-' {
                    return Err(InvalidJobId);
                }
                continue;
            }
            let digit = character.to_digit(JOB_ID_HEX_RADIX).ok_or(InvalidJobId)?;
            value = (value << JOB_ID_HEX_SHIFT_BITS) | u128::from(digit);
        }
        Ok(Self(value))
    }
}

impl TryFrom<String> for JobId {
    type Error = InvalidJobId;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl JobId {
    /// The id whose UUID is the 128 bits of `value`.
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }
}

impl From<JobId> for String {
    fn from(job_id: JobId) -> Self {
        job_id.to_string()
    }
}
