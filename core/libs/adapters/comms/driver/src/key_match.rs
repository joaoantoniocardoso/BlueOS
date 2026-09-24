/// Returns whether `key` matches a Zenoh-style key expression (`*` = one segment, `**` = any depth).
pub fn key_matches(key_expression: &str, key: &str) -> bool {
    let pattern_segments: Vec<&str> = key_expression.split('/').collect();
    let key_segments: Vec<&str> = key.split('/').collect();
    key_matches_segments(&pattern_segments, &key_segments, 0, 0)
}

fn key_matches_segments(
    pattern: &[&str],
    key: &[&str],
    pattern_index: usize,
    key_index: usize,
) -> bool {
    if pattern_index == pattern.len() {
        return key_index == key.len();
    }
    let segment = pattern[pattern_index];
    if segment == "**" {
        if pattern_index == pattern.len() - 1 {
            return true;
        }
        for skip in key_index..=key.len() {
            if key_matches_segments(pattern, key, pattern_index + 1, skip) {
                return true;
            }
        }
        return false;
    }
    if key_index >= key.len() {
        return false;
    }
    if segment == "*" || segment == key[key_index] {
        return key_matches_segments(pattern, key, pattern_index + 1, key_index + 1);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::key_matches;

    #[test]
    fn exact_key() {
        assert!(key_matches("blueos/v1/foo/state", "blueos/v1/foo/state"));
        assert!(!key_matches("blueos/v1/foo/state", "blueos/v1/foo/events"));
    }

    #[test]
    fn single_wildcard() {
        assert!(key_matches("blueos/v1/*/state", "blueos/v1/recorder/state"));
        assert!(!key_matches("blueos/v1/*/state", "blueos/v1/a/b/state"));
    }

    #[test]
    fn double_wildcard() {
        assert!(key_matches("blueos/**", "blueos/v1/foo"));
        assert!(key_matches("blueos/v1/**/log", "blueos/v1/svc/log"));
    }
}
