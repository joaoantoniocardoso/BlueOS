//! Minimal `no_std` logic crate used to exercise workspace gates (ticket #31).

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

/// Returns one so callers can assert the crate links.
///
/// ```
/// assert_eq!(blueos_smoke::ANSWER, 1);
/// ```
pub const ANSWER: u8 = 1;

#[cfg(test)]
mod tests {
    use super::ANSWER;

    #[test]
    fn answer_is_one() {
        assert_eq!(ANSWER, 1);
    }
}
