//! Non-default CDR regression vectors grouped by message family.

mod layout;
mod log_and_ack;
mod old_writer;
mod recording;

use super::fixture::CdrVector;

pub(crate) fn extra_vectors() -> Vec<CdrVector> {
    let mut vectors = Vec::new();
    vectors.extend(log_and_ack::vectors());
    vectors.extend(recording::vectors());
    vectors.extend(old_writer::vectors());
    vectors.extend(layout::vectors());
    vectors
}
